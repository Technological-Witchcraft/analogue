use ::serde::Deserialize;
use ::std::{cmp::Ordering, collections::HashMap};

#[derive(Debug, Deserialize)]
pub struct Leaderboard {
	event: String,
	members: HashMap<String, Member>,
	owner_id: String,
}

impl Leaderboard {
	pub fn members(&self) -> Vec<&Member> {
		self.members.values().collect()
	}
}

#[derive(Debug, Deserialize)]
pub struct Member {
	id: String,
	local_score: usize,
	name: String,
	stars: usize,
}

impl Member {
	pub fn name(&self) -> String {
		self.name.clone()
	}

	pub fn score(&self) -> usize {
		self.local_score
	}

	pub fn stars(&self) -> usize {
		self.stars
	}
}

impl Eq for Member {}

impl Ord for Member {
	fn cmp(&self, other: &Self) -> Ordering {
		self.local_score.cmp(&other.local_score)
	}
}

impl PartialOrd for Member {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl PartialEq for Member {
	fn eq(&self, other: &Self) -> bool {
		self.local_score == other.local_score
	}
}
