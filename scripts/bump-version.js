import fs from 'fs';
import path from 'path';
import { execSync } from 'child_process';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// 1. Lay so luong commit tu Git
let commitCount = 0;
try {
  commitCount = parseInt(execSync('git rev-list --count HEAD', { stdio: ['pipe', 'pipe', 'ignore'] }).toString().trim(), 10);
} catch (error) {
  console.error('Khong the doc so luong commit tu Git:', error.message);
  process.exit(1);
}

// 2. Tinh toan phien ban theo cong thuc dong bo
const major = 1 + Math.floor(commitCount / 100);
const minor = Math.floor((commitCount % 100) / 10);
const patch = commitCount % 10;
const newVersion = `${major}.${minor}.${patch}`;

console.log(`[Version Bumper] So commit: ${commitCount} -> Phien ban tinh toan: ${newVersion}`);

const rootDir = path.resolve(__dirname, '..');

// 3. Cap nhat package.json
const packageJsonPath = path.join(rootDir, 'package.json');
if (fs.existsSync(packageJsonPath)) {
  try {
    const pkg = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));
    if (pkg.version !== newVersion) {
      pkg.version = newVersion;
      fs.writeFileSync(packageJsonPath, JSON.stringify(pkg, null, 2) + '\n');
      console.log(`[Version Bumper] Da cap nhat package.json sang phien ban: ${newVersion}`);
    } else {
      console.log(`[Version Bumper] Phien ban trong package.json da khop: ${newVersion}`);
    }
  } catch (err) {
    console.error('Loi khi cap nhat package.json:', err.message);
  }
}

// 4. Cap nhat tauri.conf.json
const tauriConfPath = path.join(rootDir, 'src-tauri', 'tauri.conf.json');
if (fs.existsSync(tauriConfPath)) {
  try {
    const conf = JSON.parse(fs.readFileSync(tauriConfPath, 'utf8'));
    if (conf.version !== newVersion) {
      conf.version = newVersion;
      fs.writeFileSync(tauriConfPath, JSON.stringify(conf, null, 2) + '\n');
      console.log(`[Version Bumper] Da cap nhat tauri.conf.json sang phien ban: ${newVersion}`);
    } else {
      console.log(`[Version Bumper] Phien ban trong tauri.conf.json da khop: ${newVersion}`);
    }
  } catch (err) {
    console.error('Loi khi cap nhat tauri.conf.json:', err.message);
  }
}

// 5. Cap nhat Cargo.toml
const cargoTomlPath = path.join(rootDir, 'src-tauri', 'Cargo.toml');
if (fs.existsSync(cargoTomlPath)) {
  try {
    let cargoContent = fs.readFileSync(cargoTomlPath, 'utf8');
    const versionRegex = /^version\s*=\s*".*"/m;
    if (versionRegex.test(cargoContent)) {
      cargoContent = cargoContent.replace(versionRegex, `version = "${newVersion}"`);
      fs.writeFileSync(cargoTomlPath, cargoContent);
      console.log(`[Version Bumper] Da cap nhat Cargo.toml sang phien ban: ${newVersion}`);
    } else {
      console.warn('[Version Bumper] Khong tim thay truong version trong Cargo.toml de thay the.');
    }
  } catch (err) {
    console.error('Loi khi cap nhat Cargo.toml:', err.message);
  }
}

// 6. Cap nhat src/lib/version.js
const versionJsPath = path.join(rootDir, 'src', 'lib', 'version.js');
try {
  fs.writeFileSync(versionJsPath, `export const APP_VERSION = "${newVersion}";\n`);
  console.log(`[Version Bumper] Da cap nhat src/lib/version.js sang phien ban: ${newVersion}`);
} catch (err) {
  console.error('Loi khi cap nhat version.js:', err.message);
}

