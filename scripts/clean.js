import fs from "node:fs";
import path from "node:path";
import { execSync } from "node:child_process";

const rootDir = process.cwd();

console.log("Cleaning build artifacts & caches...");

// 1. Clean Vite / Frontend build artifacts
const dirsToClean = [
  path.join(rootDir, "dist"),
  path.join(rootDir, "node_modules", ".vite"),
  path.join(rootDir, ".svelte-kit"),
];

for (const dir of dirsToClean) {
  if (fs.existsSync(dir)) {
    try {
      fs.rmSync(dir, { recursive: true, force: true });
      console.log(`- Removed: ${path.relative(rootDir, dir)}`);
    } catch (err) {
      console.warn(`- Could not remove ${dir}: ${err.message}`);
    }
  }
}

// 2. Clean Cargo build artifacts
const tauriDir = path.join(rootDir, "src-tauri");
if (fs.existsSync(tauriDir)) {
  try {
    // Check if background binaries are currently running
    try {
      execSync('powershell -Command "Stop-Process -Name sups,su-profile-switcher -Force -ErrorAction SilentlyContinue"', { stdio: "ignore" });
    } catch {}

    console.log("- Running cargo clean...");
    execSync("cargo clean", { cwd: tauriDir, stdio: "inherit" });
  } catch (err) {
    console.warn(`- Cargo clean warning: ${err.message}`);
  }
}

console.log("Clean completed successfully!");
