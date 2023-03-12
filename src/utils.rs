use std::{
    path::Path,
    process::{Command, ExitStatusError, Stdio},
};

use crate::TARGET_DIR;

pub fn update_target(url: &str, branch: &str, main_branch: &str) -> Result<(), ExitStatusError> {
    if !Path::new(&format!("{TARGET_DIR}/.git")).exists() {
        Command::new("git")
            .args(["clone", url, TARGET_DIR])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .unwrap()
            .exit_ok()?;
    }
    run_command("git", &["checkout", main_branch])?;
    run_command("git", &["pull"])?;
    run_command("git", &["checkout", branch])?;
    run_command("git", &["pull"])
}

pub fn run_command(command: &str, args: &[&str]) -> Result<(), ExitStatusError> {
    Command::new(command)
        .args(args)
        .current_dir(TARGET_DIR)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap()
        .exit_ok()
}
