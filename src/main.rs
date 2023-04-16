#![feature(async_closure, exit_status_error)]

use tokio::{sync::Mutex, task::spawn};

use actix_web::{
    middleware::Logger,
    post,
    web::{Data, Json},
    App, HttpResponse, HttpServer,
};

use env_logger::Env;
use regex::Regex;

use serde::Deserialize;
use structs::{CommitStatus, PushRequest, StatusRequestBody};

mod structs;
mod utils;

const TARGET_DIR: &str = "target_repo";

#[derive(Deserialize)]
struct Config {
    token: String,
    main_branch: String,
}

type Lock = Data<tokio::sync::Mutex<()>>;

#[post("/push")]
async fn push(Json(request): Json<PushRequest>, config: Data<Config>, lock: Lock) -> HttpResponse {
    println!("Processing push request on branch: {}", request.branch());
    let re = Regex::new(r"^0+$").unwrap();
    if re.is_match(&request.after) {
        println!("\tMerge commit detected, aborting...");
        HttpResponse::Ok().body("SHA was zero, no build was tested.")
    } else {
        spawn(run_ci(request, config, lock));
        println!("\tTasks started, sending response...");
        HttpResponse::Accepted().body("Push request accepted. Building repository...")
    }
}

async fn run_ci(request: PushRequest, config: Data<Config>, lock: Lock) {
    use CommitStatus::*;
    let repo = &request.repository;
    let post = |status, description| {
        post_status(
            &repo.full_name,
            &request.after,
            status,
            description,
            &config.token,
        )
    };
    let _guard = match lock.try_lock() {
        Ok(g) => g,
        Err(_) => {
            post(Pending, "Waiting for previous job to finish...").await;
            lock.lock().await
        }
    };
    post(Pending, "Building repository and running tests...").await;
    println!("\tUpdating target repository...");
    let report = async move |status, error_msg| {
        post(status, error_msg).await;
        println!("\t{error_msg}");
    };
    if utils::update_target(&repo.clone_url, &request.branch(), &config.main_branch).is_err() {
        report(Error, "Failed to update local repository.").await;
        return;
    }
    println!("\tBuilding project...");
    if utils::run_command("cargo", &["build"]).is_err() {
        report(Failure, "Failed to build project.").await;
        return;
    }
    println!("\tRunning tests...");
    if utils::run_command("cargo", &["test"]).is_err() {
        report(Failure, "One or more tests failed.").await;
        return;
    }
    post(Success, "All tests passed.").await;
    println!("\tAll tasks complete.\n");
}

async fn post_status(
    repo_name: &str,
    sha: &str,
    status: CommitStatus,
    description: &str,
    token: &str,
) {
    let context = "Custom CI server";
    let req = reqwest::Client::new()
        .post(format!(
            "https://api.github.com/repos/{repo_name}/statuses/{sha}"
        ))
        .header("Authorization", format!("Bearer {token}"))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", context)
        .json(&StatusRequestBody::new(status, description, context));
    let res = req.send().await.unwrap();
    if !res.status().is_success() {
        println!("\tError: {}", res.text().await.unwrap());
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    let config = Data::new(
        serde_json::from_str::<Config>(
            &std::fs::read_to_string("config.json")
                .expect("Could not find `config.json` in prject root."),
        )
        .unwrap(),
    );
    let lock = Data::new(Mutex::new(()));
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(config.clone())
            .app_data(lock.clone())
            .service(push)
    })
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
