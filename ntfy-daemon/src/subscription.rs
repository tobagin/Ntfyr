use crate::listener::{ListenerEvent, ListenerHandle};
use crate::models::{self, ReceivedMessage};
use crate::{Error, SharedEnv};
use tokio::select;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio::task::spawn_local;
use tracing::{debug, error, info, trace, warn};

#[derive(Debug)]
enum SubscriptionCommand {
    GetModel {
        resp_tx: oneshot::Sender<models::Subscription>,
    },
    UpdateInfo {
        new_model: models::Subscription,
        resp_tx: oneshot::Sender<anyhow::Result<()>>,
    },
    Attach {
        resp_tx: oneshot::Sender<(Vec<ListenerEvent>, broadcast::Receiver<ListenerEvent>)>,
    },
    Publish {
        msg: models::OutgoingMessage,
        encrypt: bool,
        resp_tx: oneshot::Sender<anyhow::Result<()>>,
    },
    ClearNotifications {
        resp_tx: oneshot::Sender<anyhow::Result<()>>,
    },
    UpdateReadUntil {
        timestamp: u64,
        resp_tx: oneshot::Sender<anyhow::Result<()>>,
    },
}

#[derive(Clone)]
pub struct SubscriptionHandle {
    command_tx: mpsc::Sender<SubscriptionCommand>,
    listener: ListenerHandle,
}

impl SubscriptionHandle {
    pub fn new(listener: ListenerHandle, model: models::Subscription, env: &SharedEnv) -> Self {
        let (command_tx, command_rx) = mpsc::channel(32);
        let broadcast_tx = broadcast::channel(8).0;
        let actor = SubscriptionActor {
            listener: listener.clone(),
            model,
            command_rx,
            env: env.clone(),
            broadcast_tx: broadcast_tx.clone(),
        };
        spawn_local(actor.run());
        Self {
            command_tx,
            listener,
        }
    }

    pub async fn model(&self) -> models::Subscription {
        let (resp_tx, resp_rx) = oneshot::channel();
        self.command_tx
            .send(SubscriptionCommand::GetModel { resp_tx })
            .await
            .unwrap();
        resp_rx.await.unwrap()
    }

    pub async fn update_info(&self, new_model: models::Subscription) -> anyhow::Result<()> {
        let (resp_tx, resp_rx) = oneshot::channel();
        self.command_tx
            .send(SubscriptionCommand::UpdateInfo { new_model, resp_tx })
            .await?;
        resp_rx.await.unwrap()
    }

    pub async fn restart(&self) -> anyhow::Result<()> {
        self.listener
            .commands
            .send(crate::ListenerCommand::Restart)
            .await?;
        Ok(())
    }

    pub async fn shutdown(&self) -> anyhow::Result<()> {
        self.listener
            .commands
            .send(crate::ListenerCommand::Shutdown)
            .await?;
        Ok(())
    }

    // returns a vector containing all the past messages stored in the database and the current connection state.
    // The first vector is useful to get a summary of what happened before.
    // The `ListenerHandle` is returned to receive new events.
    pub async fn attach(&self) -> (Vec<ListenerEvent>, broadcast::Receiver<ListenerEvent>) {
        let (resp_tx, resp_rx) = oneshot::channel();
        self.command_tx
            .send(SubscriptionCommand::Attach { resp_tx })
            .await
            .unwrap();
        resp_rx.await.unwrap()
    }

    pub async fn publish(&self, msg: models::OutgoingMessage, encrypt: bool) -> anyhow::Result<()> {
        let (resp_tx, resp_rx) = oneshot::channel();
        self.command_tx
            .send(SubscriptionCommand::Publish { msg, encrypt, resp_tx })
            .await
            .unwrap();
        resp_rx.await.unwrap()
    }

    pub async fn clear_notifications(&self) -> anyhow::Result<()> {
        let (resp_tx, resp_rx) = oneshot::channel();
        self.command_tx
            .send(SubscriptionCommand::ClearNotifications { resp_tx })
            .await
            .unwrap();
        resp_rx.await.unwrap()
    }

    pub async fn update_read_until(&self, timestamp: u64) -> anyhow::Result<()> {
        let (resp_tx, resp_rx) = oneshot::channel();
        self.command_tx
            .send(SubscriptionCommand::UpdateReadUntil { timestamp, resp_tx })
            .await
            .unwrap();
        resp_rx.await.unwrap()
    }
}

struct SubscriptionActor {
    listener: ListenerHandle,
    model: models::Subscription,
    command_rx: mpsc::Receiver<SubscriptionCommand>,
    env: SharedEnv,
    broadcast_tx: broadcast::Sender<ListenerEvent>,
}

impl SubscriptionActor {
    async fn run(mut self) {
        loop {
            select! {
                Ok(event) = self.listener.events.recv() => {
                    debug!(?event, "received listener event");
                    match event {
                        ListenerEvent::Message(msg) => self.handle_msg_event(msg),
                        other => {
                            let _ = self.broadcast_tx.send(other);
                        }
                    }
                }
                Some(command) = self.command_rx.recv() => {
                    trace!(?command, "processing subscription command");
                    match command {
                        SubscriptionCommand::GetModel { resp_tx } => {
                            debug!("getting subscription model");
                            let _ = resp_tx.send(self.model.clone());
                        }
                        SubscriptionCommand::UpdateInfo {
                            mut new_model,
                            resp_tx,
                        } => {
                            debug!(server=?new_model.server, topic=?new_model.topic, "updating subscription info");
                            new_model.server = self.model.server.clone();
                            new_model.topic = self.model.topic.clone();
                            new_model.read_until = self.model.read_until;
                            let res = self.env.db.update_subscription(new_model.clone());
                            if let Ok(_) = res {
                                self.model = new_model;
                            }
                            let _ = resp_tx.send(res.map_err(|e| e.into()));
                        }
                        SubscriptionCommand::Publish {msg, encrypt, resp_tx} => {
                            debug!(topic=?self.model.topic, "publishing message");
                            let _ = resp_tx.send(self.publish(msg, encrypt).await);
                        }
                        SubscriptionCommand::Attach { resp_tx } => {
                            debug!(topic=?self.model.topic, "attaching new listener");
                            let messages = self
                            .env
                                .db
                                .list_messages(&self.model.server, &self.model.topic, 0)
                                .unwrap_or_default();
                            let mut previous_events: Vec<ListenerEvent> = messages
                                .into_iter()
                                .filter_map(|msg| {
                                    let msg = serde_json::from_str(&msg);
                                    match msg {
                                        Err(e) => {
                                            error!(error = ?e, "error parsing stored message");
                                            None
                                        }
                                        Ok(msg) => Some(msg),
                                    }
                                })
                                .map(ListenerEvent::Message)
                                .collect();
                            previous_events.push(ListenerEvent::ConnectionStateChanged(self.listener.state().await));
                            let _ = resp_tx.send((previous_events, self.broadcast_tx.subscribe()));
                        }
                        SubscriptionCommand::ClearNotifications {resp_tx} => {
                            debug!(topic=?self.model.topic, "clearing notifications");
                            let _ = resp_tx.send(self.env.db.delete_messages(&self.model.server, &self.model.topic).map_err(|e| anyhow::anyhow!(e)));
                        }
                        SubscriptionCommand::UpdateReadUntil { timestamp, resp_tx } => {
                            debug!(topic=?self.model.topic, timestamp=timestamp, "updating read until timestamp");
                            let res = self.env.db.update_read_until(&self.model.server, &self.model.topic, timestamp);
                            let _ = resp_tx.send(res.map_err(|e| anyhow::anyhow!(e)));
                        }
                    }
                }
            }
        }
    }

    async fn publish(&self, mut msg: models::OutgoingMessage, encrypt: bool) -> anyhow::Result<()> {
        let server = &self.model.server;
        let topic = &self.model.topic;

        if encrypt {
             let key_str = self.env.keys.get(server, topic).ok_or_else(|| anyhow::anyhow!("Encryption requested but no key found"))?;
             {
                 use aes_gcm::{
                    aead::{Aead, KeyInit},
                    Aes256Gcm, Key, Nonce
                };
                use base64::{engine::general_purpose, Engine as _};
                use rand::Rng;

                let key_bytes = general_purpose::STANDARD.decode(key_str).map_err(|e| anyhow::anyhow!("Invalid key: {}", e))?;
                let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
                let cipher = Aes256Gcm::new(key);
                
                let mut nonce_bytes = [0u8; 12];
                rand::thread_rng().fill(&mut nonce_bytes);
                let nonce = Nonce::from_slice(&nonce_bytes);

                let plaintext = msg.message.as_deref().unwrap_or("").as_bytes();
                let ciphertext = cipher.encrypt(nonce, plaintext)
                    .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;
                
                // Format: version(1) + nonce(12) + ciphertext
                let mut payload = Vec::with_capacity(1 + 12 + ciphertext.len());
                payload.push(1); // Version 1
                payload.extend_from_slice(&nonce_bytes);
                payload.extend_from_slice(&ciphertext);

                let payload_b64 = general_purpose::STANDARD.encode(&payload);
                msg.message = Some(payload_b64);
                // Important: clear title/tags if we want full privacy?
                // But user might want open title with encrypted body.
                // We just encrypt the body as per ntfy spec.
             }
        }

        debug!(server=?server, "preparing to publish message");
        let creds = self.env.credentials.get(server);
        let mut req = self.env.http_client.post(server);
        if let Some(creds) = creds {
            req = req.basic_auth(creds.username, Some(creds.password));
        }

        let body = serde_json::to_string(&msg)?;

        info!(server=?server, "sending message");
        let res = req.body(body).send().await?;
        res.error_for_status()?;
        debug!(server=?server, "message published successfully");
        Ok(())
    }
    fn check_filters(&self, msg: &ReceivedMessage) -> Option<models::FilterAction> {
        let Some(rules) = &self.model.rules else { return None };
        let mut text = msg.display_title().unwrap_or_default();
        text.push_str(" ");
        text.push_str(&msg.display_message().unwrap_or_default());
        
        for rule in rules {
             if let Ok(re) = regex::Regex::new(&rule.regex) {
                 if re.is_match(&text) {
                     return Some(rule.action.clone());
                 }
             }
        }
        None
    }

    fn check_schedule(&self) -> bool {
        // Returns true if notification should be MUTED
        let Some(schedule) = &self.model.schedule else { return false };
        
        use chrono::{Local, Timelike, Datelike};
        let now = Local::now();
        let weekday = now.weekday().num_days_from_sunday() as u8;

        let parse_time = |s: &str| -> Option<(u32, u32)> {
            let parts: Vec<&str> = s.split(':').collect();
            if parts.len() != 2 { return None; }
            Some((parts[0].parse().ok()?, parts[1].parse().ok()?))
        };

        let Some((start_h, start_m)) = parse_time(&schedule.start_time) else { return false };
        let Some((end_h, end_m)) = parse_time(&schedule.end_time) else { return false };

        let now_mins = now.hour() * 60 + now.minute();
        let start_mins = start_h * 60 + start_m;
        let end_mins = end_h * 60 + end_m;

        if start_mins < end_mins {
            // Simple range (e.g. 09:00 - 17:00)
            if schedule.days.contains(&weekday) {
                return now_mins >= start_mins && now_mins < end_mins;
            }
        } else {
            // Crosses midnight (e.g. 22:00 - 07:00)
            // It is quiet if:
            // 1. It's after start_mins AND today is enabled
            // 2. It's before end_mins AND yesterday was enabled
            
            if now_mins >= start_mins && schedule.days.contains(&weekday) {
                return true;
            }
            if now_mins < end_mins {
                let yesterday_weekday = if weekday == 0 { 6 } else { weekday - 1 };
                if schedule.days.contains(&yesterday_weekday) {
                    return true;
                }
            }
        }

        false
    }

    fn handle_msg_event(&mut self, msg: ReceivedMessage) {
        debug!(topic=?self.model.topic, "handling new message");

        // Check for Discard rule BEFORE storage
        let filter_action = self.check_filters(&msg);
        if let Some(models::FilterAction::Discard) = &filter_action {
             debug!(topic=?self.model.topic, "message discarded by filter rule");
             return;
        }

        // Store in database
        let already_stored: bool = {
            let json_ev = &serde_json::to_string(&msg).unwrap();
            match self.env.db.insert_message(&self.model.server, json_ev) {
                Err(Error::DuplicateMessage) => {
                    warn!(topic=?self.model.topic, "received duplicate message");
                    true
                }
                Err(e) => {
                    error!(error=?e, topic=?self.model.topic, "can't store the message");
                    false
                }
                _ => {
                    debug!(topic=?self.model.topic, "message stored successfully");
                    false
                }
            }
        };

        if !already_stored {
            debug!(topic=?self.model.topic, muted=?self.model.muted, "checking if notification should be shown");
            
            let mut muted = self.model.muted;
            
            // Check filters for Mute
            if let Some(models::FilterAction::Mute) = filter_action {
                muted = true;
                debug!("muted by filter");
            }
            if let Some(models::FilterAction::MarkRead) = filter_action {
                muted = true;
                debug!("muted by mark_read filter");
                
                // Update read_until
                if let Err(e) = self.env.db.update_read_until(&self.model.server, &self.model.topic, msg.time) {
                    error!(error=?e, "failed to update read_until for mark_read rule");
                } else {
                    self.model.read_until = msg.time;
                }
            }
            
            // Check Schedule
            if !muted && self.check_schedule() {
                muted = true;
                debug!("muted by schedule");
            }

            // Show notification. If this fails, panic
            if !muted && msg.time > self.model.read_until {
                let notifier = self.env.notifier.clone();

                let title = { msg.notification_title(&self.model) };

                let n = models::Notification {
                    title,
                    body: msg.display_message().as_deref().unwrap_or("").to_string(),
                    actions: msg.actions.clone(),
                };

                info!(topic=?self.model.topic, "showing notification");
                notifier.send(n).unwrap();
            } else {
                debug!(topic=?self.model.topic, "notification muted, skipping");
            }

            // Forward to app
            debug!(topic=?self.model.topic, "forwarding message to app");
            let _ = self.broadcast_tx.send(ListenerEvent::Message(msg));
        }
    }
}
