use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokens {
    pub access_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
}

impl OAuthTokens {
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() >= expires_at
        } else {
            false
        }
    }

    pub fn needs_refresh(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            let buffer = chrono::Duration::minutes(5);
            Utc::now() + buffer >= expires_at
        } else {
            false
        }
    }
}

pub struct AuthStore {
    base_path: PathBuf,
}

impl AuthStore {
    pub fn new() -> Result<Self> {
        let base_path = dirs::home_dir()
            .context("Cannot find home directory")?
            .join(".dynamic-mcp")
            .join("oauth-servers");

        Ok(Self { base_path })
    }

    pub async fn save_token(&self, server_name: &str, tokens: &OAuthTokens) -> Result<()> {
        fs::create_dir_all(&self.base_path)
            .await
            .context("Failed to create auth directory")?;

        let path = self.base_path.join(format!("{}.json", server_name));
        let json = serde_json::to_string_pretty(tokens)?;

        fs::write(&path, json)
            .await
            .with_context(|| format!("Failed to save token for {}", server_name))?;

        tracing::debug!("Saved OAuth token for {} to {:?}", server_name, path);
        Ok(())
    }

    pub async fn load_token(&self, server_name: &str) -> Result<Option<OAuthTokens>> {
        let path = self.base_path.join(format!("{}.json", server_name));

        if !tokio::fs::try_exists(&path).await? {
            return Ok(None);
        }

        let json = fs::read_to_string(&path)
            .await
            .with_context(|| format!("Failed to read token for {}", server_name))?;

        let tokens: OAuthTokens = serde_json::from_str(&json)
            .with_context(|| format!("Failed to parse token for {}", server_name))?;

        tracing::debug!("Loaded OAuth token for {} from {:?}", server_name, path);
        Ok(Some(tokens))
    }

    pub async fn delete_token(&self, server_name: &str) -> Result<()> {
        let path = self.base_path.join(format!("{}.json", server_name));

        if tokio::fs::try_exists(&path).await? {
            fs::remove_file(&path)
                .await
                .with_context(|| format!("Failed to delete token for {}", server_name))?;

            tracing::debug!("Deleted OAuth token for {} from {:?}", server_name, path);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_store() -> (AuthStore, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let store = AuthStore {
            base_path: temp_dir.path().to_path_buf(),
        };
        (store, temp_dir)
    }

    #[tokio::test]
    async fn test_save_and_load_token() {
        let (store, _temp) = create_test_store();

        let tokens = OAuthTokens {
            access_token: "test_access_token".to_string(),
            refresh_token: Some("test_refresh_token".to_string()),
            expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
        };

        store.save_token("test_server", &tokens).await.unwrap();

        let loaded = store.load_token("test_server").await.unwrap();
        assert!(loaded.is_some());

        let loaded_tokens = loaded.unwrap();
        assert_eq!(loaded_tokens.access_token, "test_access_token");
        assert_eq!(
            loaded_tokens.refresh_token,
            Some("test_refresh_token".to_string())
        );
    }

    #[tokio::test]
    async fn test_load_nonexistent_token() {
        let (store, _temp) = create_test_store();

        let loaded = store.load_token("nonexistent").await.unwrap();
        assert!(loaded.is_none());
    }

    #[tokio::test]
    async fn test_delete_token() {
        let (store, _temp) = create_test_store();

        let tokens = OAuthTokens {
            access_token: "test_token".to_string(),
            refresh_token: None,
            expires_at: None,
        };

        store.save_token("test_server", &tokens).await.unwrap();
        store.delete_token("test_server").await.unwrap();

        let loaded = store.load_token("test_server").await.unwrap();
        assert!(loaded.is_none());
    }

    #[tokio::test]
    async fn test_token_expiry_check() {
        let expired = OAuthTokens {
            access_token: "token".to_string(),
            refresh_token: None,
            expires_at: Some(Utc::now() - chrono::Duration::hours(1)),
        };
        assert!(expired.is_expired());

        let valid = OAuthTokens {
            access_token: "token".to_string(),
            refresh_token: None,
            expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
        };
        assert!(!valid.is_expired());

        let no_expiry = OAuthTokens {
            access_token: "token".to_string(),
            refresh_token: None,
            expires_at: None,
        };
        assert!(!no_expiry.is_expired());
    }

    #[tokio::test]
    async fn test_needs_refresh() {
        let needs_refresh = OAuthTokens {
            access_token: "token".to_string(),
            refresh_token: Some("refresh".to_string()),
            expires_at: Some(Utc::now() + chrono::Duration::minutes(3)),
        };
        assert!(needs_refresh.needs_refresh());

        let no_refresh_needed = OAuthTokens {
            access_token: "token".to_string(),
            refresh_token: Some("refresh".to_string()),
            expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
        };
        assert!(!no_refresh_needed.needs_refresh());
    }
}
