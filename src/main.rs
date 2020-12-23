use ::chrono::{DateTime, Datelike, NaiveDateTime, Utc};
use ::reqwest::Client as HTTPClient;
use ::serde::Deserialize;
use ::serenity::{
	async_trait,
	client::{Context, EventHandler},
	framework::{
		standard::{
			macros::{command, group},
			CommandResult,
		},
		StandardFramework,
	},
	model::{
		channel::{Channel, Message},
		gateway::Ready,
	},
	Client,
};
use ::std::{collections::HashMap, error::Error, fs::File, io::Read, path::Path};

pub mod advent;
mod interaction_commands;
pub mod interactions;

static mut ADVENT_OF_CODE: (String, usize) = (String::new(), 0);
static mut CONFIG: Option<AnalogueConfig> = None;

struct Analogue;

#[async_trait]
impl EventHandler for Analogue {
	async fn ready(&self, _: Context, _: Ready) {
		if let Some(config) = unsafe { &CONFIG } {
			let role_commands = vec![interactions::definition::CommandOption {
				r#type: 8,
				name: "role".to_string(),
				description: "The role to add/remove".to_string(),
				default: None,
				required: true,
				choices: None,
				options: None,
			}];

			interactions::construct_interactions(
				vec![
					interactions::definition::Command {
						name: "advent".to_string(),
						description: "Shows the leaderboard for the Advent of Code".to_string(),
						options: vec![],
					},
					interactions::definition::Command {
						name: "role".to_string(),
						description: "Used for self-assigning roles".to_string(),
						options: role_commands,
					},
					interactions::definition::Command {
						name: "roles".to_string(),
						description: "Display a list of roles that can be assigned".to_string(),
						options: vec![],
					},
				],
				"791006048823148554",
				"791005585243242546",
				&config.token(),
			)
			.await;
		}
	}

	async fn unknown(&self, ctx: Context, name: String, raw: serde_json::Value) {
		if name == "INTERACTION_CREATE" {
			// Handle an interaction command.
			if let Some(config) = unsafe { &CONFIG } {
				println!("{:#?}", raw);
				let interaction =
					serde_json::from_str::<interactions::Interaction>(&raw.to_string()).unwrap();

				match &interaction.data.name[..] {
					"advent" => {
						let response =
							interaction_commands::advent_interaction(&ctx, &interaction).await;
						interactions::send_interaction_response(
							response,
							&interaction.id,
							&interaction.token,
							&config.token(),
						)
						.await
						.unwrap();
					}
					"role" => {
						let response =
							interaction_commands::role_interaction(&ctx, &interaction).await;
						interactions::send_interaction_response(
							response,
							&interaction.id,
							&interaction.token,
							&config.token(),
						)
						.await
						.unwrap();
					}
					"roles" => {
						let response =
							interaction_commands::roles_interaction(&ctx, &interaction).await;
						interactions::send_interaction_response(
							response,
							&interaction.id,
							&interaction.token,
							&config.token(),
						)
						.await
						.unwrap();
					}
					_ => {}
				}
			}
		}
	}
}

#[derive(Deserialize)]
struct AnalogueConfig {
	advent_session: String,
	roles: HashMap<String, usize>,
	token: String,
}

impl AnalogueConfig {
	fn advent_session(&self) -> String {
		self.advent_session.clone()
	}

	fn role_allowed(&self, name: &String) -> bool {
		self.roles.contains_key(name)
	}

	fn roles(&self) -> Vec<&String> {
		self.roles.keys().collect()
	}

	fn token(&self) -> String {
		self.token.clone()
	}
}

#[group]
#[commands(help, advent, role)]
struct Command;

#[command]
async fn help(ctx: &Context, msg: &Message) -> CommandResult {
	if let Channel::Guild(channel) = msg.channel(ctx).await.unwrap() {
		channel
	    .send_message(ctx, |m| {
		m.embed(|e| {
		    e.color(0x722f37);
		    e.description(
			"
					`advent` - Shows the leaderboard for the Advent of Code
					`help` - Shows this lovely message
					`role` - Used for self-assigning roles
                                        
                                        *Note: These commands are also available as /advent, /role, and /roles*
				",
		    );
		    e.title("List of Commands");
		    e
		});
		m
	    })
	    .await?;
	}
	Ok(())
}

#[::tokio::main]
async fn main() {
	unsafe {
		CONFIG = Some(read_config("analogue.toml").await.unwrap());
	}
	let framework = StandardFramework::new()
		.configure(|c| c.prefix("a?"))
		.group(&COMMAND_GROUP);
	if let Some(config) = unsafe { &CONFIG } {
		let mut client = Client::builder(config.token())
			.event_handler(Analogue)
			.framework(framework)
			.await
			.unwrap();
		if let Err(e) = client.start().await {
			eprintln!("{}", e);
		}
	}
}

async fn read_config<P: AsRef<Path>>(path: P) -> Result<AnalogueConfig, Box<dyn Error>> {
	let mut config_file = File::open(path)?;
	let mut buffer = String::new();
	config_file.read_to_string(&mut buffer)?;
	let config = ::toml::from_str(&buffer)?;
	Ok(config)
}

#[command]
async fn advent(ctx: &Context, msg: &Message) -> CommandResult {
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
					.build()?;
				let body = client.execute(request).await?.text().await?;
				advent_of_code.0 = body;
				advent_of_code.1 = now.timestamp() as usize + 900;
			}
		}
		let leaderboard: advent::Leaderboard = ::serde_json::from_str(&advent_of_code.0.clone())?;
		let mut ranking = leaderboard.members();
		ranking.sort_unstable();
		if let Channel::Guild(channel) = msg.channel(ctx).await.unwrap() {
			channel
				.send_message(ctx, |m| {
					m.embed(|e| {
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
						e
					});
					m
				})
				.await?;
		}
	} else {
		if let Channel::Guild(channel) = msg.channel(ctx).await.unwrap() {
			channel
				.send_message(ctx, |m| {
					m.embed(|e| {
						e.color(0x722f37);
						e.description(
							"This leaderboard is only available in the month of December!",
						);
						e.title("Advent of Code Leaderboard");
						e
					});
					m
				})
				.await?;
		}
	}
	Ok(())
}

#[command]
async fn role(ctx: &Context, msg: &Message) -> CommandResult {
	let args: Vec<&str> = msg.content.split(" ").collect();
	let guild = msg.guild(ctx).await.unwrap();
	let mut member = guild.member(ctx, msg.author.id).await.unwrap();
	let roles = member.roles(ctx).await.unwrap();
	let mut desc = String::new();
	if let Some(config) = unsafe { &CONFIG } {
		if let Channel::Guild(channel) = msg.channel(ctx).await.unwrap() {
			if args.len() > 1 {
				let role_name = &msg.content[7..];
				if let Some(role) = guild.role_by_name(role_name) {
					if config.role_allowed(&role_name.to_string()) {
						if roles.contains(role) {
							member.remove_role(ctx, role).await.unwrap();
							desc = format!("The \"{}\" role has been removed.", role_name);
						} else {
							member.add_role(ctx, role).await.unwrap();
							desc = format!("The \"{}\" role has been added.", role_name);
						}
					} else {
						desc = String::from("Permission denied.");
					}
				} else {
					desc = String::from("Unknown role given.");
				}
			} else {
				let mut roles = config.roles();
				roles.sort_unstable();
				for r in roles {
					desc += format!("{}\n", r).as_str();
				}
			}
			channel
				.send_message(ctx, |m| {
					m.embed(|e| {
						e.color(0x722f37);
						e.description(desc);
						if args.len() > 1 {
							e.title("Role Notification");
						} else {
							e.title("List of Roles");
						}
						e
					});
					m
				})
				.await?;
		}
	}
	Ok(())
}
