use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CommitStatus {
    Error,
    Failure,
    Pending,
    Success,
}

#[derive(Deserialize)]
pub struct PushRequest {
    pub before: String,
    pub after: String,
    #[serde(rename = "ref")]
    pub reference: String,
    pub repository: Repository,
}

impl PushRequest {
    pub fn branch(&self) -> String {
        self.reference.split('/').last().unwrap().to_string()
    }
}

#[derive(Deserialize)]
pub struct Repository {
    pub clone_url: String,
    pub full_name: String,
}

#[derive(Serialize)]
pub struct StatusRequestBody<'a> {
    state: CommitStatus,
    description: &'a str,
    context: &'a str,
}

impl<'a> StatusRequestBody<'a> {
    pub fn new(state: CommitStatus, description: &'a str, context: &'a str) -> Self {
        StatusRequestBody { state, description, context }
    }
}