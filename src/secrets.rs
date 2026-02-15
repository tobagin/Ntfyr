use anyhow::Result;
use oo7::Keyring;
use std::collections::HashMap;

const ATTR_APP_ID: &str = "app_id";
const ATTR_VALUE: &str = "io.github.tobagin.Ntfyr";
const LABEL: &str = "Ntfyr App Lock";

pub async fn store_password(password: &str) -> Result<()> {
    let keyring = Keyring::new().await?;
    
    // Clean up old passwords
    let attributes = HashMap::from([(ATTR_APP_ID, ATTR_VALUE)]);
    let items = keyring.search_items(attributes.clone()).await?;
    
    for item in items {
        item.delete().await?;
    }
    
    keyring.create_item(LABEL, attributes, password, true).await?;
    Ok(())
}

pub async fn get_password() -> Result<Option<String>> {
    let keyring = Keyring::new().await?;
    let attributes = HashMap::from([(ATTR_APP_ID, ATTR_VALUE)]);
    let items = keyring.search_items(attributes).await?;
    
    if let Some(item) = items.first() {
        let secret = item.secret().await?;
        // Secret is Zeroizing<Vec<u8>>, convert to Vec<u8>
        let secret_str = String::from_utf8(secret.to_vec())?;
        Ok(Some(secret_str))
    } else {
        Ok(None)
    }
}

pub async fn has_password() -> Result<bool> {
    let keyring = Keyring::new().await?;
    let attributes = HashMap::from([(ATTR_APP_ID, ATTR_VALUE)]);
    let items = keyring.search_items(attributes).await?;
    Ok(!items.is_empty())
}
