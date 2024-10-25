use clap::Parser;
use email_address::EmailAddress;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, str::FromStr};

#[derive(Debug, Parser, Serialize)]
pub struct CliConfig {
    /// Port of webserver
    #[arg(long)]
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub port: Option<u16>,

    /// Path to the sqlite database file
    ///
    /// ./config.toml is always used with the lowest priority
    #[arg(long)]
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub database_path: Option<String>,

    /// Additional config files
    #[arg(long)]
    #[serde(skip_serializing)]
    pub config_file: Vec<PathBuf>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    pub port: u16,
    pub database_path: String,
    pub mail_server: MailServerConfig,
    pub mail_message: MailMessageConfig,
}

#[derive(Serialize, Deserialize, Default)]
pub struct MailServerConfig {
    pub smtp_server: String,
    pub smtp_username: String,
    pub smtp_password: String,
}

#[derive(Serialize, Deserialize)]
pub struct MailMessageConfig {
    pub sender_name: String,
    pub sender_address: EmailAddress,
}

impl Default for MailMessageConfig {
    fn default() -> Self {
        Self {
            sender_name: String::new(),
            sender_address: EmailAddress::from_str("example@mail.com").unwrap(),
        }
    }
}
