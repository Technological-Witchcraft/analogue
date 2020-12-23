#![allow(dead_code)]
use reqwest;
use reqwest::Response;
use std::future::Future;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Serialize, Deserialize, Debug)]
pub struct DataOption {
	pub name: String,
	pub value: Option<serde_json::Value>,
	pub options: Option<Vec<DataOption>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InteractionData {
	pub id: String,
	pub name: String,
	pub options: Option<Vec<DataOption>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InteractionUser {
	pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InteractionGuildMember {
	pub user: InteractionUser,
}

#[derive(Serialize_repr, Deserialize_repr, Debug)]
#[repr(u8)]
pub enum InteractionType {
	Ping = 1,
	ApplicationCommand = 2,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Interaction {
	pub id: String,
	pub r#type: InteractionType,
	pub data: InteractionData,
	pub guild_id: String,
	pub channel_id: String,
	pub member: InteractionGuildMember,
	pub token: String,
	pub version: usize,
}

pub mod response {
	use serde::{Deserialize, Serialize};
	use serde_repr::{Deserialize_repr, Serialize_repr};

	use std::collections::HashMap;

	#[derive(Serialize_repr, Deserialize_repr, Debug)]
	#[repr(u8)]
	pub enum ResponseType {
		Pong = 1,
		Acknowledge = 2,
		ChannelMessage = 3,
		ChannelMessageWithSource = 4,
		ACKWithSource = 5,
	}

	#[derive(Default, Serialize, Deserialize, Debug)]
	pub struct AllowedMentions {
		pub parse: Vec<String>,
		pub roles: Vec<String>,
		pub users: Vec<String>,
		pub replied_user: bool,
	}

	impl AllowedMentions {
		pub fn everyone() -> AllowedMentions {
			AllowedMentions {
				parse: vec!["everyone".to_string()],
				..Default::default()
			}
		}

		pub fn everyone_with_reply() -> AllowedMentions {
			AllowedMentions {
				parse: vec!["everyone".to_string()],
				replied_user: true,
				..Default::default()
			}
		}

		pub fn reply() -> AllowedMentions {
			AllowedMentions {
				replied_user: true,
				..Default::default()
			}
		}

		pub fn none() -> AllowedMentions {
			Default::default()
		}
	}

	#[derive(Serialize, Deserialize, Debug)]
	pub struct ResponseData {
		pub tts: bool,
		pub content: String,
		pub allowed_mentions: AllowedMentions,
		pub embeds: Vec<serde_json::Map<String, serde_json::Value>>,
	}

	#[derive(Serialize, Deserialize, Debug)]
	pub struct Response {
		pub r#type: ResponseType,
		pub data: Option<ResponseData>,
	}
}

pub mod definition {
	use serde::{Deserialize, Serialize};

	#[derive(Serialize, Deserialize, Debug)]
	pub struct OptionChoice {
		pub name: String,
		pub value: serde_json::Value,
	}

	#[derive(Serialize, Deserialize, Debug)]
	pub struct CommandOption {
		pub r#type: usize,
		pub name: String,
		pub description: String,
		pub default: Option<bool>,
		pub required: bool,
		pub choices: Option<Vec<OptionChoice>>,
		pub options: Option<Vec<CommandOption>>,
	}

	#[derive(Serialize, Deserialize, Debug)]
	pub struct Command {
		pub name: String,
		pub description: String,
		pub options: Vec<CommandOption>,
	}
}

pub fn build_guild_endpoint(app_id: &str, guild_id: &str) -> String {
	format!(
		"https://discord.com/api/v8/applications/{}/guilds/{}/commands",
		app_id, guild_id
	)
}

pub fn build_response_endpoint(interaction_id: &str, interaction_token: &str) -> String {
	format!(
		"https://discord.com/api/v8/interactions/{}/{}/callback",
		interaction_id, interaction_token
	)
}

pub async fn construct_interactions(
	commands: Vec<definition::Command>,
	application_id: &str,
	guild_id: &str,
	bot_token: &str,
) {
	let endpoint = build_guild_endpoint(application_id, guild_id);
	let client = reqwest::Client::new();

	for command in commands {
		client
			.post(&endpoint)
			.header("Authorization", format!("Bot {}", bot_token))
			.header("content-type", "application/json")
			.body(serde_json::to_string(&command).unwrap())
			.send()
			.await
			.unwrap();
	}
}

pub fn send_interaction_response(
	response: response::Response,
	interaction_id: &str,
	interaction_token: &str,
	bot_token: &str,
) -> impl Future<Output = Result<Response, reqwest::Error>> {
	let client = reqwest::Client::new();

	client
		.post(&build_response_endpoint(interaction_id, interaction_token))
		.header("Authorization", format!("Bot {}", bot_token))
		.header("content-type", "application/json")
		.body(serde_json::to_string(&response).unwrap())
		.send()
}
