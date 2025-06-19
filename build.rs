use std::{env, fs, path::Path, process::Command};

use anyhow::{Result, anyhow};

fn is_exsit(dir: &str) -> bool {
    fs::metadata(dir).is_ok()
}

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

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=./cxx");
    println!("cargo:rerun-if-changed=./build.rs");

    let out_dir = env::var("OUT_DIR")?;
    let cef_path: &str = &join(&out_dir, "./cef");
    let is_debug = env::var("DEBUG")
        .map(|label| label == "true")
        .unwrap_or(true);

    if !is_exsit(cef_path) {
        #[cfg(target_os = "macos")]
        {
            exec(
                &format!(
                    "curl -L --retry 5 --retry-delay 2 --retry-connrefused -o ./cef.tar.bz2 \"{}\"",
                    get_binary_url(),
                ),
                &out_dir,
            )?;

            exec("tar -xjf ./cef.tar.bz2 -C ./", &out_dir)?;
            exec("rm -f ./cef.tar.bz2", &out_dir)?;
            exec(&format!("mv ./{} ./cef", get_binary_name()), &out_dir)?;
            exec(
                "mv ./cef/Release/cef_sandbox.a ./cef/Release/libcef_sandbox.a",
                &out_dir,
            )?;
        }

        #[cfg(target_os = "windows")]
        {
            exec(
                &format!(
                    "Invoke-WebRequest -Uri {URL}/cef-windows-{}.zip -OutFile ./cef.zip",
                    env::var("CARGO_CFG_TARGET_ARCH")?
                ),
                &out_dir,
            )?;

            exec("Expand-Archive -Path cef.zip -DestinationPath ./", &out_dir)?;
            exec("Remove-Item ./cef.zip", &out_dir)?;
        }
    }

    if !is_exsit(&join(cef_path, "./libcef_dll_wrapper")) {
        #[cfg(not(target_os = "windows"))]
        {
            exec(
                "cmake \
                -DCMAKE_CXX_FLAGS=\"-Wno-deprecated-builtins\" \
                -DCMAKE_BUILD_TYPE=Release .",
                cef_path,
            )?;
        }

        #[cfg(target_os = "windows")]
        {
            exec("cmake -DCMAKE_BUILD_TYPE=Release .", cef_path)?;
        }

        exec("cmake --build . --config Release", cef_path)?;
    }

    {
        let mut compiler = cc::Build::new();
        compiler
            .cpp(true)
            .debug(is_debug)
            .static_crt(true)
            .target(&env::var("TARGET")?)
            .warnings(false)
            .out_dir(&out_dir)
            .flag(if cfg!(target_os = "windows") {
                "/std:c++20"
            } else {
                "-std=c++20"
            })
            .file("./cxx/app.cpp")
            .file("./cxx/page.cpp")
            .file("./cxx/control.cpp")
            .file("./cxx/render.cpp")
            .file("./cxx/display.cpp")
            .file("./cxx/webview.cpp")
            .file("./cxx/scheme_handler.cpp")
            .include(cef_path);

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

        compiler.compile("sys");
    }

    {
        bindgen::Builder::default()
            .default_enum_style(bindgen::EnumVariation::Rust {
                non_exhaustive: false,
            })
            .generate_comments(false)
            .prepend_enum_name(false)
            .derive_eq(true)
            .size_t_is_usize(true)
            .header("./cxx/webview.h")
            .generate()?
            .write_to_file(&join(&out_dir, "bindings.rs"))?;
    }

    println!("cargo:rustc-link-lib=static=sys");
    println!("cargo:rustc-link-search=all={}", &out_dir);

    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-lib=libcef");
        println!("cargo:rustc-link-lib=libcef_dll_wrapper");
        println!("cargo:rustc-link-lib=delayimp");
        println!("cargo:rustc-link-lib=winmm");
        println!("cargo:rustc-link-lib=user32");
        println!("cargo:rustc-link-arg=/NODEFAULTLIB:libcmt.lib");
        println!(
            "cargo:rustc-link-search=all={}",
            join(cef_path, "./libcef_dll_wrapper/Release")
        );

        println!(
            "cargo:rustc-link-search=all={}",
            join(cef_path, "./Release")
        );
    }

    #[cfg(target_os = "linux")]
    {
        println!("cargo:rustc-link-lib=cef");
        println!("cargo:rustc-link-lib=cef_dll_wrapper");
        println!("cargo:rustc-link-lib=X11");
    }

    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-lib=framework=Chromium Embedded Framework");
        println!(
            "cargo:rustc-link-search=framework={}",
            join(cef_path, "./Release")
        );

        println!("cargo:rustc-link-lib=cef_dll_wrapper");
        println!(
            "cargo:rustc-link-search=all={}",
            join(cef_path, "./libcef_dll_wrapper")
        );

        println!(
            "cargo:rustc-link-search=native={}",
            join(cef_path, "Release")
        );
    }

    Ok(())
}
