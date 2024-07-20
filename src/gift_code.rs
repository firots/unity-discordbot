use poise::serenity_prelude::CreateEmbed;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use anyhow::anyhow;
use serde_json::Value as JsonValue;
use crate::{models::{GiftCode, GiftCodeReward}, Error};
use chrono::{DateTime, Utc, Duration};

pub fn is_valid_gift_code_reward(value: &JsonValue) -> bool {
    let result: Result<GiftCodeReward, _> = serde_json::from_value(value.clone());
    result.is_ok()
}

pub fn is_valid_gift_code(code: &str) -> bool {
    code.len() == 16 && code.chars().all(|c| c.is_ascii_uppercase() || c.is_digit(10))
}

pub fn generate_gift_code() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .filter(|c| !c.is_ascii_lowercase() && c != &'O' && c != &'0')
        .take(16)
        .collect()
}

pub fn add_days_to_current_date(days: i64) -> String {
    let now = Utc::now();
    let future_date = now + Duration::days(days);
    future_date.to_rfc3339()
}

pub fn validate_gift_code(
    title: &str,
    subtitle: &str,
    amount: u32,
    duration: u32,
    rewards: &JsonValue,
) -> Result<(), Error> {
    if title.is_empty() {
        Err(anyhow!("Title cannot be empty").into())
    } else if subtitle.is_empty() {
        Err(anyhow!("Subtitle cannot be empty").into())
    } else if amount <= 0 {
        Err(anyhow!("Amount cannot be 0").into())
    } else if duration <= 0 {
        Err(anyhow!("Duration cannot be 0").into())
    } else if rewards.is_null() || rewards.as_object().map_or(true, |o| o.is_empty()) {
        Err(anyhow!("Rewards cannot be empty").into())
    } else if is_valid_gift_code_reward(rewards) == false {
        Err(anyhow!("Invalid gift code reward").into())
    } else {
        Ok(())
    }
}

pub fn get_gift_code_embed(gift_code: &GiftCode) -> CreateEmbed {
    let mut embed = CreateEmbed::default()
        .title(gift_code.title.clone())
        .description(gift_code.subtitle.clone());

    let parsed_date = match DateTime::parse_from_rfc3339(&gift_code.expired_at) {
        Ok(parsed_date) => parsed_date.with_timezone(&Utc),
        Err(_) => Utc::now(),
    };

    embed = embed.field("➖➖➖➖➖", "", false);

    for currency in &gift_code.rewards.currency_rewards {
        let currency_field = format!("**{}**", currency.name);
        embed = embed.field(currency_field, "", false);
    }

    if gift_code.rewards.xp_reward > 0 {
        let formatted_xp = format!("{:.3}", gift_code.rewards.xp_reward as f32 / 1000.0);
        let xp_field = format!("**<:xp:1250546574518521916> x {}**", formatted_xp);
        embed = embed.field(xp_field, "", false);
    }

    for item in &gift_code.rewards.item_rewards {
        embed = embed.field(item.name.clone(), "", false);
    }

    embed = embed.field("➖➖➖➖➖", "", false);

    embed = embed.field("Remaining Gift Codes", format!("{}", gift_code.amount), false);

    let duration_since_now = parsed_date - Utc::now();
    if duration_since_now.num_hours() > 0 && duration_since_now.num_days() < 180 {
        let friendly_date = parsed_date.format("%B %d, %Y").to_string();
        embed = embed.field("Expiration", friendly_date, false);
    }

    embed
}


pub fn get_gift_code_message(gift_code: &GiftCode) -> String {
    let mut message = format!("**{}**\n{}\n", gift_code.title, gift_code.subtitle);


    message.push_str("\n");

    let parsed_date = match DateTime::parse_from_rfc3339(&gift_code.expired_at) {
        Ok(parsed_date) => {
            parsed_date.with_timezone(&Utc)
        }
        Err(_) => Utc::now()
    };

    let currency_rewards = &gift_code.rewards.currency_rewards;
    for (index, currency) in currency_rewards.iter().enumerate() {
        message.push_str(&format!("**{}**", currency.name));
        if index < currency_rewards.len() - 1 {
            message.push_str("     ");
        }
    }

    if gift_code.rewards.xp_reward > 0 {
        if currency_rewards.len() > 0 {
            message.push_str("     ");
        }
        let formatted_xp = format!("{:.3}", gift_code.rewards.xp_reward as f32 / 1000.0);
        message.push_str(&format!("**<:xp:1250546574518521916> x {}**", formatted_xp));
    }

    message.push_str("\n");

    for item in &gift_code.rewards.item_rewards {
        message.push_str(&format!("{}\n", item.name));
    }

    message.push_str("\n");

    message.push_str(&format!("Remaining Gift Codes: {}               ", gift_code.amount));

    let duration_since_now = parsed_date - Utc::now();
    if duration_since_now.num_hours() > 0 && duration_since_now.num_days() < 180 {
        let friendly_date = parsed_date.format("%B %d, %Y").to_string();
        message.push_str(&format!("Expiration: {}\n", friendly_date));
    }

    message.push_str("\n");
    message
}