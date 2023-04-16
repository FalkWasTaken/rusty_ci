use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Config {
    pub token: String,
    pub main_branch: String,
    pub base_url: String,
    pub tasks: Vec<Task>,
}

#[derive(Deserialize)]
pub struct Task {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
}

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
    description: String,
    context: &'a str,
    target_url: &'a str,
}

impl<'a> StatusRequestBody<'a> {
    pub fn new(result: CIResult, context: &'a str, target_url: &'a str) -> Self {
        StatusRequestBody {
            state: result.status,
            description: result.msg,
            context,
            target_url,
        }
    }
}

pub struct CIResult {
    pub status: CommitStatus,
    pub msg: String,
}

impl From<(CommitStatus, String)> for CIResult {
    fn from((status, msg): (CommitStatus, String)) -> Self {
        CIResult { status, msg }
    }
}

impl<'a> From<(CommitStatus, &'a str)> for CIResult {
    fn from((status, msg): (CommitStatus, &'a str)) -> Self {
        CIResult {
            status,
            msg: msg.to_string(),
        }
    }
}
