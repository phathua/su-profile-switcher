import fs from "node:fs";
import path from "node:path";

const input = "release-artifacts";
const output = "release-assets";
fs.mkdirSync(output, { recursive: true });

function files(dir) {
  return fs.readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const file = path.join(dir, entry.name);
    return entry.isDirectory() ? files(file) : [file];
  });
}

const all = files(input);
const installer = all.find((file) => /SuPS-x64-v[\d.]+-Setup\.exe$/.test(file));
if (!installer) throw new Error("Missing Windows x64 installer");
const version = path.basename(installer).match(/-v([\d.]+)-Setup\.exe$/)[1];
const tag = `v${version}`;
const repo = process.env.GITHUB_REPOSITORY;
if (!process.env.TAURI_SIGNING_PRIVATE_KEY) throw new Error("Missing updater signing key");
const canPublishMac = Boolean(process.env.APPLE_SIGNING_IDENTITY && process.env.APPLE_ID && process.env.APPLE_PASSWORD && process.env.APPLE_TEAM_ID);
const platforms = {};

const targets = [
  ["x64", "windows-x86_64", "Setup.nsis.zip"],
  ["x86", "windows-i686", "Setup.nsis.zip"],
  ["arm64", "windows-aarch64", "Setup.nsis.zip"],
  ["macos-x64", "darwin-x86_64", "app.tar.gz"],
  ["macos-arm64", "darwin-aarch64", "app.tar.gz"],
];

for (const [arch, platform, suffix] of targets) {
  if (arch.startsWith("macos-") && !canPublishMac) continue;
  const updateName = arch.startsWith("macos-")
    ? `SuPS-${arch}-${tag}.${suffix}`
    : `SuPS-${arch}-${tag}-${suffix}`;
  const updateFile = all.find((file) => path.basename(file) === updateName);
  const signature = all.find((file) => path.basename(file) === `${updateName}.sig`);
  if (!updateFile || !signature) throw new Error(`Missing signed updater for ${platform}`);
  platforms[platform] = {
    signature: fs.readFileSync(signature, "utf8").trim(),
    url: `https://github.com/${repo}/releases/download/${tag}/${updateName}`,
  };
  fs.copyFileSync(updateFile, path.join(output, updateName));
  fs.copyFileSync(signature, path.join(output, `${updateName}.sig`));
  const downloadName = arch.startsWith("macos-")
    ? `SuPS-${arch}-${tag}.dmg`
    : `SuPS-${arch}-${tag}-Setup.exe`;
  const downloadFile = all.find((file) => path.basename(file) === downloadName);
  if (!downloadFile) throw new Error(`Missing installer for ${platform}`);
  fs.copyFileSync(downloadFile, path.join(output, downloadName));
}

if (Object.keys(platforms).length) {
  fs.writeFileSync(path.join(output, "latest.json"), JSON.stringify({ version, platforms }, null, 2));
}
fs.appendFileSync(process.env.GITHUB_ENV, `RELEASE_TAG=${tag}\n`);
