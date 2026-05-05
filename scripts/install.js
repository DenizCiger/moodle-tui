#!/usr/bin/env node

const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const https = require("node:https");
const { pipeline } = require("node:stream/promises");
const { spawnSync } = require("node:child_process");

const pkg = require("../package.json");

const OWNER = "DenizCiger";
const REPO = "moodle-tui";
const VERSION = pkg.version;
const TAG = `v${VERSION}`;
const ARTIFACTS = {
  "darwin-arm64": { asset: "moodle-aarch64-apple-darwin.tar.gz", binary: "moodle" },
  "darwin-x64": { asset: "moodle-x86_64-apple-darwin.tar.gz", binary: "moodle" },
  "linux-x64": { asset: "moodle-x86_64-unknown-linux-gnu.tar.gz", binary: "moodle" },
  "win32-x64": { asset: "moodle-x86_64-pc-windows-msvc.zip", binary: "moodle.exe" }
};

async function main() {
  if (process.env.MOODLE_SKIP_DOWNLOAD === "1") {
    return;
  }

  const key = `${process.platform}-${process.arch}`;
  const artifact = ARTIFACTS[key];

  if (!artifact) {
    console.error(
      `Unsupported platform: ${process.platform} ${process.arch}. Supported targets: ${Object.keys(ARTIFACTS).join(", ")}`
    );
    process.exit(1);
  }

  const installDir = path.join(__dirname, "native");
  fs.mkdirSync(installDir, { recursive: true });

  const downloadUrl = `https://github.com/${OWNER}/${REPO}/releases/download/${TAG}/${artifact.asset}`;
  const archivePath = path.join(os.tmpdir(), artifact.asset);

  try {
    await download(downloadUrl, archivePath);
    await extractArchive(archivePath, installDir, artifact.asset);
    const binaryPath = path.join(installDir, artifact.binary);
    if (!fs.existsSync(binaryPath)) {
      throw new Error(`Expected binary was not found after extraction: ${artifact.binary}`);
    }
    if (process.platform !== "win32") {
      fs.chmodSync(binaryPath, 0o755);
    }
  } catch (error) {
    console.error(`Failed to install moodle-tui ${VERSION} from ${downloadUrl}`);
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  } finally {
    fs.rmSync(archivePath, { force: true });
  }
}

function download(url, destination) {
  return new Promise((resolve, reject) => {
    const request = https.get(
      url,
      { headers: { "User-Agent": `${pkg.name}/${pkg.version}` } },
      async (response) => {
        if (
          response.statusCode &&
          response.statusCode >= 300 &&
          response.statusCode < 400 &&
          response.headers.location
        ) {
          response.resume();
          try {
            await download(response.headers.location, destination);
            resolve();
          } catch (error) {
            reject(error);
          }
          return;
        }
        if (response.statusCode !== 200) {
          response.resume();
          reject(new Error(`Download failed with status ${response.statusCode}`));
          return;
        }
        try {
          await pipeline(response, fs.createWriteStream(destination));
          resolve();
        } catch (error) {
          reject(error);
        }
      }
    );
    request.on("error", reject);
  });
}

async function extractArchive(archivePath, installDir, assetName) {
  fs.rmSync(installDir, { recursive: true, force: true });
  fs.mkdirSync(installDir, { recursive: true });

  if (assetName.endsWith(".tar.gz")) {
    const tar = spawnSync("tar", ["-xzf", archivePath, "-C", installDir], { stdio: "inherit" });
    if (tar.status !== 0) {
      throw new Error("Failed to extract tar.gz archive. A working `tar` executable is required.");
    }
    return;
  }

  if (assetName.endsWith(".zip")) {
    const result = spawnSync(
      "powershell.exe",
      [
        "-NoProfile",
        "-Command",
        `Expand-Archive -LiteralPath '${archivePath.replace(/'/g, "''")}' -DestinationPath '${installDir.replace(/'/g, "''")}' -Force`
      ],
      { stdio: "inherit" }
    );
    if (result.status !== 0) {
      throw new Error("Failed to extract Windows archive");
    }
    return;
  }

  throw new Error(`Unsupported archive format for ${assetName}`);
}

main();
