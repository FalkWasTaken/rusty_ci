use std::{
    path::Path,
    process::{Command, ExitStatus},
};

use crate::TARGET_DIR;

pub fn update_target(url: &str, branch: &str, main_branch: &str) -> ExitStatus {
    if !Path::new(&format!("{TARGET_DIR}/.git")).exists() {
        let status = Command::new("git")
            .args(["clone", url, TARGET_DIR])
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
        if !status.success() {
            return status;
        }
    }
    for args in [vec!["checkout", main_branch], vec!["pull"], vec!["checkout", branch]] {
        let status = run_command("git", &args);
        if !status.success() {
            return status;
        }
    }
    run_command("git", &["pull"])
}

pub fn run_command(command: &str, args: &[&str]) -> ExitStatus {
    Command::new(command)
        .args(args)
        .current_dir(TARGET_DIR)
        .spawn()
        .unwrap()
        .wait()
        .unwrap()
}
