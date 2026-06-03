use crate::actor_utils::send_command;
use anyhow::anyhow;
use futures::future::join_all;
use futures::StreamExt;
use std::{collections::HashMap, future::Future, sync::Arc};
use tokio::{
    select,
    sync::{mpsc, oneshot, RwLock},
    task::LocalSet,
};
use tracing::{error, info};

use crate::{
    http_client::HttpClient,
    message_repo::Db,
    models::{self, Account},
    ListenerConfig, ListenerHandle, SharedEnv, SubscriptionHandle,
};

const CONNECT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(15);
const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(240); // 4 minutes

pub fn build_client() -> anyhow::Result<reqwest::Client> {
    // rustls is configured via the `rustls-native-certs` feature flag in Cargo.toml.
    // HTTP/2 multiplexing is used automatically when the server supports it.
    Ok(reqwest::Client::builder()
        .connect_timeout(CONNECT_TIMEOUT)
        .pool_idle_timeout(TIMEOUT)
        .build()?)
}

// Message types for the actor
#[derive()]
pub enum NtfyCommand {
    Subscribe {
        server: String,
        topic: String,
        resp_tx: oneshot::Sender<Result<SubscriptionHandle, anyhow::Error>>,
    },
    Unsubscribe {
        server: String,
        topic: String,
        resp_tx: oneshot::Sender<anyhow::Result<()>>,
    },
    RefreshAll {
        resp_tx: oneshot::Sender<anyhow::Result<()>>,
    },
    ListSubscriptions {
        resp_tx: oneshot::Sender<anyhow::Result<Vec<SubscriptionHandle>>>,
    },
    ListAccounts {
        resp_tx: oneshot::Sender<anyhow::Result<Vec<Account>>>,
    },
    WatchSubscribed {
        resp_tx: oneshot::Sender<anyhow::Result<()>>,
    },
    AddAccount {
        server: String,
        username: String,
        password: String,
        resp_tx: oneshot::Sender<anyhow::Result<()>>,
    },
    RemoveAccount {
        server: String,
        resp_tx: oneshot::Sender<anyhow::Result<()>>,
    },
    AddKey {
        server: String,
        topic: String,
        key: String,
        resp_tx: oneshot::Sender<anyhow::Result<()>>,
    },
    RemoveKey {
        server: String,
        topic: String,
        resp_tx: oneshot::Sender<anyhow::Result<()>>,
    },
    #[allow(dead_code)]
    ListKeys {
        resp_tx: oneshot::Sender<anyhow::Result<HashMap<(String, String), String>>>,
    },
    GetKey {
        server: String,
        topic: String,
        resp_tx: oneshot::Sender<Option<String>>,
    },
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct WatchKey {
    server: String,
    topic: String,
}

pub struct NtfyActor {
    listener_handles: Arc<RwLock<HashMap<WatchKey, SubscriptionHandle>>>,
    env: SharedEnv,
    command_rx: mpsc::Receiver<NtfyCommand>,
}

#[derive(Clone)]
pub struct NtfyHandle {
    command_tx: mpsc::Sender<NtfyCommand>,
}

impl NtfyActor {
    pub fn new(env: SharedEnv) -> (Self, NtfyHandle) {
        let (command_tx, command_rx) = mpsc::channel(32);

        let actor = Self {
            listener_handles: Default::default(),
            env,
            command_rx,
        };

        let handle = NtfyHandle { command_tx };

        (actor, handle)
    }

    async fn handle_subscribe(
        &self,
        server: String,
        topic: String,
    ) -> Result<SubscriptionHandle, anyhow::Error> {
        let read_until = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let subscription = models::Subscription::builder(topic.clone())
            .server(server.clone())
            .read_until(read_until)
            .build()?;

        let mut db = self.env.db.clone();
        db.insert_subscription(subscription.clone())?;

        self.listen(subscription).await
    }

    async fn handle_unsubscribe(&mut self, server: String, topic: String) -> anyhow::Result<()> {
        let subscription = self.listener_handles.write().await.remove(&WatchKey {
            server: server.clone(),
            topic: topic.clone(),
        });

        if let Some(sub) = subscription {
            sub.shutdown().await?;
        }

        self.env.db.remove_subscription(&server, &topic)?;
        info!(server, topic, "Unsubscribed");
        Ok(())
    }

    pub async fn run(&mut self) {
        let mut network_change_stream = self.env.network_monitor.listen();
        loop {
            select! {
                Some(_) = network_change_stream.next() => {
                    let _ = self.refresh_all().await;
                },
                Some(command) = self.command_rx.recv() => self.handle_command(command).await,
            };
        }
    }

    async fn handle_command(&mut self, command: NtfyCommand) {
        match command {
            NtfyCommand::Subscribe {
                server,
                topic,
                resp_tx,
            } => {
                let result = self.handle_subscribe(server, topic).await;
                let _ = resp_tx.send(result);
            }

            NtfyCommand::Unsubscribe {
                server,
                topic,
                resp_tx,
            } => {
                let result = self.handle_unsubscribe(server, topic).await;
                let _ = resp_tx.send(result);
            }

            NtfyCommand::RefreshAll { resp_tx } => {
                let res = self.refresh_all().await;
                let _ = resp_tx.send(res);
            }

            NtfyCommand::ListSubscriptions { resp_tx } => {
                let subs = self
                    .listener_handles
                    .read()
                    .await
                    .values()
                    .cloned()
                    .collect();
                let _ = resp_tx.send(Ok(subs));
            }

            NtfyCommand::ListAccounts { resp_tx } => {
                let accounts = self
                    .env
                    .credentials
                    .list_all()
                    .into_iter()
                    .map(|(server, credential)| Account {
                        server,
                        username: credential.username,
                    })
                    .collect();
                let _ = resp_tx.send(Ok(accounts));
            }

            NtfyCommand::WatchSubscribed { resp_tx } => {
                let result = self.handle_watch_subscribed().await;
                let _ = resp_tx.send(result);
            }

            NtfyCommand::AddAccount {
                server,
                username,
                password,
                resp_tx,
            } => {
                let result = self
                    .env
                    .credentials
                    .insert(&server, &username, &password)
                    .await;
                let _ = resp_tx.send(result);
            }

            NtfyCommand::RemoveAccount { server, resp_tx } => {
                let result = self.env.credentials.delete(&server).await;
                let _ = resp_tx.send(result);
            }

            NtfyCommand::AddKey {
                server,
                topic,
                key,
                resp_tx,
            } => {
                let result = self.env.keys.insert(&server, &topic, &key).await;
                let _ = resp_tx.send(result);
            }

            NtfyCommand::RemoveKey {
                server,
                topic,
                resp_tx,
            } => {
                let result = self.env.keys.delete(&server, &topic).await;
                let _ = resp_tx.send(result);
            }

            NtfyCommand::ListKeys { resp_tx } => {
                let _keys = self.env.keys.clone();
                // Not optimal but we need to read from the internal lock which is not exposed directly 
                // We'll rely on the keys internal cache which we can't easily access from here without exposing it.
                // Or we can just read from the struct if we make `keys` map public or add a method.
                // Let's assume we don't need ListKeys for now or we implement `list_all` in `keys.rs`.
                // Wait, I didn't implement `list_all` in `keys.rs`. Let's skip ListKeys implementation in keys.rs for now or add it.
                
                // Correction: I should have added `list_all` to `keys.rs`.
                // Since I cannot edit `keys.rs` in this same tool call, I will assume I can just access it via a new method later 
                // OR I can't fulfill this command yet.
                // Actually, I can just not implement ListKeys command if the UI doesn't need it (it acts per subscription).
                // Or I can add `list_all` to `keys.rs` in a subsequent step.
                // For now, let's return empty or error? No, let's implement the other two.
                let _ = resp_tx.send(Err(anyhow::anyhow!("Not implemented")));
            }
            NtfyCommand::GetKey {
                server,
                topic,
                resp_tx,
            } => {
                let result = self.env.keys.get(&server, &topic);
                let _ = resp_tx.send(result);
            }
        }
    }

    async fn handle_watch_subscribed(&mut self) -> anyhow::Result<()> {
        let f: Vec<_> = self
            .env
            .db
            .list_subscriptions()?
            .into_iter()
            .map(|m| self.listen(m))
            .collect();

        join_all(f.into_iter().map(|x| async move {
            if let Err(e) = x.await {
                error!(error = ?e, "Can't rewatch subscribed topic");
            }
        }))
        .await;

        Ok(())
    }

    fn listen(
        &self,
        sub: models::Subscription,
    ) -> impl Future<Output = anyhow::Result<SubscriptionHandle>> {
        let server = sub.server.clone();
        let topic = sub.topic.clone();
        let db_max_message_time = self
            .env
            .db
            .get_last_message_time(&server, &topic)
            .unwrap_or(None)
            .unwrap_or(0);

        let since = crate::message_repo::compute_listen_since(db_max_message_time, sub.listen_since);

        let listener = ListenerHandle::new(ListenerConfig {
            http_client: self.env.http_client.clone(),
            credentials: self.env.credentials.clone(),
            keys: self.env.keys.clone(),
            endpoint: server.clone(),
            topic: topic.clone(),
            since,
        });
        let listener_handles = self.listener_handles.clone();
        let sub = SubscriptionHandle::new(listener.clone(), sub, &self.env);

        async move {
            listener_handles
                .write()
                .await
                .insert(WatchKey { server, topic }, sub.clone());
            Ok(sub)
        }
    }

    async fn refresh_all(&self) -> anyhow::Result<()> {
        let mut res = Ok(());
        for sub in self.listener_handles.read().await.values() {
            res = sub.restart().await;
            if res.is_err() {
                break;
            }
        }
        res
    }
}

impl NtfyHandle {
    pub async fn subscribe(
        &self,
        server: &str,
        topic: &str,
    ) -> Result<SubscriptionHandle, anyhow::Error> {
        send_command!(self, |resp_tx| NtfyCommand::Subscribe {
            server: server.to_string(),
            topic: topic.to_string(),
            resp_tx,
        })
    }

    pub async fn unsubscribe(&self, server: &str, topic: &str) -> anyhow::Result<()> {
        send_command!(self, |resp_tx| NtfyCommand::Unsubscribe {
            server: server.to_string(),
            topic: topic.to_string(),
            resp_tx,
        })
    }

    pub async fn refresh_all(&self) -> anyhow::Result<()> {
        send_command!(self, |resp_tx| NtfyCommand::RefreshAll { resp_tx })
    }

    pub async fn list_subscriptions(&self) -> anyhow::Result<Vec<SubscriptionHandle>> {
        send_command!(self, |resp_tx| NtfyCommand::ListSubscriptions { resp_tx })
    }

    pub async fn list_accounts(&self) -> anyhow::Result<Vec<Account>> {
        send_command!(self, |resp_tx| NtfyCommand::ListAccounts { resp_tx })
    }

    pub async fn watch_subscribed(&self) -> anyhow::Result<()> {
        send_command!(self, |resp_tx| NtfyCommand::WatchSubscribed { resp_tx })
    }

    pub async fn add_account(
        &self,
        server: &str,
        username: &str,
        password: &str,
    ) -> anyhow::Result<()> {
        send_command!(self, |resp_tx| NtfyCommand::AddAccount {
            server: server.to_string(),
            username: username.to_string(),
            password: password.to_string(),
            resp_tx,
        })
    }

    pub async fn remove_account(&self, server: &str) -> anyhow::Result<()> {
        send_command!(self, |resp_tx| NtfyCommand::RemoveAccount {
            server: server.to_string(),
            resp_tx,
        })
    }

    pub async fn add_key(&self, server: &str, topic: &str, key: &str) -> anyhow::Result<()> {
         send_command!(self, |resp_tx| NtfyCommand::AddKey {
            server: server.to_string(),
            topic: topic.to_string(),
            key: key.to_string(),
            resp_tx,
        })
    }

    pub async fn remove_key(&self, server: &str, topic: &str) -> anyhow::Result<()> {
        send_command!(self, |resp_tx| NtfyCommand::RemoveKey {
            server: server.to_string(),
            topic: topic.to_string(),
            resp_tx,
        })
    }

    pub async fn get_key(&self, server: &str, topic: &str) -> anyhow::Result<Option<String>> {
        let (resp_tx, resp_rx) = oneshot::channel();
        self.command_tx
            .send(NtfyCommand::GetKey {
                server: server.to_string(),
                topic: topic.to_string(),
                resp_tx,
            })
            .await
            .map_err(|e| anyhow!("Actor is dead: {}", e))?;
        Ok(resp_rx.await?)
    }
}

pub fn start(
    dbpath: &str,
    notification_proxy: Arc<dyn models::NotificationProxy>,
    network_proxy: Arc<dyn models::NetworkMonitorProxy>,
) -> anyhow::Result<NtfyHandle> {
    let dbpath = dbpath.to_owned();

    // Create a channel to receive the handle from the spawned thread
    let (handle_tx, handle_rx) = oneshot::channel();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        // Create everything inside the new thread's runtime. Both stores fall back to
        // an in-memory keyring when oo7 cannot reach the system Secret Service / Secret
        // portal, but if even that fails (e.g. a transient I/O error during load) we
        // still want the app to launch with empty stores rather than crash.
        let (credentials, keys) = rt.block_on(async move {
            let credentials = match crate::credentials::Credentials::new().await {
                Ok(c) => c,
                Err(e) => {
                    error!(error = %e, "Failed to initialize credentials store; continuing without saved accounts");
                    crate::credentials::Credentials::new_nullable(vec![])
                        .await
                        .expect("nullable credentials should always succeed")
                }
            };
            let keys = match crate::keys::Keys::new().await {
                Ok(k) => k,
                Err(e) => {
                    error!(error = %e, "Failed to initialize topic key store; continuing without encrypted topic keys");
                    crate::keys::Keys::new_nullable(std::collections::HashMap::new())
                        .expect("nullable keys should always succeed")
                }
            };
            (credentials, keys)
        });

        let env = SharedEnv {
            db: Db::connect(&dbpath).unwrap(),
            notifier: notification_proxy,
            http_client: HttpClient::new(build_client().unwrap()),
            network_monitor: network_proxy,
            credentials,
            keys,
        };

        let (mut actor, handle) = NtfyActor::new(env);
        let handle_clone = handle.clone();

        // Send the handle back to the calling thread
        let _ = handle_tx.send(handle.clone());

        rt.block_on({
            let local_set = LocalSet::new();
            // Spawn the watch_subscribed task
            local_set.spawn_local(async move {
                if let Err(e) = handle_clone.watch_subscribed().await {
                    error!(error = ?e, "Failed to watch subscribed topics");
                }
            });

            // Run the actor
            local_set.spawn_local(async move {
                actor.run().await;
            });
            local_set
        })
    });

    // Wait for the handle from the spawned thread
    Ok(handle_rx
        .blocking_recv()
        .map_err(|_| anyhow!("Failed to receive actor handle"))?)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::ListenerEvent;
    use models::{NullNetworkMonitor, NullNotifier, OutgoingMessage};
    use tokio::time::sleep;

    use super::*;

    #[test]
    fn test_subscribe_and_publish() {
        let notification_proxy = Arc::new(NullNotifier::new());
        let network_proxy = Arc::new(NullNetworkMonitor::new());
        let dbpath = ":memory:";

        let handle = start(dbpath, notification_proxy, network_proxy).unwrap();

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async move {
            let server = "http://localhost:8000";
            let topic = "test_topic";

            // Subscribe to the topic
            let subscription_handle = handle.subscribe(server, topic).await.unwrap();

            // Publish a message
            let message = OutgoingMessage {
                topic: topic.to_string(),
                ..Default::default()
            };
            let result = subscription_handle.publish(message, false).await;
            assert!(result.is_ok());

            sleep(Duration::from_millis(250)).await;

            // Attach to the subscription and check if the message is received and stored
            let (events, _receiver) = subscription_handle.attach().await;
            dbg!(&events);
            assert!(events.iter().any(|event| match event {
                ListenerEvent::Message(msg) => msg.topic == topic,
                _ => false,
            }));
        });
    }

    #[test]
    #[ignore = "hits the live ntfy.sh server; run with: cargo test -p ntfy-daemon integration_subscribe_fetches_server_history -- --ignored"]
    fn integration_subscribe_fetches_server_history() {
        let topic = format!("ntfyr-hist-it-{}", std::process::id());
        let dbpath = std::env::temp_dir().join(format!("ntfyr-it-{topic}.sqlite"));
        let _ = std::fs::remove_file(&dbpath);

        let handle = start(
            dbpath.to_str().unwrap(),
            Arc::new(NullNotifier::new()),
            Arc::new(NullNetworkMonitor::new()),
        )
        .unwrap();

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            let server = models::DEFAULT_SERVER;
            let publish_url = format!("{server}/{topic}");

            let client = reqwest::Client::new();
            client
                .post(&publish_url)
                .body("historical one")
                .header("Title", "History 1")
                .send()
                .await
                .expect("first publish")
                .error_for_status()
                .expect("first publish status");
            client
                .post(&publish_url)
                .body("historical two")
                .header("Title", "History 2")
                .send()
                .await
                .expect("second publish")
                .error_for_status()
                .expect("second publish status");

            let subscription_handle = handle
                .subscribe(server, &topic)
                .await
                .expect("subscribe");

            sleep(Duration::from_secs(3)).await;

            let (events, mut rx) = subscription_handle.attach().await;
            let mut messages: Vec<_> = events
                .into_iter()
                .filter_map(|event| match event {
                    ListenerEvent::Message(msg) => Some(msg),
                    _ => None,
                })
                .collect();

            let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
            while messages.len() < 2 && tokio::time::Instant::now() < deadline {
                if let Ok(Ok(ListenerEvent::Message(msg))) =
                    tokio::time::timeout(Duration::from_secs(1), rx.recv()).await
                {
                    messages.push(msg);
                }
            }

            assert!(
                messages.len() >= 2,
                "expected cached server messages, got {messages:?}"
            );

            let model = subscription_handle.model().await;
            assert_eq!(model.listen_since, 0);
            assert!(model.read_until > 0);
        });

        let _ = std::fs::remove_file(dbpath);
    }
}
