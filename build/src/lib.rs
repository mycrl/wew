use std::{
    io::{Error, ErrorKind},
    process::Command,
};

pub static CEF_PATH: &str = concat!(env!("OUT_DIR"), "/cef");

fn exec(command: &str, work_dir: &str) -> Result<String, Error> {
    let output = Command::new(if cfg!(windows) { "powershell" } else { "bash" })
        .arg(if cfg!(windows) { "-command" } else { "-c" })
        .arg(if cfg!(windows) {
            format!("$ProgressPreference = 'SilentlyContinue';{}", command)
        } else {
            command.to_string()
        })
        .current_dir(work_dir)
        .output()?;
    if !output.status.success() {
        Err(Error::new(ErrorKind::Other, unsafe {
            String::from_utf8_unchecked(output.stderr)
        }))
    } else {
        Ok(unsafe { String::from_utf8_unchecked(output.stdout) })
    }
}

pub fn copy_resources(output: &str) -> Result<(), Error> {
    exec(
        &format!(
            "Copy-Item -Path \"{}/Resources/*\" -Destination \"{}\" -Recurse -Force",
            CEF_PATH, output
        ),
        CEF_PATH,
    )?;

    exec(
        &format!(
            "Copy-Item -Path \"{}/Release/*\" -Destination \"{}\" -Exclude \"*.lib\" -Recurse -Force",
            CEF_PATH, output
        ),
        CEF_PATH,
    )?;

    Ok(())
}
