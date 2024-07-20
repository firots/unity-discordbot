use std::sync::Arc;
use unity_service::UnityService;

pub struct ContextData {
    pub bot: Arc<bot::Bot>,
    pub unity_service: Arc<UnityService>
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, ContextData, Error>;

pub mod unity_service;
pub mod gift_code;
pub mod commands;
pub mod bot;
pub mod db;
pub mod config;
pub mod constans;
pub mod models;