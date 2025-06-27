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
const Args = process.argv.slice(process.argv.indexOf("--") + 1).reduce(
    (ctx, item) => {
        if (typeof item != "string") {
            item = item.toString();
        }

        if (item.startsWith("--")) {
            ctx.last = item.replace("--", "");
        } else {
            ctx.args[ctx.last] = item;
        }

        return ctx;
    },
    {
        last: "",
        args: {},
    }
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
    const Project = Args.args.project;

    await command("cargo build", {
        cwd: `./${Project.replace("-", "_")}`,
        env: {
            ...process.env,
            MACOSX_DEPLOYMENT_TARGET: "15.4",
            CACHE_PATH: resolve(__dirname, "../target/cache"),
        },
    });

    if (!existsSync("../target/examples")) {
        await mkdir("../target/examples");
    }

    if (process.platform == "win32") {
        const cefOutDir = join(
            await getCrateOutdir(`../target/debug`, "wew", "./cef/Release"),
            "../"
        );

        if (!existsSync(`../target/examples/${Project}`)) {
            await mkdir(`../target/examples/${Project}`);
        }

        for (const item of [
            [`../target/debug/${Project}.exe`, `../target/examples/${Project}/${Project}.exe`],
            [
                `../target/debug/${Project}-helper.exe`,
                `../target/examples/${Project}/${Project}-helper.exe`,
            ],
            [`${cefOutDir}/Release`, `../target/examples/${Project}/`],
            [`${cefOutDir}/Resources`, `../target/examples/${Project}/`],
        ]) {
            await cp(...item, { force: true, recursive: true });
        }

        for (const path of [
            `../target/examples/${Project}/cef_sandbox.lib`,
            `../target/examples/${Project}/libcef.lib`,
        ]) {
            if (existsSync(path)) {
                await rm(path, { force: true, recursive: true });
            }
        }

        await command(`../target/examples/${Project}/${Project}.exe`);
    } else if (process.platform == "darwin") {
        const cefReleasePath = await getCrateOutdir(`../target/debug`, "wew", "./cef/Release");

        for (const path of [
            `../target/examples/${Project}.app`,
            `../target/examples/${Project}.app/Contents`,
            `../target/examples/${Project}.app/Contents/MacOS`,
            `../target/examples/${Project}.app/Contents/Frameworks`,
        ]) {
            if (!existsSync(path)) {
                await mkdir(path);
            }
        }

        for (const item of [
            [
                `../target/debug/${Project}`,
                `../target/examples/${Project}.app/Contents/MacOS/${Project}`,
            ],
            [
                join(cefReleasePath, "./Chromium Embedded Framework.framework"),
                `../target/examples/${Project}.app/Contents/Frameworks/Chromium Embedded Framework.framework`,
            ],
        ]) {
            await cp(...item, { force: true, recursive: true });
        }

        {
            await writeFile(
                `../target/examples/${Project}.app/Contents/Info.plist`,
                (await readFile("./Info.plist", "utf8"))
                    .replaceAll("{name}", Project)
                    .replaceAll("{identifier}", `com.github.mycrl.wew.${Project}`)
            );
        }

        // generate helper
        {
            for (const [name, identifier] of [
                [`${Project} Helper`, `com.github.mycrl.wew.${Project}.helper`],
                [`${Project} Helper (GPU)`, `com.github.mycrl.wew.${Project}.helper.gpu`],
                [`${Project} Helper (Plugin)`, `com.github.mycrl.wew.${Project}.helper.plugin`],
                [`${Project} Helper (Renderer)`, `com.github.mycrl.wew.${Project}.helper.renderer`],
            ]) {
                const helperPath = join(
                    __dirname,
                    `../target/examples/${Project}.app/Contents/Frameworks`,
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
                            .replaceAll("{name}", name)
                            .replaceAll("{identifier}", identifier)
                    );
                }

                for (const item of [
                    [
                        `../target/debug/${Project}-helper`,
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

        await command(`../target/examples/${Project}.app/Contents/MacOS/${Project}`);
    }
})();
