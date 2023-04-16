use std::{
    borrow::Borrow,
    ffi::OsStr,
    fs::File,
    io::Write,
    path::Path,
    process::{Command, ExitStatusError, Stdio},
};

use crate::{TARGET_DIR, structs::{PushRequest, Config, CIResult, StatusRequestBody}};

pub fn update_target(
    url: &str,
    branch: &str,
    main_branch: &str,
    log: &mut Log,
) -> Result<(), ExitStatusError> {
    if !Path::new(&format!("{TARGET_DIR}/.git")).exists() {
        run_command("git", &["clone", url, TARGET_DIR], log)?;
    }
    run_command("git", &["checkout", main_branch], log)?;
    run_command("git", &["pull"], log)?;
    run_command("git", &["checkout", branch], log)?;
    run_command("git", &["pull"], log)
}

pub fn run_command<S: Borrow<str> + AsRef<OsStr>>(
    command: &str,
    args: &[S],
    log: &mut Log,
) -> Result<(), ExitStatusError> {
    let output = Command::new(command)
        .args(args)
        .current_dir(TARGET_DIR)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .unwrap();
    log.write_str(&String::from_utf8(output.stdout).unwrap());
    output.status.exit_ok()
}

pub async fn post_status(request: &PushRequest, config: &Config, res: CIResult, log: &mut Log) {
    let context = "Custom CI server";
    let req = reqwest::Client::new()
        .post(format!(
            "https://api.github.com/repos/{}/statuses/{}",
            request.repository.full_name, request.after
        ))
        .header("Authorization", format!("Bearer {}", config.token))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", context)
        .json(&StatusRequestBody::new(
            res,
            context,
            &format!("{}/logs/{}", config.base_url, request.after),
        ));
    let res = req.send().await.unwrap();
    if !res.status().is_success() {
        log.log(&format!("Error: {}", res.text().await.unwrap()));
    }
}

pub struct Log(File);

impl Log {
    pub fn new(path: &str) -> Log {
        Log(File::create(path).expect("Could not create log file"))
    }

    pub fn log(&mut self, message: &str) {
        println!("\t{message}");
        self.write_str(message);
    }

    fn write_str(&mut self, message: &str) {
        if let Err(e) = writeln!(self.0, "{message}") {
            eprintln!("Could not write to log file, {e}")
        }
    }
}
