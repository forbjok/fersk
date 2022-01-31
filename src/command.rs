use std::process::Command;

use anyhow::{anyhow, Context};

pub fn exec_command(command: &str, f: impl FnOnce(&mut Command)) -> Result<(), anyhow::Error> {
    let mut command = Command::new(command);

    f(&mut command);

    // Execute command
    let status = command.status().with_context(|| "Error executing command")?;

    if !status.success() {
        return Err(anyhow!(
            "Command returned with a non-success error code: {}",
            status.code().unwrap_or(-1)
        ));
    }

    Ok(())
}
