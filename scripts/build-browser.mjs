import { spawnSync } from "node:child_process";
import { mkdir } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import path from "node:path";

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const wasmBindgenVersion = "0.2.129";
const target = "wasm32-unknown-unknown";
const outputDirectory = path.join(repositoryRoot, "apps", "ide", "public", "wasm");

function run(command, args) {
  const result = spawnSync(command, args, {
    cwd: repositoryRoot,
    stdio: "inherit",
    shell: process.platform === "win32",
  });
  if (result.error) throw result.error;
  if (result.status !== 0) process.exit(result.status ?? 1);
}

function capture(command, args) {
  const result = spawnSync(command, args, {
    cwd: repositoryRoot,
    encoding: "utf8",
    shell: process.platform === "win32",
  });
  if (result.error) throw result.error;
  if (result.status !== 0) {
    const output = result.stderr.trim();
    if (command === "rustc") {
      throw new Error(`Rust 1.99.1 is required. Install rustup and the pinned toolchain from rust-toolchain.toml.${output ? `\n${output}` : ""}`);
    }
    if (command === "wasm-bindgen") {
      throw new Error(`wasm-bindgen CLI ${wasmBindgenVersion} is required. Install it with: cargo install wasm-bindgen-cli --version ${wasmBindgenVersion} --locked${output ? `\n${output}` : ""}`);
    }
    throw new Error(`Could not run ${command} ${args.join(" ")}.${output ? `\n${output}` : ""}`);
  }
  return result.stdout.trim();
}

const rustVersion = capture("rustc", ["--version"]);
if (!rustVersion.includes("1.99.1")) {
  throw new Error(`Expected Rust 1.99.1 from rust-toolchain.toml, got: ${rustVersion}`);
}

const bindgenVersion = capture("wasm-bindgen", ["--version"]);
if (bindgenVersion !== `wasm-bindgen ${wasmBindgenVersion}`) {
  throw new Error(`Expected wasm-bindgen ${wasmBindgenVersion}, got: ${bindgenVersion}. Install it with: cargo install wasm-bindgen-cli --version ${wasmBindgenVersion} --locked`);
}

console.log(`Building Xezim WASM for ${target} with ${rustVersion} and ${bindgenVersion}`);
run("rustup", ["target", "add", target]);
run("cargo", [
  "build", "--locked", "--release", "--manifest-path", "crates/xezim-web/Cargo.toml", "--target", target,
]);
await mkdir(outputDirectory, { recursive: true });
run("wasm-bindgen", ["--target", "web", "--out-dir", "apps/ide/public/wasm", "crates/xezim-web/target/wasm32-unknown-unknown/release/xezim_web.wasm"]);
console.log(`Browser assets written to ${path.relative(repositoryRoot, outputDirectory)}`);
