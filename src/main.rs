#![feature(exit_status_error)]

use tokio::{sync::Mutex, task::spawn};

use actix_web::{
    get,
    middleware::Logger,
    post,
    web::{Data, Json},
    App, HttpResponse, HttpServer, Responder,
};

use env_logger::Env;
use regex::Regex;

use structs::{CIResult, CommitStatus, Config, PushRequest};
use utils::Log;

use crate::utils::post_status;

mod structs;
mod utils;

const TARGET_DIR: &str = "target_repo";

type Lock = Data<tokio::sync::Mutex<()>>;

#[get("/logs/{sha}")]
async fn logs(sha: actix_web::web::Path<String>) -> impl Responder {
    std::fs::read_to_string(format!("logs/{sha}.txt"))
}

#[post("/push")]
async fn push(Json(request): Json<PushRequest>, config: Data<Config>, lock: Lock) -> HttpResponse {
    println!("Processing push request on branch: {}", request.branch());
    let re = Regex::new(r"^0+$").unwrap();
    if re.is_match(&request.after) {
        println!("\tMerge commit detected, aborting...");
        HttpResponse::Ok().body("SHA was zero, no build was tested.")
    } else {
        spawn(async move {
            let mut log = Log::new(&format!("logs/{}.txt", request.after));
            log.log(&format!(
                "Processing push request on branch: {}",
                request.branch()
            ));
            let res = run_ci(&request, &config, lock, &mut log).await;
            log.log(&res.msg);
            post_status(&request, &config, res, &mut log).await;
        });
        println!("\tTasks started, sending response...");
        HttpResponse::Accepted().body("Push request accepted. Building repository...")
    }
}

async fn run_ci(
    request: &PushRequest,
    config: &Data<Config>,
    lock: Lock,
    log: &mut Log,
) -> CIResult {
    use CommitStatus::*;
    let repo = &request.repository;
    let _guard = match lock.try_lock() {
        Ok(g) => g,
        Err(_) => {
            post_status(
                request,
                config,
                (Pending, "Waiting for previous job to finish...").into(),
                log,
            )
            .await;
            lock.lock().await
        }
    };
    post_status(request, config, (Pending, "Running tasks...").into(), log).await;

    log.log("Updating target repository...");
    if utils::update_target(&repo.clone_url, &request.branch(), &config.main_branch, log).is_err() {
        return (Error, "Failed to update local repository.").into();
    }
    for task in &config.tasks {
        log.log("---------------------------------------------------------");
        log.log(&format!("Running task `{}`...", task.name));
        if utils::run_command(&task.command, &task.args, log).is_err() {
            return (Failure, format!("Task `{}` failed.", task.name)).into();
        }
    }
    (Success, "All tasks finished successfuly.").into()
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
            .service(logs)
    })
    .bind(("localhost", 8080))?
    .run()
    .await
}

#[cfg(test)]
mod dummy_tests {
    #[test]
    fn test1() {
        let x = 10;
        let y = 5;
        assert_eq!(15, x + y)
    }
}
