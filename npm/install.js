const fs = require('fs');
const path = require('path');
const https = require('https');
const { execSync } = require('child_process');

const REPO = 'kurarensu16/TIDE';
const pkg = require('./package.json');
const VERSION = pkg.version;

function getPlatformArchive() {
  const platform = process.platform;
  const arch = process.arch;

  if (platform === 'win32' && arch === 'x64') {
    return {
      archive: `tide-v${VERSION}-windows-x86_64.zip`,
      binName: 'tide.exe',
      isZip: true,
    };
  }

  if (platform === 'darwin' && arch === 'arm64') {
    return {
      archive: `tide-v${VERSION}-macos-arm64.tar.gz`,
      binName: 'tide',
      isZip: false,
    };
  }

  if (platform === 'darwin' && arch === 'x64') {
    return {
      archive: `tide-v${VERSION}-macos-x86_64.tar.gz`,
      binName: 'tide',
      isZip: false,
    };
  }

  if (platform === 'linux' && arch === 'x64') {
    return {
      archive: `tide-v${VERSION}-linux-x86_64.tar.gz`,
      binName: 'tide',
      isZip: false,
    };
  }

  return null;
}

function downloadFile(url, dest) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(dest);

    const get = (currentUrl) => {
      https.get(currentUrl, { headers: { 'User-Agent': 'tide-npm-installer' } }, (res) => {
        if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
          return get(res.headers.location);
        }

        if (res.statusCode !== 200) {
          return reject(new Error(`Download failed with status ${res.statusCode}: ${currentUrl}`));
        }

        res.pipe(file);
        file.on('finish', () => {
          file.close(() => resolve(dest));
        });
      }).on('error', (err) => {
        fs.unlink(dest, () => {});
        reject(err);
      });
    };

    get(url);
  });
}

async function install() {
  const target = getPlatformArchive();
  if (!target) {
    console.warn(`[tide-cli] Pre-compiled binary not available for ${process.platform} (${process.arch}).`);
    console.warn(`[tide-cli] You can build from source: cargo install --git https://github.com/${REPO}.git`);
    return null;
  }

  const binDir = path.join(__dirname, 'bin');
  if (!fs.existsSync(binDir)) {
    fs.mkdirSync(binDir, { recursive: true });
  }

  const targetBinPath = path.join(binDir, target.binName);
  if (fs.existsSync(targetBinPath)) {
    return targetBinPath;
  }

  const downloadUrl = `https://github.com/${REPO}/releases/download/v${VERSION}/${target.archive}`;
  const tmpArchive = path.join(binDir, target.archive);

  console.log(`[tide-cli] Downloading native binary from GitHub Releases...`);
  console.log(`[tide-cli] URL: ${downloadUrl}`);

  try {
    await downloadFile(downloadUrl, tmpArchive);

    console.log(`[tide-cli] Extracting ${target.archive}...`);
    if (target.isZip) {
      // Use built-in tar (available in modern Windows) or PowerShell Expand-Archive
      try {
        execSync(`tar -xf "${tmpArchive}" -C "${binDir}"`, { stdio: 'ignore' });
      } catch (_) {
        execSync(`powershell -NoProfile -Command "Expand-Archive -Path '${tmpArchive}' -DestinationPath '${binDir}' -Force"`, { stdio: 'inherit' });
      }
    } else {
      execSync(`tar -xzf "${tmpArchive}" -C "${binDir}"`, { stdio: 'inherit' });
    }

    // Clean up archive
    if (fs.existsSync(tmpArchive)) {
      fs.unlinkSync(tmpArchive);
    }

    // Ensure executable permissions on Unix
    if (process.platform !== 'win32') {
      fs.chmodSync(targetBinPath, 0o755);
    }

    console.log(`[tide-cli] Successfully installed TIDE binary to: ${targetBinPath}`);
    return targetBinPath;
  } catch (err) {
    console.error(`[tide-cli] Installation warning: ${err.message}`);
    if (fs.existsSync(tmpArchive)) {
      fs.unlinkSync(tmpArchive);
    }
    return null;
  }
}

if (require.main === module) {
  install().catch((err) => {
    console.error(`[tide-cli] Error: ${err.message}`);
  });
}

module.exports = { install, getPlatformArchive };

