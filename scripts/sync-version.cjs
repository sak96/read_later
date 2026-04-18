const fs = require("fs");
const path = require("path");
const { execSync } = require("child_process");

// Read package.json
const pkg = JSON.parse(fs.readFileSync("package.json", "utf-8"));
const newVersion = pkg.version;

// --- Update tauri.conf.json ---
const tauriPath = path.join("src-tauri", "tauri.conf.json");
const tauri = JSON.parse(fs.readFileSync(tauriPath, "utf-8"));

tauri.version = newVersion;

fs.writeFileSync(tauriPath, JSON.stringify(tauri, null, 2));

// --- Update Cargo.toml ---
const cargoPath = path.join("src-tauri", "Cargo.toml");
let cargoContent = fs.readFileSync(cargoPath, "utf-8");

// Replace version = "x.x.x"
cargoContent = cargoContent.replace(
  /version\s*=\s*"[^\"]+"/,
  `version = "${newVersion}"`
);

fs.writeFileSync(cargoPath, cargoContent);

try {
  // Run Rust check
  execSync("cd src-tauri && cargo check", { stdio: "inherit" });
  console.log("🦀 cargo check passed");
} catch (err) {
  console.error("❌ Process failed:", err.message);
  process.exit(1);
}

const files = [
  "src-tauri/Cargo.toml",
  "src-tauri/Cargo.lock",
  "src-tauri/tauri.conf.json"
];

// Git add cargo files
execSync(
  "git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/tauri.conf.json",
  { stdio: "inherit" }
);

for (const file of files) {
  try {
    execSync(`git add ${file}`, { stdio: "inherit" });
    console.log(`✅ staged: ${file}`);
  } catch (err) {
    console.log(`❌ failed: ${file}`);
    process.exit(1);
  }
}

console.log("📦 Git staged Cargo.toml, Cargo.lock, tauri.conf.json");

console.log(`✅ Synced version ${newVersion} across Tauri + Cargo`);
