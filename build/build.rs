use std::{
    env, fs,
    io::{Error, ErrorKind},
    path::Path,
    process::Command,
};

fn is_exsit(dir: &str) -> bool {
    fs::metadata(dir).is_ok()
}

fn join(root: &str, next: &str) -> String {
    Path::new(root).join(next).to_str().unwrap().to_string()
}

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

fn main() -> Result<(), Error> {
    println!("cargo:rerun-if-changed=./build.rs");

    let out_dir = env::var("OUT_DIR").unwrap();

    if !is_exsit(&join(&out_dir, "./cef")) {
        exec("Invoke-WebRequest -Uri https://github.com/mycrl/webview-rs/releases/download/distributions/cef-windows.zip -OutFile ./cef.zip", &out_dir)?;
        exec("Expand-Archive -Path cef.zip -DestinationPath ./", &out_dir)?;
        exec("Remove-Item ./cef.zip", &out_dir)?;
    }

    Ok(())
}
