use poise::serenity_prelude::UserId;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::env;
use std::fs::File as SyncFile;
use std::io::Read as SyncRead;

use crate::constans::{BOT_USER_ID, DISCORD_BOT_CONFIG_PATH, DISCORD_TOKEN, GIFT_CODE_CHANNEL, GIFT_CODE_TEST_CHANNEL, OWNERS, SQLITE_DATABASE_PATH, SUBSCRIPTION_TYPES, UNITY_ENVIRONMENT_ID, UNITY_KEY_ID, UNITY_PROJECT_ID, UNITY_SAVE_DATA_KEY, UNITY_SECRET_KEY};

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    sqlite_database_path: String,
    unity_project_id: String,
    unity_environment_id: String,
    unity_key_id: String,
    unity_secret_key: String,
    unity_save_data_key: String,
    discord_token: String,
    owners: Vec<u64>,
    gift_code_channel: u64,
    gift_code_test_channel: u64,
    bot_user_id: u64,
    subscription_types: Vec<String>,
}

pub fn load_config() {
    let file_path = DISCORD_BOT_CONFIG_PATH;
    let mut file = match SyncFile::open(&file_path) {
        Ok(file) => file,
        Err(e) => {
            panic!("Failed to open file {}: {}", file_path, e);
        }
    };

    let mut contents = String::new();
    if let Err(e) = file.read_to_string(&mut contents) {
        panic!("Failed to read file {}: {}", file_path, e);
    }

    let config: Config = match serde_json::from_str(&contents) {
        Ok(config) => config,
        Err(e) => {
            panic!("Failed to parse JSON: {}", e);
        }
    };

    env::set_var(DISCORD_TOKEN, config.discord_token);
    let owners = config.owners.iter().map(|i| i.to_string()).collect::<Vec<String>>().join(",");
    env::set_var(OWNERS, owners);
    env::set_var(GIFT_CODE_CHANNEL, config.gift_code_channel.to_string());
    env::set_var(GIFT_CODE_TEST_CHANNEL, config.gift_code_test_channel.to_string());
    env::set_var(UNITY_KEY_ID, config.unity_key_id);
    env::set_var(UNITY_SECRET_KEY, config.unity_secret_key);
    env::set_var(UNITY_PROJECT_ID, config.unity_project_id);
    env::set_var(UNITY_ENVIRONMENT_ID, config.unity_environment_id);
    env::set_var(BOT_USER_ID, config.bot_user_id.to_string());
    env::set_var(SQLITE_DATABASE_PATH, config.sqlite_database_path);
    env::set_var(UNITY_SAVE_DATA_KEY, config.unity_save_data_key);
    env::set_var(SUBSCRIPTION_TYPES, config.subscription_types.join(","));
}

pub fn read_owners() -> HashSet<UserId> {
    let owners_str = env::var(OWNERS).expect("OWNERS not set");
    owners_str.split(',')
        .map(|s| UserId::new(s.parse::<u64>().expect("Failed to parse owner ID")))
        .collect()
}

pub fn read_subscription_types() -> HashSet<String> {
    let subscription_types_str = env::var(SUBSCRIPTION_TYPES).expect("SUBSCRIPTION_TYPES not set");
    subscription_types_str.split(',')
        .map(|s| s.to_string())
        .collect()
}