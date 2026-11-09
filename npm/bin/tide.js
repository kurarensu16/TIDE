#!/usr/bin/env node

const path = require('path');
const fs = require('fs');
const { spawnSync } = require('child_process');
const { install, getPlatformArchive } = require('../install');

async function main() {
  const target = getPlatformArchive();
  if (!target) {
    console.error(`Unsupported platform: ${process.platform} (${process.arch})`);
    process.exit(1);
  }

  const binPath = path.join(__dirname, target.binName);

  if (!fs.existsSync(binPath)) {
    console.log(`[tide-cli] Native binary not found. Downloading on first run...`);
    const installed = await install();
    if (!installed || !fs.existsSync(binPath)) {
      console.error(`[tide-cli] Failed to initialize native binary. Please check network or download manually from GitHub Releases.`);
      process.exit(1);
    }
  }

  const result = spawnSync(binPath, process.argv.slice(2), {
    stdio: 'inherit',
    windowsHide: false,
  });

  if (result.error) {
    console.error(`[tide-cli] Failed to launch binary: ${result.error.message}`);
    process.exit(1);
  }

  process.exit(result.status ?? 0);
}

main().catch((err) => {
  console.error(`[tide-cli] ${err.message}`);
  process.exit(1);
});

