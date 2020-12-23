use crate::{advent, interactions, ADVENT_OF_CODE, CONFIG};
use ::chrono::{DateTime, Datelike, NaiveDateTime, Utc};
use ::reqwest::Client as HTTPClient;
use ::serenity::{client::Context, model::id::RoleId};
use serenity::builder::CreateEmbed;

pub async fn advent_interaction(
	_: &Context,
	_: &interactions::Interaction,
) -> interactions::response::Response {
	let mut advent_of_code = unsafe { &mut ADVENT_OF_CODE };
	let now = Utc::now();

	if now.month() == 12 {
		if advent_of_code.1 < now.timestamp() as usize {
			if let Some(config) = unsafe { &CONFIG } {
				let client = HTTPClient::new();
				let request = client
					.get(
						format!(
							"https://adventofcode.com/{}/leaderboard/private/view/635430.json",
							now.year()
						)
						.as_str(),
					)
					.header("Cookie", format!("session={}", config.advent_session()))
					.build()
					.unwrap();
				let body = client.execute(request).await.unwrap().text().await.unwrap();
				advent_of_code.0 = body;
				advent_of_code.1 = now.timestamp() as usize + 900;
			}
		}
		let leaderboard: advent::Leaderboard =
			::serde_json::from_str(&advent_of_code.0.clone()).unwrap();
		let mut ranking = leaderboard.members();
		ranking.sort_unstable();

		let mut e = CreateEmbed::default();
		e.color(0x722f37);
		let mut desc = String::new();
		for i in 0..ranking.len() {
			let idx = (ranking.len() - 1) - i;
			desc += format!(
				"{}. {} {}({} stars)\n",
				i + 1,
				ranking[idx].score(),
				ranking[idx].name(),
				ranking[idx].stars()
			)
			.as_str();
		}
		e.description(desc);
		let naive = NaiveDateTime::from_timestamp(advent_of_code.1 as i64, 0);
		e.timestamp(DateTime::<Utc>::from_utc(naive, Utc).to_rfc3339());
		e.title("Advent of Code Leaderboard");

		interactions::response::Response {
			r#type: interactions::response::ResponseType::ChannelMessageWithSource,
			data: Some(interactions::response::ResponseData {
				tts: false,
				content: "".to_string(),
				embeds: vec![serenity::utils::hashmap_to_json_map(e.0.clone())],
				allowed_mentions: interactions::response::AllowedMentions::none(),
			}),
		}
	} else {
		let mut embed = CreateEmbed::default();
		let embed = embed
			.color(0x722f37)
			.description("This leaderboard is only available in the month of December!")
			.title("Advent of Code Leaderboard");

		interactions::response::Response {
			r#type: interactions::response::ResponseType::ChannelMessageWithSource,
			data: Some(interactions::response::ResponseData {
				tts: false,
				content: "".to_string(),
				embeds: vec![serenity::utils::hashmap_to_json_map(embed.0.clone())],
				allowed_mentions: interactions::response::AllowedMentions::none(),
			}),
		}
	}
}

pub async fn role_interaction(
	ctx: &Context,
	interaction: &interactions::Interaction,
) -> interactions::response::Response {
	let interaction = interaction.clone();
	let mut desc = "An error occured".to_string();
	if let Some(config) = unsafe { &CONFIG } {
		if let Some(role_value) = &interaction.data.options {
			if let Some(role_id) = &role_value[0].value {
				let guild = ctx
					.http
					.get_guild(interaction.guild_id.parse::<u64>().unwrap())
					.await
					.unwrap();
				let mut member = guild
					.member(
						ctx.http.clone(),
						interaction.member.user.id.parse::<u64>().unwrap(),
					)
					.await
					.unwrap();
				let role = guild
					.roles
					.get(&RoleId(role_id.as_str().unwrap().parse::<u64>().unwrap()))
					.unwrap();
				let roles = member.roles(ctx).await.unwrap();

				if config.role_allowed(&role.name.to_string()) {
					if roles.contains(role) {
						member.remove_role(ctx, role).await.unwrap();
						desc = format!("The \"{}\" role has been removed.", role.name);
					} else {
						member.add_role(ctx, role).await.unwrap();
						desc = format!("The \"{}\" role has been added.", role.name);
					}
				} else {
					desc = String::from("Permission denied.");
				}
			} else {
				desc = String::from("Unknown role given.");
			}
		}
	}

	let mut e = CreateEmbed::default();
	e.color(0x722f37);
	e.description(desc);
	e.title("Role Notification");

	interactions::response::Response {
		r#type: interactions::response::ResponseType::ChannelMessageWithSource,
		data: Some(interactions::response::ResponseData {
			tts: false,
			content: "".to_string(),
			allowed_mentions: interactions::response::AllowedMentions::none(),
			embeds: vec![serenity::utils::hashmap_to_json_map(e.0.clone())],
		}),
	}
}

pub async fn roles_interaction(
	_: &Context,
	_: &interactions::Interaction,
) -> interactions::response::Response {
	if let Some(config) = unsafe { &CONFIG } {
		let mut e = CreateEmbed::default();
		e.color(0x722f37);
		e.description(
			config
				.roles()
				.iter()
				.map(|&x| x.clone())
				.collect::<Vec<String>>()
				.join("\n"),
		);
		e.title("Role Notification");

		interactions::response::Response {
			r#type: interactions::response::ResponseType::ChannelMessageWithSource,
			data: Some(interactions::response::ResponseData {
				tts: false,
				content: "".to_string(),
				allowed_mentions: interactions::response::AllowedMentions::none(),
				embeds: vec![serenity::utils::hashmap_to_json_map(e.0.clone())],
			}),
		}
	} else {
		interactions::response::Response {
			r#type: interactions::response::ResponseType::ChannelMessageWithSource,
			data: Some(interactions::response::ResponseData {
				tts: false,
				content: "Failed to read config".to_string(),
				allowed_mentions: interactions::response::AllowedMentions::none(),
				embeds: vec![],
			}),
		}
	}
}
