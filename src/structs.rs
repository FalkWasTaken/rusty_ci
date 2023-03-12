use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub enum CommitStatus {
    Error,
    Failure,
    Pending,
    Success,
}

impl Into<&str> for &CommitStatus {
    fn into(self) -> &'static str {
        match self {
            CommitStatus::Error => "error",
            CommitStatus::Failure => "failure",
            CommitStatus::Pending => "pending",
            CommitStatus::Success => "success",
        }
    }
}

impl Display for CommitStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.into())
    }
}

#[derive(Deserialize)]
pub struct PushRequest {
    pub before: String,
    pub after: String,
    pub base_ref: String,
    pub commits: Vec<Commit>,
    pub compare: String,
    pub created: bool,
    pub deleted: bool,
    pub enerprise: String,
    pub forced: bool,
    pub head_commit: Commit,
    pub pusher: Pusher,
    pub full_ref: String,
    pub repository: Repository,
}

#[derive(Deserialize)]
pub struct Commit {}

#[derive(Deserialize)]
pub struct Repository {
    pub clone_url: String,
    pub name: String,
    //pub owner: Actor,
    pub full_name: String,
}

#[derive(Deserialize)]
pub struct Pusher {
    pub date: String,
    pub email: String,
    pub name: String,
    pub username: String,
}

//#[derive(Deserialize)]
//pub struct Actor {
//    pub login: String,
//    pub id: String,
//    pub node_id: String,
//    pub avatar_url: String,
//    pub gravatar_id: String,
//    pub url: String,
//    pub html_url: String,
//    pub followers_url: String,
//    pub following_url: String,
//    pub gists_url: String,
//    pub starred_url: String,
//    pub subscriptions_url: String,
//    pub organizations_url: String,
//    pub repos_url: String,
//    pub events_url: String,
//    pub received_events_url: String,
//    #[serde(rename = "type")]
//    pub actor_type: String,
//    pub site_admin: bool,
//}

#[derive(Serialize)]
pub struct StatusRequestBody<'a> {
    status: CommitStatus,
    description: &'a str,
    context: &'a str,
}

impl<'a> StatusRequestBody<'a> {
    pub fn new(status: CommitStatus, description: &'a str, context: &'a str) -> Self {
        Self {
            status,
            description,
            context,
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}
