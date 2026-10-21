import { spawn } from 'node:child_process';
import { copyFile, mkdir, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { createRequire } from 'node:module';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

export const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const iconRoot = join(projectRoot, 'apps', 'desktop', 'src-tauri', 'icons');
const icoSizes = [16, 20, 24, 32, 40, 48, 64, 128, 256];

// Resolve the installed CLI JavaScript entry, never a platform-specific .cmd shim.
export async function renderTauriIcons(source, output, sizes = []) {
  const require = createRequire(join(projectRoot, 'apps', 'desktop', 'package.json'));
  const entry = require.resolve('@tauri-apps/cli/tauri.js');
  const args = [entry, 'icon', source, '--output', output];
  for (const size of sizes) args.push('--png', String(size));
  await new Promise((resolveRun, reject) => {
    const child = spawn(process.execPath, args, { cwd: projectRoot, stdio: 'inherit', shell: false });
    child.once('error', reject);
    child.once('exit', (code, signal) => code === 0 ? resolveRun() : reject(new Error(`Tauri icon generation failed (${signal ?? code}).`)));
  });
}

export function outputDirectory(args, fallback) {
  if (args.length === 0) return fallback;
  if (args.length === 2 && args[0] === '--output-dir' && args[1].trim()) return resolve(args[1]);
  throw new Error('Usage: --output-dir <directory> (optional).');
}

// PNG payloads are supported by the ICO format, including the 256 px entry.
function encodeIco(images) {
  const directory = Buffer.alloc(6 + images.length * 16);
  directory.writeUInt16LE(1, 2);
  directory.writeUInt16LE(images.length, 4);
  let offset = directory.length;
  images.forEach(({ size, bytes }, index) => {
    const entry = 6 + index * 16;
    directory[entry] = size === 256 ? 0 : size;
    directory[entry + 1] = directory[entry];
    directory.writeUInt16LE(1, entry + 4);
    directory.writeUInt16LE(32, entry + 6);
    directory.writeUInt32LE(bytes.length, entry + 8);
    directory.writeUInt32LE(offset, entry + 12);
    offset += bytes.length;
  });
  return Buffer.concat([directory, ...images.map(({ bytes }) => bytes)]);
}

export async function generateIcons(output = iconRoot) {
  const temporary = await mkdtemp(join(tmpdir(), 'quickpick-icons-'));
  try {
    const app = join(iconRoot, 'source', 'quickpick-app-icon.svg');
    const generated = join(temporary, 'app');
    const ico = join(temporary, 'ico');
    const tray = join(temporary, 'tray');
    await renderTauriIcons(app, generated);
    await renderTauriIcons(app, ico, icoSizes);
    await renderTauriIcons(join(iconRoot, 'source', 'quickpick-tray-icon.svg'), tray, [64]);
    const images = await Promise.all(icoSizes.map(async (size) => ({ size, bytes: await readFile(join(ico, `${size}x${size}.png`)) })));
    await mkdir(output, { recursive: true });
    // Publish only desktop assets; temporary mobile/Appx output is discarded.
    for (const name of ['icon.png', 'icon.icns', '32x32.png', '64x64.png', '128x128.png', '128x128@2x.png']) {
      await copyFile(join(generated, name), join(output, name));
    }
    await writeFile(join(output, 'icon.ico'), encodeIco(images));
    await copyFile(join(tray, '64x64.png'), join(output, 'tray-icon.png'));
    return output;
  } finally {
    await rm(temporary, { recursive: true, force: true });
  }
}

if (process.argv[1] && pathToFileURL(resolve(process.argv[1])).href === import.meta.url) {
  if (process.argv.slice(2).join(' ') === '--help') {
    console.log('Generate QuickPick PNG/ICO/ICNS and 64 px tray assets. Usage: node tools/generate-quickpick-icons.mjs [--output-dir <directory>]');
  } else {
    try {
      console.log(`QuickPick icons generated in ${await generateIcons(outputDirectory(process.argv.slice(2), iconRoot))}`);
    } catch (error) {
      console.error(error.message);
      process.exitCode = 1;
    }
  }
}
