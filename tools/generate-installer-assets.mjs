import { mkdir, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { outputDirectory, projectRoot, renderTauriIcons } from './generate-quickpick-icons.mjs';
import { runNativeHelper } from './run-native-helper.mjs';

const installerRoot = join(projectRoot, 'apps', 'desktop', 'src-tauri', 'installer');
const assets = [
  { name: 'installer-header', width: 150, height: 57, x: 9, y: 8, iconSize: 40 },
  { name: 'installer-sidebar', width: 164, height: 314, x: 34, y: 42, iconSize: 96 },
];

export async function generateInstallerAssets(output = installerRoot) {
  const temporary = await mkdtemp(join(tmpdir(), 'quickpick-installer-'));
  try {
    const appSource = await readFile(join(projectRoot, 'apps', 'desktop', 'src-tauri', 'icons', 'source', 'quickpick-app-icon.svg'), 'utf8');
    await mkdir(output, { recursive: true });
    for (const asset of assets) {
      const source = await readFile(join(projectRoot, 'tools', 'assets', `${asset.name}.svg`), 'utf8');
      const icon = appSource.replace(/<svg\b[^>]*>/, `<svg xmlns="http://www.w3.org/2000/svg" x="${asset.x}" y="${asset.y}" width="${asset.iconSize}" height="${asset.iconSize}" viewBox="0 0 128 128">`);
      // The official icon command requires a square input. Preserve 1:1 geometry,
      // pad the short edge, then crop that padding while writing the final BMP.
      const size = Math.max(asset.width, asset.height);
      const square = source.replace(/<svg\b[^>]*>/, `<svg xmlns="http://www.w3.org/2000/svg" width="${size}" height="${size}" viewBox="0 0 ${size} ${size}">`).replace('<!-- QUICKPICK_ICON -->', icon);
      const sourcePath = join(temporary, `${asset.name}.svg`);
      const pngDirectory = join(temporary, asset.name);
      await writeFile(sourcePath, square);
      await renderTauriIcons(sourcePath, pngDirectory, [size]);
      const result = await runNativeHelper(['png-to-bmp', join(pngDirectory, `${size}x${size}.png`), join(output, `${asset.name}.bmp`), String(asset.width), String(asset.height)]);
      if (result.code !== 0) throw new Error(`Installer BMP conversion failed (${result.code}).`);
    }
    return output;
  } finally {
    await rm(temporary, { recursive: true, force: true });
  }
}

if (process.argv[1] && pathToFileURL(resolve(process.argv[1])).href === import.meta.url) {
  if (process.argv.slice(2).join(' ') === '--help') {
    console.log('Generate 150×57/164×314 24-bit NSIS BMPs. Usage: node tools/generate-installer-assets.mjs [--output-dir <directory>]');
  } else {
    try {
      console.log(`QuickPick installer assets generated in ${await generateInstallerAssets(outputDirectory(process.argv.slice(2), installerRoot))}`);
    } catch (error) {
      console.error(error.message);
      process.exitCode = 1;
    }
  }
}
