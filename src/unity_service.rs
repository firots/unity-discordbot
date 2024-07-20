use std::env;
use base64::encode;
use anyhow::anyhow;
use chrono::{Duration, Utc};
use serde_json::{from_str, Value};
use crate::constans::{UNITY_ENVIRONMENT_ID, UNITY_KEY_ID, UNITY_PROJECT_ID, UNITY_SAVE_DATA_KEY, UNITY_SECRET_KEY};
use crate::models::{GamePlatform, GameVersion, GetAllGiftCodesResponse, GiftCode, SaveRequest};
use crate::Error;

pub struct UnityService {
    client: reqwest::Client,
    custom_url: String,
    players_url: String,
    auth_header: String,
    save_data_key: String,
}

impl UnityService {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            client: reqwest::Client::new(),
            custom_url: format!("{}/custom", Self::initialize_url()?),
            players_url: format!("{}/players", Self::initialize_url()?),
            auth_header: UnityService::initialize_auth_header()?,
            save_data_key: env::var(UNITY_SAVE_DATA_KEY)?,
        })
    }
}

impl UnityService {
    fn initialize_url() -> Result<String, Error> {
        let project_id = env::var(UNITY_PROJECT_ID)?;
        let environment_id = env::var(UNITY_ENVIRONMENT_ID)?;

        Ok(format!(
            "https://services.api.unity.com/cloud-save/v1/data/projects/{}/environments/{}",
            project_id, environment_id
        ))
    }

    fn initialize_auth_header() -> Result<String, Error> {
        let key_id = env::var(UNITY_KEY_ID)?;
        let secret_key = env::var(UNITY_SECRET_KEY)?;
        let credentials = format!("{}:{}", key_id, secret_key);
        let encoded_credentials = encode(credentials);
        Ok(format!("Basic {}", encoded_credentials))
    }
    
    pub async fn get_gift_code(&self, gift_code_key: String) -> Result<GiftCode, Error> {
        let get_url = format!("{}/gift_codes/items?keys={}", self.custom_url, gift_code_key);
        let response = self.client.get(&get_url)
            .header("Authorization", &self.auth_header)
            .send()
            .await?;
    
        if response.status().is_success() {
            let text = response.text().await?;
            let gift_codes: GetAllGiftCodesResponse = serde_json::from_str(&text)?;
            if gift_codes.results.is_empty() {
                return Err(anyhow!("Gift code not found").into());
            } else {
                return Ok(gift_codes.results[0].value.clone());
            }
        } else {
            let text = response.text().await?;
            Err(anyhow!("Failed to get gift code: {}", text).into())
        }
    }
    
    pub async fn get_all_gift_codes(&self) -> Result<GetAllGiftCodesResponse, Error> {
        let get_url = format!("{}/gift_codes/items", self.custom_url);
    
        let response = self.client.get(&get_url)
            .header("Authorization", &self.auth_header)
            .send()
            .await?;
    
        if response.status().is_success() {
            let gift_codes: GetAllGiftCodesResponse = response.json().await?;
            Ok(gift_codes)
        } else {
            let text = response.text().await?;
            Err(anyhow!("Failed to get gift codes: {}", text).into())
        }
    }
    
    pub async fn get_gift_code_count(&self) -> Result<u32, Error> {
        let gift_codes = self.get_all_gift_codes().await?;
        let count = gift_codes.results.len();
        Ok(count as u32)
    }
    
    pub async fn save_gift_code(&self, gift_code_key: &String, gift_code: &GiftCode) -> Result<(), Error> {
        let save_url = format!("{}/gift_codes/items", self.custom_url);
    
        let serialized_data = serde_json::to_string(gift_code)?;
        let request_body = SaveRequest {
            key: gift_code_key.to_string(),
            value: serialized_data,
        };
    
        let response = self.client.post(&save_url)
            .header("Authorization", &self.auth_header)
            .json(&request_body)
            .send()
            .await?;
    
        if response.status().is_success() {
            println!("Gift code saved successfully.");
        } else {
            let text = response.text().await?;
            return Err(anyhow!("Failed to save gift code: {}", text).into());
        }
    
        Ok(())
    }
    
    pub async fn update_game_version(&self, game_version: &GameVersion, platform: GamePlatform) -> Result<(), Error> {
        let update_url = format!("{}/game_version/items", self.custom_url);
    
        let serialized_data = serde_json::to_string(game_version)?;
        let request_body = SaveRequest {
            key: platform.to_string(),
            value: serialized_data,
        };
    
        let response = self.client.post(&update_url)
            .header("Authorization", &self.auth_header)
            .json(&request_body)
            .send()
            .await?;
    
        if response.status().is_success() {
            println!("Game version updated successfully for platform: {}", platform);
        } else {
            let text = response.text().await?;
            return Err(anyhow!("Failed to update game version: {}", text).into());
        }
    
        Ok(())
    }
    
    pub async fn delete_gift_code(&self, gift_code_key: &str) -> Result<(), Error> {
        let delete_url = format!("{}/gift_codes/items/{}", self.custom_url, gift_code_key);
    
        let response = self.client.delete(&delete_url)
            .header("Authorization", &self.auth_header)
            .send()
            .await?;
    
        if response.status().is_success() {
            println!("Gift code deleted successfully.");
        } else {
            let text = response.text().await?;
            return Err(anyhow!("Failed to delete gift code: {}", text).into());
        }
        Ok(())
    }

    pub async fn get_player_items(&self, player_id: &str, key: String) -> Result<Value, Error> {
        let get_url = format!("{}/{}/items", self.players_url, player_id);

        let mut params = vec![];
        params.push(("keys", key));

        let response = self.client.get(&get_url)
            .header("Authorization", &self.auth_header)
            .query(&params)
            .send()
            .await?;
    
            if response.status().is_success() {
                let json: Value = response.json().await?;
                Ok(json)
            } else {
                let text = response.text().await?;
                Err(anyhow!("Failed to get player items, request_url: {}, error: {}", get_url, text).into())
            }
    }

    pub async fn set_player_item(&self, player_id: &str, key: String, value: Value) -> Result<(), Error> {
        let save_url = format!("{}/{}/items", self.players_url, player_id);

        let request_body = SaveRequest {
            key,
            value: value.to_string(),
        };

        let response = self.client.post(&save_url)
            .header("Authorization", &self.auth_header)
            .json(&request_body)
            .send()
            .await?;

        if response.status().is_success() {
            println!("Player item saved successfully.");
        } else {
            let text = response.text().await?;
            return Err(anyhow!("Failed to save player item: {}", text).into());
        }

        Ok(())
    }

    pub async fn get_save_data(&self, player_id: &str) -> Result<Value, Error> {
        let player_items = self.get_player_items(player_id, self.save_data_key.clone()).await?;
        let results_array = player_items.get("results").and_then(|v| v.as_array()).ok_or_else(|| anyhow!("Results array not found"))?;
        let first_result = results_array.get(0).ok_or_else(|| anyhow!("No first result"))?;
        let value_str = first_result.get("value").and_then(|v| v.as_str()).ok_or_else(|| anyhow!("Value field not found or not a string"))?;
        let save_data: Value = from_str(value_str)?;
        Ok(save_data)
    }

    pub async fn set_save_data(&self, player_id: &str, save_data: Value) -> Result<(), Error> {
        self.set_player_item(player_id, self.save_data_key.clone(), save_data).await?;
        Ok(())
    }
    
    pub async fn update_subscription_data(&self, player_id: &str, product_id: &str, duration: i32, increase_save_count_by: u64) -> Result<(), Error> {
        if increase_save_count_by < 1 {
            return Err(anyhow!("increase_save_count_by must be greater than 0").into());
        }

        let mut save_data = self.get_save_data(player_id).await?;
        let expires_at = Utc::now() + Duration::days(duration as i64);

        let player_account_data = save_data
            .get_mut("playerAccountData")
            .and_then(|v| v.as_object_mut())
            .ok_or_else(|| anyhow!("'playerAccountData' not found or null"))?;

        let shop_data = player_account_data
            .get_mut("shopData")
            .and_then(|v| v.as_object_mut())
            .ok_or_else(|| anyhow!("'shopData' not found or null"))?;

        let shop_subscription_data = shop_data
            .get_mut("shopSubscriptionData")
            .and_then(|v| v.as_object_mut())
            .ok_or_else(|| anyhow!("'shopSubscriptionData' not found or null"))?;

        let product = shop_subscription_data
            .entry(product_id.to_string())
            .or_insert_with(|| Value::Object(serde_json::Map::new()))
            .as_object_mut().unwrap();

        product.insert("expiresAt".to_string(), Value::String(expires_at.to_rfc3339()));

        let player_progress_data = save_data
            .get_mut("playerProgressData")
            .and_then(|v| v.as_object_mut())
            .ok_or_else(|| anyhow!("'playerProgressData' not found or null"))?;
    
        let save_count = player_progress_data
            .get_mut("saveCount")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow!("Unable to get or cast 'saveCount' as mutable number"))?;
    
        player_progress_data.insert("saveCount".to_string(), Value::Number((save_count + increase_save_count_by).into()));

        self.set_save_data(player_id, save_data).await?;

        Ok(())
    }
}

