//! Secrets manager for secure credential storage.
//!
//! Provides encrypted storage for API keys, tokens, and other secrets.
//! Secrets can be referenced in tool configurations as `${secrets.KEY_NAME}`.

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;

/// A secret value with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Secret {
    /// The secret value (stored encrypted at rest).
    pub value: String,
    /// When the secret was created.
    pub created_at: String,
    /// When the secret was last updated.
    pub updated_at: String,
    /// Optional description.
    pub description: Option<String>,
}

/// Secrets manager for storing and retrieving secrets.
#[derive(Debug)]
pub struct SecretsManager {
    secrets: RwLock<HashMap<String, Secret>>,
    file_path: Option<String>,
    encryption_key: [u8; 32],
}

impl SecretsManager {
    /// Creates a new secrets manager.
    pub fn new(file_path: Option<String>, master_password: Option<&str>) -> Self {
        // Derive encryption key from password or use default
        let encryption_key = if let Some(password) = master_password {
            let mut hasher = Sha256::new();
            hasher.update(password.as_bytes());
            hasher.update(b"nexus-secrets-salt");
            let result = hasher.finalize();
            let mut key = [0u8; 32];
            key.copy_from_slice(&result);
            key
        } else {
            // Default key (not secure, but works for development)
            let mut key = [0u8; 32];
            key.copy_from_slice(b"nexus-default-key-not-for-prod!!");
            key
        };

        let mut manager = Self {
            secrets: RwLock::new(HashMap::new()),
            file_path,
            encryption_key,
        };

        // Load existing secrets
        manager.load();
        manager
    }

    /// Sets a secret.
    pub fn set(&self, key: &str, value: &str, description: Option<&str>) {
        let now = chrono::Utc::now().to_rfc3339();
        let secret = Secret {
            value: self.encrypt(value),
            created_at: now.clone(),
            updated_at: now,
            description: description.map(|s| s.to_string()),
        };

        self.secrets.write().insert(key.to_string(), secret);
        self.save();
    }

    /// Gets a secret value.
    pub fn get(&self, key: &str) -> Option<String> {
        self.secrets
            .read()
            .get(key)
            .map(|s| self.decrypt(&s.value))
    }

    /// Gets secret metadata (without the value).
    pub fn get_metadata(&self, key: &str) -> Option<(String, String, Option<String>)> {
        self.secrets.read().get(key).map(|s| {
            (
                s.created_at.clone(),
                s.updated_at.clone(),
                s.description.clone(),
            )
        })
    }

    /// Deletes a secret.
    pub fn delete(&self, key: &str) -> bool {
        let removed = self.secrets.write().remove(key).is_some();
        if removed {
            self.save();
        }
        removed
    }

    /// Lists all secret keys (not values).
    pub fn list(&self) -> Vec<String> {
        self.secrets.read().keys().cloned().collect()
    }

    /// Checks if a secret exists.
    pub fn exists(&self, key: &str) -> bool {
        self.secrets.read().contains_key(key)
    }

    /// Substitutes secret references in a string.
    /// Replaces `${secrets.KEY}` with the actual secret value.
    pub fn substitute(&self, text: &str) -> String {
        let mut result = text.to_string();
        let secrets = self.secrets.read();

        for (key, secret) in secrets.iter() {
            let placeholder = format!("${{secrets.{}}}", key);
            if result.contains(&placeholder) {
                result = result.replace(&placeholder, &self.decrypt(&secret.value));
            }
        }

        result
    }

    /// Simple XOR encryption (for demo - use proper encryption in production).
    fn encrypt(&self, plaintext: &str) -> String {
        let encrypted: Vec<u8> = plaintext
            .bytes()
            .enumerate()
            .map(|(i, b)| b ^ self.encryption_key[i % 32])
            .collect();
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &encrypted)
    }

    /// Simple XOR decryption.
    fn decrypt(&self, ciphertext: &str) -> String {
        let decoded = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, ciphertext)
            .unwrap_or_default();
        let decrypted: Vec<u8> = decoded
            .iter()
            .enumerate()
            .map(|(i, b)| b ^ self.encryption_key[i % 32])
            .collect();
        String::from_utf8(decrypted).unwrap_or_default()
    }

    /// Saves secrets to file.
    fn save(&self) {
        if let Some(ref path) = self.file_path {
            let secrets = self.secrets.read();
            if let Ok(json) = serde_json::to_string_pretty(&*secrets) {
                let _ = std::fs::write(path, json);
            }
        }
    }

    /// Loads secrets from file.
    fn load(&mut self) {
        if let Some(ref path) = self.file_path {
            if Path::new(path).exists() {
                if let Ok(content) = std::fs::read_to_string(path) {
                    if let Ok(secrets) = serde_json::from_str(&content) {
                        *self.secrets.write() = secrets;
                    }
                }
            }
        }
    }
}

impl Default for SecretsManager {
    fn default() -> Self {
        Self::new(None, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_get_secret() {
        let manager = SecretsManager::new(None, Some("test-password"));
        manager.set("API_KEY", "sk-12345", Some("OpenAI key"));

        let value = manager.get("API_KEY");
        assert_eq!(value, Some("sk-12345".to_string()));
    }

    #[test]
    fn test_substitute() {
        let manager = SecretsManager::new(None, None);
        manager.set("TOKEN", "abc123", None);

        let result = manager.substitute("Bearer ${secrets.TOKEN}");
        assert_eq!(result, "Bearer abc123");
    }

    #[test]
    fn test_list() {
        let manager = SecretsManager::new(None, None);
        manager.set("KEY1", "value1", None);
        manager.set("KEY2", "value2", None);

        let keys = manager.list();
        assert_eq!(keys.len(), 2);
    }
}

