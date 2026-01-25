use std::collections::HashMap;
use std::sync::{Arc, RwLock};



use crate::credentials::{KeyringItem, LightKeyring, NullableKeyring, RealKeyring};

#[derive(Clone)]
pub struct Keys {
    keyring: Arc<dyn LightKeyring + Send + Sync>,
    // Map<(server, topic), key>
    keys: Arc<RwLock<HashMap<(String, String), String>>>,
}

impl Keys {
    pub async fn new() -> anyhow::Result<Self> {
        let mut this = Self {
            keyring: Arc::new(RealKeyring {
                keyring: oo7::Keyring::new()
                    .await
                    .expect("Failed to start Secret Service"),
            }),
            keys: Default::default(),
        };
        this.load().await?;
        Ok(this)
    }

    pub fn new_nullable(mock_keys: HashMap<(String, String), String>) -> anyhow::Result<Self> {        
        // Convert map to keyring items format for the mock
        let mut search_response = vec![];
        for ((server, topic), key) in &mock_keys {
             let attributes = HashMap::from([
                ("type".to_string(), "topic_key".to_string()),
                ("server".to_string(), server.clone()),
                ("topic".to_string(), topic.clone()),
            ]);
            search_response.push(KeyringItem {
                attributes,
                secret: key.clone().into_bytes(),
            });
        }

        let this = Self {
            keyring: Arc::new(NullableKeyring::new(search_response)),
            keys: Default::default(),
        };
        // Pre-load the memory cache
        *this.keys.write().unwrap() = mock_keys;
        Ok(this)
    }

    pub async fn load(&mut self) -> anyhow::Result<()> {
        let attrs = HashMap::from([("type", "topic_key")]);
        let values = self.keyring.search_items(attrs).await?;

        let mut lock = self.keys.write().unwrap();
        lock.clear();
        for item in values {
        let attrs: HashMap<String, String> = item.attributes().await;
        if let (Some(server), Some(topic)) = (attrs.get("server"), attrs.get("topic")) {
             lock.insert(
                (server.clone(), topic.clone()),
                std::str::from_utf8(item.secret().await)?.to_string(),
            );
        }
        }
        Ok(())
    }

    pub fn get(&self, server: &str, topic: &str) -> Option<String> {
        self.keys.read().unwrap().get(&(server.to_string(), topic.to_string())).cloned()
    }

    pub async fn insert(&self, server: &str, topic: &str, key: &str) -> anyhow::Result<()> {
        let attrs = HashMap::from([
            ("type", "topic_key"),
            ("server", server),
            ("topic", topic),
        ]);
        
        self.keyring
            .create_item("Ntfyr Topic Key", attrs, key, true)
            .await?;

        self.keys.write().unwrap().insert(
            (server.to_string(), topic.to_string()),
            key.to_string(),
        );
        Ok(())
    }

    pub async fn delete(&self, server: &str, topic: &str) -> anyhow::Result<()> {
        let attrs = HashMap::from([
            ("type", "topic_key"),
            ("server", server),
            ("topic", topic),
        ]);
        self.keyring.delete(attrs).await?;
        
        self.keys
            .write()
            .unwrap()
            .remove(&(server.to_string(), topic.to_string()));
            
        Ok(())
    }
}
