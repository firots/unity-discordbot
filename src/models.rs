use std::{fmt, str::FromStr};
use crate::Error;
use serde::{de::Error as SerdeError, Deserialize, Deserializer, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GiftCode {
    pub title: String,
    pub subtitle: String,
    pub amount: u32,
    pub duration: u32,
    pub expired_at: String,
    pub rewards: GiftCodeReward,
    pub channel_id: u64,
    pub message_id: String,
    pub button_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyReward {
    pub name: String,
    pub currency_type: u32,
    pub currency_amount: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemReward {
    pub name: String,
    item_id: u32,
    item_grade: u32,
    upgrade_level: u32,
    item_refinement_quality: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GiftCodeReward {
    pub currency_rewards: Vec<CurrencyReward>,
    pub item_rewards: Vec<ItemReward>,
    pub xp_reward: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveRequest {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GiftCodeResponse {
    pub key: String,
    #[serde(deserialize_with = "string_to_gift_code")]
    pub value: GiftCode,
}

fn string_to_gift_code<'de, D>(deserializer: D) -> Result<GiftCode, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    serde_json::from_str(&s).map_err(SerdeError::custom)
}

#[allow(non_camel_case_types)]
pub enum GamePlatform {
    iOS,
    Android,
}

impl fmt::Display for GamePlatform {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GamePlatform::iOS => write!(f, "iOS"),
            GamePlatform::Android => write!(f, "Android"),
        }
    }
}

impl FromStr for GamePlatform {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ios" => Ok(GamePlatform::iOS),
            "android" => Ok(GamePlatform::Android),
            _ => Err(format!("Invalid platform name: {}", s).into()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAllGiftCodesResponse {
    pub results: Vec<GiftCodeResponse>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GameVersion {
    pub version_number: String,
    pub force_update: bool,
}
