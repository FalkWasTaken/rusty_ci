use actix_web::{
    post,
    web::{Data, Json},
    App, HttpResponse, HttpServer,
};

use regex::Regex;

use serde::Deserialize;
use structs::{CommitStatus, PushRequest, StatusRequestBody};

mod structs;
mod utils;

const TARGET_DIR: &str = "target_repo";

#[derive(Clone, Copy, Deserialize)]
struct Config {
    status_token: &'static str,
    main_branch: &'static str,
}

#[post("/push")]
async fn push(Json(request): Json<PushRequest>, config: Data<Config>) -> HttpResponse {
    println!("Processing push request on branch: {}", request.base_ref);
    let re = Regex::new(r"^0+$").unwrap();
    if re.is_match(&request.after) {
        println!("Merge commit detected, aborting...");
        HttpResponse::Ok().body("SHA was zero, no build was tested.")
    } else {
        tokio::task::spawn(run_ci(request, config));
        println!("Build started, sending response...");
        HttpResponse::Accepted().body("Push request accepted. Building repository...")
    }
}

async fn run_ci(request: PushRequest, config: Data<Config>) {
    let repo = request.repository;
    let post = |status, description| {
        post_status(
            &repo.full_name,
            &request.after,
            status,
            description,
            config.status_token,
        )
    };
    post(
        CommitStatus::Pending,
        "Building repository and running tests...",
    )
    .await;
    if !utils::update_target(&repo.clone_url, &request.base_ref, config.main_branch).success() {
        post(CommitStatus::Error, "Failed to update local repository.").await;
        return;
    }
    if !utils::run_command("cargo", &["build"]).success() {
        post(CommitStatus::Failure, "Failed to build project.").await;
        return;
    }
    if !utils::run_command("cargo", &["test"]).success() {
        post(CommitStatus::Failure, "One or more tests failed.").await;
        return;
    }
    post(CommitStatus::Success, "All tests passed.").await;
}

async fn post_status<'a>(
    repo_name: &'a str,
    commit_sha: &'a str,
    status: CommitStatus,
    description: &'static str,
    token: &'a str,
) {
    let client = reqwest::Client::new();
    let res = client
        .post(format!(
            "https://api.github.com/repos/{}/statuses/{}",
            repo_name, commit_sha
        ))
        .bearer_auth(token)
        .header("Accept", "application/vnd.github+json")
        .body(StatusRequestBody::new(status, description, "Custom CI server").to_json())
        .send()
        .await
        .unwrap();
    if !res.status().is_success() {
        println!("Error: {}", res.text().await.unwrap());
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config: Config = serde_json::from_str(include_str!("../config.json")).unwrap();

    HttpServer::new(move || App::new().app_data(config).service(push))
        .bind(("localhost", 8080))?
        .run()
        .await
}

#[cfg(test)]
mod test {
    #[test]
    fn test1() {
        let x = 10;
        let y = 5;
        assert_eq!(15, x + y)
    }
}