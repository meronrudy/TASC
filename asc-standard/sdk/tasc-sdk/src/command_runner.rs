use std::process::{Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

use crate::SdkError;

#[derive(Debug, Clone)]
pub struct CommandResult {
    pub status: ExitStatus,
    pub stdout: String,
    pub stderr: String,
    pub duration: Duration,
}

pub fn run(command: &mut Command, timeout: Option<Duration>) -> Result<CommandResult, SdkError> {
    let result = run_allow_failure(command, timeout)?;
    if !result.status.success() {
        return Err(SdkError::CommandFailed {
            code: result.status.code(),
            stdout: result.stdout,
            stderr: result.stderr,
        });
    }

    Ok(result)
}

pub fn run_allow_failure(
    command: &mut Command,
    timeout: Option<Duration>,
) -> Result<CommandResult, SdkError> {
    let start = Instant::now();
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let output = if let Some(timeout) = timeout {
        loop {
            if child.try_wait()?.is_some() {
                break;
            }
            if start.elapsed() >= timeout {
                child.kill()?;
                let output = child.wait_with_output()?;
                return Err(SdkError::Timeout {
                    after_ms: timeout.as_millis() as u64,
                    stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                    stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                });
            }
            std::thread::sleep(Duration::from_millis(25));
        }
        child.wait_with_output()?
    } else {
        child.wait_with_output()?
    };

    Ok(CommandResult {
        status: output.status,
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        duration: start.elapsed(),
    })
}
