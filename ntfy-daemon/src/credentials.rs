use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use tracing::warn;

#[derive(Clone)]
pub struct KeyringItem {
    pub attributes: HashMap<String, String>,
    // we could zero-out this region of memory
    pub secret: Vec<u8>,
}

impl KeyringItem {
    pub async fn attributes(&self) -> HashMap<String, String> {
        self.attributes.clone()
    }
    pub async fn secret(&self) -> &[u8] {
        &self.secret[..]
    }
}

#[async_trait]
pub trait LightKeyring {
    async fn search_items(
        &self,
        attributes: HashMap<&str, &str>,
    ) -> anyhow::Result<Vec<KeyringItem>>;
    async fn create_item(
        &self,
        label: &str,
        attributes: HashMap<&str, &str>,
        secret: &str,
        replace: bool,
    ) -> anyhow::Result<()>;
    async fn delete(&self, attributes: HashMap<&str, &str>) -> anyhow::Result<()>;
}

pub struct RealKeyring {
    pub(crate) keyring: oo7::Keyring,
}

#[async_trait]
impl LightKeyring for RealKeyring {
    async fn search_items(
        &self,
        attributes: HashMap<&str, &str>,
    ) -> anyhow::Result<Vec<KeyringItem>> {
        let items = self.keyring.search_items(&attributes).await?;

        let mut out_items = vec![];
        for item in items {
            out_items.push(KeyringItem {
                attributes: item.attributes().await?,
                secret: item.secret().await?.to_vec(),
            });
        }
        Ok(out_items)
    }

    async fn create_item(
        &self,
        label: &str,
        attributes: HashMap<&str, &str>,
        secret: &str,
        replace: bool,
    ) -> anyhow::Result<()> {
        self.keyring
            .create_item(label, &attributes, secret, replace)
            .await?;
        Ok(())
    }

    async fn delete(&self, attributes: HashMap<&str, &str>) -> anyhow::Result<()> {
        self.keyring.delete(&attributes).await?;
        Ok(())
    }
}

pub struct NullableKeyring {
    pub(crate) search_response: Vec<KeyringItem>,
}

impl NullableKeyring {
    #[allow(dead_code)]
    pub fn new(search_response: Vec<KeyringItem>) -> Self {
        Self { search_response }
    }
}

/// Wraps an `oo7::dbus::Collection` directly. Used as a fallback when the file
/// backend (driven by `org.freedesktop.portal.Secret`) is unusable in a sandbox
/// — for example when the portal is missing or returns a 0-byte master key —
/// but the host Secret Service is reachable via `--talk-name=org.freedesktop.secrets`.
pub struct DBusKeyring {
    collection: oo7::dbus::Collection,
}

#[async_trait]
impl LightKeyring for DBusKeyring {
    async fn search_items(
        &self,
        attributes: HashMap<&str, &str>,
    ) -> anyhow::Result<Vec<KeyringItem>> {
        let items = self.collection.search_items(&attributes).await?;
        let mut out_items = vec![];
        for item in items {
            out_items.push(KeyringItem {
                attributes: item.attributes().await?,
                secret: item.secret().await?.to_vec(),
            });
        }
        Ok(out_items)
    }

    async fn create_item(
        &self,
        label: &str,
        attributes: HashMap<&str, &str>,
        secret: &str,
        replace: bool,
    ) -> anyhow::Result<()> {
        self.collection
            .create_item(label, &attributes, secret, replace, None)
            .await?;
        Ok(())
    }

    async fn delete(&self, attributes: HashMap<&str, &str>) -> anyhow::Result<()> {
        for item in self.collection.search_items(&attributes).await? {
            item.delete(None).await?;
        }
        Ok(())
    }
}

/// Fallback used when the system Secret Service / Secret portal is unavailable.
/// Refuses writes with a descriptive error so the UI can surface the failure
/// instead of silently losing credentials.
pub struct UnavailableKeyring {
    reason: String,
}

impl UnavailableKeyring {
    pub fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }
}

#[async_trait]
impl LightKeyring for UnavailableKeyring {
    async fn search_items(
        &self,
        _attributes: HashMap<&str, &str>,
    ) -> anyhow::Result<Vec<KeyringItem>> {
        Ok(vec![])
    }

    async fn create_item(
        &self,
        _label: &str,
        _attributes: HashMap<&str, &str>,
        _secret: &str,
        _replace: bool,
    ) -> anyhow::Result<()> {
        anyhow::bail!(
            "System keyring unavailable, cannot save secret: {}",
            self.reason
        )
    }

    async fn delete(&self, _attributes: HashMap<&str, &str>) -> anyhow::Result<()> {
        Ok(())
    }
}

#[async_trait]
impl LightKeyring for NullableKeyring {
    async fn search_items(
        &self,
        _attributes: HashMap<&str, &str>,
    ) -> anyhow::Result<Vec<KeyringItem>> {
        Ok(self.search_response.clone())
    }

    async fn create_item(
        &self,
        _label: &str,
        _attributes: HashMap<&str, &str>,
        _secret: &str,
        _replace: bool,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn delete(&self, _attributes: HashMap<&str, &str>) -> anyhow::Result<()> {
        Ok(())
    }
}
impl NullableKeyring {
    pub fn with_credentials(credentials: Vec<Credential>) -> Self {
        let mut search_response = vec![];

        for cred in credentials {
            let attributes = HashMap::from([
                ("type".to_string(), "password".to_string()),
                ("username".to_string(), cred.username.clone()),
                ("server".to_string(), cred.password.clone()),
            ]);
            search_response.push(KeyringItem {
                attributes,
                secret: cred.password.into_bytes(),
            });
        }

        Self { search_response }
    }
}

/// Try the auto-detected oo7 backend first (file-in-sandbox or DBus on host).
/// If that fails — common when the Flatpak `org.freedesktop.portal.Secret`
/// implementation is missing or returns a 0-byte master key — try the host
/// Secret Service directly via DBus. As a last resort fall back to an
/// `UnavailableKeyring` so the application keeps running.
pub async fn build_keyring(label: &str) -> Arc<dyn LightKeyring + Send + Sync> {
    match oo7::Keyring::new().await {
        Ok(kr) => return Arc::new(RealKeyring { keyring: kr }),
        Err(e) => {
            warn!(
                store = label,
                error = %e,
                "Default keyring backend unavailable, attempting Secret Service fallback"
            );
        }
    }

    match oo7::dbus::Service::new().await {
        Ok(service) => match service.default_collection().await {
            Ok(collection) => {
                return Arc::new(DBusKeyring { collection });
            }
            Err(e) => warn!(store = label, error = %e, "Failed to open default Secret Service collection"),
        },
        Err(e) => warn!(store = label, error = %e, "Secret Service DBus connection failed"),
    }

    warn!(
        store = label,
        "No usable keyring backend; secrets will not be persisted this session"
    );
    Arc::new(UnavailableKeyring::new(
        "no Secret portal or Secret Service available",
    ))
}

#[derive(Debug, Clone)]
pub struct Credential {
    pub username: String,
    pub password: String,
}

#[derive(Clone)]
pub struct Credentials {
    keyring: Arc<dyn LightKeyring + Send + Sync>,
    creds: Arc<RwLock<HashMap<String, Credential>>>,
}

impl Credentials {
    pub async fn new() -> anyhow::Result<Self> {
        let keyring = build_keyring("credentials").await;
        let mut this = Self {
            keyring,
            creds: Default::default(),
        };
        this.load().await?;
        Ok(this)
    }
    pub async fn new_nullable(credentials: Vec<Credential>) -> anyhow::Result<Self> {
        let mut this = Self {
            keyring: Arc::new(NullableKeyring::with_credentials(credentials)),
            creds: Default::default(),
        };
        this.load().await?;
        Ok(this)
    }
    pub async fn load(&mut self) -> anyhow::Result<()> {
        let attrs = HashMap::from([("type", "password")]);
        let values = self.keyring.search_items(attrs).await?;

        let mut lock = self.creds.write().unwrap();
        lock.clear();
        for item in values {
            let attrs = item.attributes().await;
            lock.insert(
                attrs["server"].to_string(),
                Credential {
                    username: attrs["username"].to_string(),
                    password: std::str::from_utf8(&item.secret().await)?.to_string(),
                },
            );
        }
        Ok(())
    }
    pub fn get(&self, server: &str) -> Option<Credential> {
        self.creds.read().unwrap().get(server).cloned()
    }
    pub fn list_all(&self) -> HashMap<String, Credential> {
        self.creds.read().unwrap().clone()
    }
    pub async fn insert(&self, server: &str, username: &str, password: &str) -> anyhow::Result<()> {
        {
            if let Some(cred) = self.creds.read().unwrap().get(server) {
                if cred.username != username {
                    anyhow::bail!("You can add only one account per server");
                }
            }
        }
        let attrs = HashMap::from([
            ("type", "password"),
            ("username", username),
            ("server", server),
        ]);
        self.keyring
            .create_item("Password", attrs, password, true)
            .await?;

        self.creds.write().unwrap().insert(
            server.to_string(),
            Credential {
                username: username.to_string(),
                password: password.to_string(),
            },
        );
        Ok(())
    }
    pub async fn delete(&self, server: &str) -> anyhow::Result<()> {
        let creds = {
            self.creds
                .read()
                .unwrap()
                .get(server)
                .ok_or(anyhow::anyhow!("server creds not found"))?
                .clone()
        };
        let attrs = HashMap::from([
            ("type", "password"),
            ("username", &creds.username),
            ("server", server),
        ]);
        self.keyring.delete(attrs).await?;
        self.creds
            .write()
            .unwrap()
            .remove(server)
            .ok_or(anyhow::anyhow!("server creds not found"))?;
        Ok(())
    }
}
