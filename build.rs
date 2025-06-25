use std::{env, fs, path::Path, process::Command};

use anyhow::{Result, anyhow};
use which::which;

fn join(root: &str, next: &str) -> String {
    Path::new(root).join(next).to_str().unwrap().to_string()
}

fn exec(command: &str, work_dir: &str) -> Result<String> {
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
        Err(anyhow!("{}", unsafe {
            String::from_utf8_unchecked(output.stderr)
        }))
    } else {
        Ok(unsafe { String::from_utf8_unchecked(output.stdout) })
    }
}

fn get_binary_name() -> String {
    format!(
        "cef_binary_137.0.17+gf354b0e+chromium-137.0.7151.104_{}{}_minimal",
        if cfg!(target_os = "macos") {
            "macos"
        } else if cfg!(target_os = "windows") {
            "windows"
        } else {
            "linux"
        },
        if cfg!(target_arch = "aarch64") {
            "arm64"
        } else if cfg!(target_arch = "x86_64") {
            if cfg!(target_os = "macos") {
                "x64"
            } else {
                "64"
            }
        } else {
            "32"
        }
    )
}

fn get_binary_url() -> String {
    format!(
        "https://cef-builds.spotifycdn.com/{}.tar.bz2",
        get_binary_name().replace("+", "%2B")
    )
}

#[cfg(not(target_os = "windows"))]
fn download_cef(outdir: &str) -> Result<()> {
    exec(
        &format!(
            "curl \
                -L \
                --retry 10 \
                --retry-delay 3 \
                --retry-connrefused \
                --retry-max-time 300 \
                -o ./cef.tar.bz2 \"{}\"",
            get_binary_url(),
        ),
        outdir,
    )?;

    exec("tar -xjf ./cef.tar.bz2 -C ./", outdir)?;
    exec("rm -f ./cef.tar.bz2", outdir)?;
    exec(&format!("mv ./{} ./cef", get_binary_name()), outdir)?;
    exec(
        "mv ./cef/Release/cef_sandbox.a ./cef/Release/libcef_sandbox.a",
        outdir,
    )?;

    Ok(())
}

#[cfg(target_os = "windows")]
fn download_cef(outdir: &str) -> Result<()> {
    if !fs::exists(&join(outdir, "./7za.exe")).unwrap_or(false) {
        exec(
            "Invoke-WebRequest -Uri 'https://7-zip.org/a/7za920.zip' -OutFile ./7za.zip",
            outdir,
        )?;

        exec(
            "Expand-Archive -Path ./7za.zip -DestinationPath ./7za",
            outdir,
        )?;

        exec("Move-Item ./7za/7za.exe ./7za.exe", outdir)?;
        exec("Remove-Item -Recurse -Force ./7za", outdir)?;
        exec("Remove-Item ./7za.zip", outdir)?;
    }

    exec(
        &format!(
            "Invoke-WebRequest -Uri {} -OutFile ./cef.tar.bz2",
            get_binary_url(),
        ),
        outdir,
    )?;

    exec("./7za.exe x ./cef.tar.bz2", outdir)?;
    exec("./7za.exe x ./cef.tar", outdir)?;
    exec("Remove-Item ./cef.tar.bz2", outdir)?;
    exec("Remove-Item ./cef.tar", outdir)?;
    exec(
        &format!("Rename-Item ./{} ./cef", get_binary_name()),
        outdir,
    )?;

    Ok(())
}

fn make_cef(cef_dir: &str) -> Result<()> {
    if which("cmake").is_err() {
        panic!("
            You don't have cmake installed, compiling srt requires cmake to do it, now it's unavoidable, you need to install cmake.
                On debian/ubuntu, you can install it with `sudo apt install cmake`.
                On window, it requires you to go to the official cmake website to load the installation file.
        ");
    }

    exec(
        &format!(
            "cmake {} -DCMAKE_BUILD_TYPE=Release .",
            if cfg!(not(target_os = "windows")) {
                "-DCMAKE_CXX_FLAGS=\"-Wno-deprecated-builtins\""
            } else {
                ""
            }
        ),
        cef_dir,
    )?;

    exec("cmake --build . --config Release", cef_dir)?;

    Ok(())
}

fn make_bindgen(outdir: &str, cef_dir: &str) -> Result<()> {
    use bindgen::{Builder, EnumVariation};

    Builder::default()
        .default_enum_style(EnumVariation::Rust {
            non_exhaustive: false,
        })
        .generate_comments(false)
        .prepend_enum_name(false)
        .size_t_is_usize(true)
        .clang_arg(format!("-I{}", cef_dir))
        .header("./cxx/library.h")
        .generate()?
        .write_to_file(&join(outdir, "bindings.rs"))?;

    Ok(())
}

fn make_library(outdir: &str, cef_dir: &str) -> Result<()> {
    let is_debug = env::var("DEBUG")
        .map(|label| label == "true")
        .unwrap_or(true);

    let mut compiler = cc::Build::new();

    compiler
        .cpp(true)
        .debug(is_debug)
        .static_crt(true)
        .target(&env::var("TARGET")?)
        .warnings(false)
        .out_dir(&outdir)
        .flag(if cfg!(target_os = "windows") {
            "/std:c++20"
        } else {
            "-std=c++20"
        })
        .include(cef_dir)
        .file("./cxx/util.cpp")
        .file("./cxx/runtime.cpp")
        .file("./cxx/request.cpp")
        .file("./cxx/subprocess.cpp")
        .file("./cxx/webview.cpp");

    #[cfg(target_os = "windows")]
    compiler
        .define("WIN32", Some("1"))
        .define("_WINDOWS", None)
        .define("__STDC_CONSTANT_MACROS", None)
        .define("__STDC_FORMAT_MACROS", None)
        .define("_WIN32", None)
        .define("UNICODE", None)
        .define("_UNICODE", None)
        .define("WINVER", Some("0x0A00"))
        .define("_WIN32_WINNT", Some("0x0A00"))
        .define("NTDDI_VERSION", Some("NTDDI_WIN10_FE"))
        .define("NOMINMAX", None)
        .define("WIN32_LEAN_AND_MEAN", None)
        .define("_HAS_EXCEPTIONS", Some("0"))
        .define("PSAPI_VERSION", Some("1"))
        .define("CEF_USE_SANDBOX", None)
        .define("CEF_USE_ATL", None)
        .define("_HAS_ITERATOR_DEBUGGING", Some("0"));

    #[cfg(target_os = "linux")]
    compiler
        .define("LINUX", Some("1"))
        .define("CEF_X11", Some("1"));

    #[cfg(target_os = "macos")]
    compiler.define("MACOS", Some("1"));

    compiler.compile("wew-sys");

    Ok(())
}

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=./cxx");
    println!("cargo:rerun-if-changed=./build.rs");

    let outdir = env::var("OUT_DIR")?;
    let cef_dir: &str = &join(&outdir, "./cef");

    make_bindgen(&outdir, cef_dir)?;

    if std::env::var("DOCS_RS").is_ok() {
        return Ok(());
    }

    if !fs::exists(cef_dir).unwrap_or(false) {
        download_cef(&outdir)?;
    }

    if !fs::exists(&join(cef_dir, "./libcef_dll_wrapper")).unwrap_or(false) {
        make_cef(cef_dir)?;
    }

    make_library(&outdir, cef_dir)?;

    println!("cargo:rustc-link-lib=static=wew-sys");
    println!("cargo:rustc-link-search=all={}", &outdir);

    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-lib=libcef");
        println!("cargo:rustc-link-lib=libcef_dll_wrapper");
        println!(
            "cargo:rustc-link-search=all={}",
            join(cef_dir, "./libcef_dll_wrapper/Release")
        );

        println!("cargo:rustc-link-search=all={}", join(cef_dir, "./Release"));
    }

    #[cfg(target_os = "linux")]
    {
        println!("cargo:rustc-link-lib=cef");
        println!("cargo:rustc-link-lib=cef_dll_wrapper");
    }

    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-lib=framework=Chromium Embedded Framework");
        println!(
            "cargo:rustc-link-search=framework={}",
            join(cef_dir, "./Release")
        );

        println!("cargo:rustc-link-lib=cef_dll_wrapper");
        println!(
            "cargo:rustc-link-search=all={}",
            join(cef_dir, "./libcef_dll_wrapper")
        );

        println!(
            "cargo:rustc-link-search=native={}",
            join(cef_dir, "Release")
        );
    }

    Ok(())
}
