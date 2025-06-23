// Build script for wew desktop application
// This script handles the build process for both Windows and macOS platforms
// It builds the frontend, Rust backend, and packages all necessary dependencies

import { exec } from "node:child_process";
import { cp, readdir, mkdir, rm, readFile, writeFile } from "node:fs/promises";
import { join, dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { existsSync } from "node:fs";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Parse command line arguments after '--' flag
// Example: node build.js --release
const Args = process.argv
    .slice(process.argv.indexOf("--") + 1)
    .map((item) => item.replace("--", ""))
    .reduce(
        (args, item) =>
            Object.assign(args, {
                [item]: true,
            }),
        {}
    );

// Helper function to execute shell commands with proper error handling
// Handles both Windows and Unix-like systems
function command(cmd, options = {}, output = true) {
    return new Promise(
        (
            resolve,
            reject,
            ps = exec(
                process.platform == "win32"
                    ? "$ProgressPreference = 'SilentlyContinue';" + cmd
                    : cmd,
                {
                    shell: process.platform == "win32" ? "powershell.exe" : "bash",
                    env: process.env,
                    stdio: "inherit",
                    cwd: __dirname,
                    ...options,
                }
            )
        ) => {
            if (output) {
                ps.stdout.pipe(process.stdout);
                ps.stderr.pipe(process.stderr);
            }

            ps.on("error", reject);
            ps.on("close", (code) => {
                code == 0 ? resolve() : reject(code || 0);
            });
        }
    );
}

// Helper function to locate Cargo build output directories
// Used to find compiled binaries and dependencies
async function getCrateOutdir(target, crate, dir) {
    const build = join(__dirname, target, "./build");
    for (const item of await readdir(build)) {
        if (item.startsWith(crate)) {
            const path = join(build, item, "./out/", dir);
            if (existsSync(path)) {
                return path;
            }
        }
    }

    throw new Error("not found cargo crate outdir");
}

void (async () => {
    await command("cargo build", {
        cwd: "./",
        env: {
            ...process.env,
            MACOSX_DEPLOYMENT_TARGET: "15.4",
            CACHE_PATH: resolve(__dirname, "../../target/cache"),
        },
    });

    if (!existsSync("../../target/windowless_rendering")) {
        await mkdir("../../target/windowless_rendering");
    }

    if (process.platform == "win32") {
        const cefOutDir = join(
            await getCrateOutdir(`../../target/debug`, "wew", "./cef/Release"),
            "../"
        );

        for (const item of [
            [
                `../../target/debug/windowless-rendering.exe`,
                "../../target/windowless_rendering/windowless-rendering.exe",
            ],
            [
                `../../target/debug/windowless-rendering-helper.exe`,
                "../../target/windowless_rendering/windowless-rendering-helper.exe",
            ],
            [`${cefOutDir}/Release`, "../../target/windowless_rendering/"],
            [`${cefOutDir}/Resources`, "../../target/windowless_rendering/"],
        ]) {
            await cp(...item, { force: true, recursive: true });
        }

        for (const path of [
            "../../target/windowless_rendering/cef_sandbox.lib",
            "../../target/windowless_rendering/libcef.lib",
        ]) {
            if (existsSync(path)) {
                await rm(path, { force: true, recursive: true });
            }
        }
    } else if (process.platform == "darwin") {
        const cefReleasePath = await getCrateOutdir(`../../target/debug`, "wew", "./cef/Release");

        for (const path of [
            "../../target/windowless_rendering/windowless-rendering.app",
            "../../target/windowless_rendering/windowless-rendering.app/Contents",
            "../../target/windowless_rendering/windowless-rendering.app/Contents/MacOS",
            "../../target/windowless_rendering/windowless-rendering.app/Contents/Frameworks",
        ]) {
            if (!existsSync(path)) {
                await mkdir(path);
            }
        }

        for (const item of [
            [
                "./Info.plist",
                "../../target/windowless_rendering/windowless-rendering.app/Contents/Info.plist",
            ],
            [
                `../../target/debug/windowless-rendering`,
                "../../target/windowless_rendering/windowless-rendering.app/Contents/MacOS/windowless-rendering",
            ],
            [
                join(cefReleasePath, "./Chromium Embedded Framework.framework"),
                "../../target/windowless_rendering/windowless-rendering.app/Contents/Frameworks/Chromium Embedded Framework.framework",
            ],
        ]) {
            await cp(...item, { force: true, recursive: true });
        }

        // generate helper
        {
            for (const [name, identifier] of [
                ["windowless-rendering Helper", "com.github.mycrl.wew.helper"],
                ["windowless-rendering Helper (GPU)", "com.github.mycrl.wew.helper.gpu"],
                ["windowless-rendering Helper (Plugin)", "com.github.mycrl.wew.helper.plugin"],
                ["windowless-rendering Helper (Renderer)", "com.github.mycrl.wew.helper.renderer"],
            ]) {
                const helperPath = join(
                    __dirname,
                    "../../target/windowless_rendering/windowless-rendering.app/Contents/Frameworks",
                    `./${name}.app`
                );

                for (const path of ["", "Contents", "Contents/MacOS", "Contents/Resources"]) {
                    if (!existsSync(join(helperPath, path))) {
                        await mkdir(join(helperPath, path));
                    }
                }

                {
                    await writeFile(
                        join(helperPath, "Contents/Info.plist"),
                        (await readFile("./helper.Info.plist", "utf8"))
                            .replace("{{CFBundleName}}", name)
                            .replace("{{CFBundleExecutable}}", name)
                            .replace("{{CFBundleIdentifier}}", identifier)
                    );
                }

                for (const item of [
                    [
                        `../../target/debug/windowless-rendering-helper`,
                        join(helperPath, `Contents/MacOS/${name}`),
                    ],
                ]) {
                    await cp(...item, { force: true, recursive: true });
                }

                await command(`install_name_tool -change \
                    "@executable_path/../Frameworks/Chromium Embedded Framework.framework/Chromium Embedded Framework" \
                    "@rpath/Chromium Embedded Framework.framework/Chromium Embedded Framework" \
                    "${join(helperPath, `Contents/MacOS/${name}`)}"`);

                await command(`install_name_tool \
                    -add_rpath "@executable_path/../../../../Frameworks" \
                    "${join(helperPath, `Contents/MacOS/${name}`)}"`);
            }
        }
    }
})();
