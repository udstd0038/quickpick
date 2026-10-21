import assert from 'node:assert/strict';
import { mkdtemp, readFile, readdir, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { inflateSync } from 'node:zlib';
import test from 'node:test';
import { generateIcons, outputDirectory, projectRoot } from './generate-quickpick-icons.mjs';
import { generateInstallerAssets } from './generate-installer-assets.mjs';

// Inspect actual decoded pixels, not only signatures/file existence. This small
// decoder is intentionally limited to the non-interlaced RGBA8 PNGs the CLI emits.
function png(bytes) {
  assert.deepEqual(bytes.subarray(0, 8), Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]));
  const width = bytes.readUInt32BE(16);
  const height = bytes.readUInt32BE(20);
  assert.equal(bytes[24], 8);
  assert.equal(bytes[25], 6);
  assert.equal(bytes[28], 0);
  const idat = [];
  for (let offset = 8; offset < bytes.length;) {
    const length = bytes.readUInt32BE(offset);
    assert.ok(offset + length + 12 <= bytes.length);
    if (bytes.toString('ascii', offset + 4, offset + 8) === 'IDAT') idat.push(bytes.subarray(offset + 8, offset + 8 + length));
    offset += length + 12;
  }
  const stride = width * 4;
  const encoded = inflateSync(Buffer.concat(idat));
  assert.equal(encoded.length, (stride + 1) * height);
  const pixels = Buffer.alloc(stride * height);
  for (let y = 0; y < height; y++) {
    const type = encoded[y * (stride + 1)];
    assert.ok(type <= 4);
    for (let x = 0; x < stride; x++) {
      const left = x >= 4 ? pixels[y * stride + x - 4] : 0;
      const up = y ? pixels[(y - 1) * stride + x] : 0;
      const upperLeft = y && x >= 4 ? pixels[(y - 1) * stride + x - 4] : 0;
      let predictor = 0;
      if (type === 1) predictor = left;
      if (type === 2) predictor = up;
      if (type === 3) predictor = Math.floor((left + up) / 2);
      if (type === 4) {
        const p = left + up - upperLeft;
        const distances = [Math.abs(p - left), Math.abs(p - up), Math.abs(p - upperLeft)];
        predictor = distances[0] <= distances[1] && distances[0] <= distances[2] ? left : distances[1] <= distances[2] ? up : upperLeft;
      }
      pixels[y * stride + x] = (encoded[y * (stride + 1) + 1 + x] + predictor) & 255;
    }
  }
  return { width, height, pixel: (x, y) => [...pixels.subarray((y * width + x) * 4, (y * width + x) * 4 + 4)], pixels };
}

function bmp(bytes, width, height) {
  const stride = (width * 3 + 3) & ~3;
  assert.equal(bytes.toString('ascii', 0, 2), 'BM');
  assert.equal(bytes.readUInt32LE(2), bytes.length);
  assert.equal(bytes.readUInt32LE(10), 54);
  assert.equal(bytes.readUInt32LE(14), 40);
  assert.equal(bytes.readInt32LE(18), width);
  assert.equal(bytes.readInt32LE(22), height);
  assert.equal(bytes.readUInt16LE(26), 1);
  assert.equal(bytes.readUInt16LE(28), 24);
  assert.equal(bytes.readUInt32LE(30), 0);
  assert.equal(bytes.readUInt32LE(34), stride * height);
  assert.equal(bytes.length, 54 + stride * height);
  for (let y = 0; y < height; y++) {
    assert.ok(bytes.subarray(54 + y * stride + width * 3, 54 + (y + 1) * stride).every((byte) => byte === 0));
  }
  return { pixel(x, y) { const offset = 54 + (height - y - 1) * stride + x * 3; return [bytes[offset + 2], bytes[offset + 1], bytes[offset]]; } };
}

test('output directory rejects unknown or incomplete arguments', () => {
  assert.equal(outputDirectory([], '/example/default'), '/example/default');
  assert.equal(outputDirectory(['--output-dir', '/tmp/new folder'], '/example/default'), '/tmp/new folder');
  for (const args of [['--output-dir'], ['--output-dir', ''], ['--wrong', '/tmp/test'], ['--output-dir', '/tmp/test', 'extra']]) {
    assert.throws(() => outputDirectory(args, '/example/default'));
  }
});

test('actual icon generation preserves app, transparent white tray, 9 ICO sizes and ICNS', async () => {
  const output = await mkdtemp(join(tmpdir(), 'quickpick-icon-test-'));
  try {
    await generateIcons(output);
    const app = png(await readFile(join(output, 'icon.png')));
    assert.deepEqual([app.width, app.height], [512, 512]);
    assert.ok(app.pixel(0, 0)[3] < 40, 'rounded icon corners remain transparent');
    assert.ok(app.pixel(256, 256)[3] > 250);
    assert.ok(app.pixel(256, 60)[0] > app.pixel(256, 460)[0] + 30, 'lavender top and blue base remain distinct');
    assert.ok(app.pixel(256, 205).slice(0, 3).every((channel) => channel > 245), 'white text line is rendered');
    const tray = png(await readFile(join(output, 'tray-icon.png')));
    assert.deepEqual([tray.width, tray.height], [64, 64]);
    const alphas = [];
    for (let i = 0; i < tray.pixels.length; i += 4) {
      alphas.push(tray.pixels[i + 3]);
      if (tray.pixels[i + 3]) assert.ok([...tray.pixels.subarray(i, i + 3)].every((channel) => channel === 255));
    }
    assert.ok(alphas.some((alpha) => alpha === 0));
    assert.ok(alphas.some((alpha) => alpha === 255));
    assert.ok(tray.pixel(6, 6)[3] > 200, 'large corner brackets retained');
    assert.ok(tray.pixel(25, 21)[3] > 200, 'text strokes retained');
    const ico = await readFile(join(output, 'icon.ico'));
    assert.equal(ico.readUInt16LE(0), 0);
    assert.equal(ico.readUInt16LE(2), 1);
    assert.equal(ico.readUInt16LE(4), 9);
    const sizes = [16, 20, 24, 32, 40, 48, 64, 128, 256];
    let end = 6 + 16 * sizes.length;
    sizes.forEach((size, index) => {
      const entry = 6 + index * 16;
      assert.equal(ico[entry] || 256, size);
      assert.equal(ico[entry + 1] || 256, size);
      assert.equal(ico.readUInt16LE(entry + 6), 32);
      const length = ico.readUInt32LE(entry + 8);
      assert.equal(ico.readUInt32LE(entry + 12), end);
      const image = png(ico.subarray(end, end + length));
      assert.deepEqual([image.width, image.height], [size, size]);
      end += length;
    });
    assert.equal(end, ico.length);
    const icns = await readFile(join(output, 'icon.icns'));
    assert.equal(icns.toString('ascii', 0, 4), 'icns');
    assert.equal(icns.readUInt32BE(4), icns.length);
    const representations = new Set();
    for (let offset = 8; offset < icns.length;) {
      representations.add(icns.toString('ascii', offset, offset + 4));
      const length = icns.readUInt32BE(offset + 4);
      assert.ok(length >= 8 && offset + length <= icns.length);
      offset += length;
      if (offset === icns.length) break;
    }
    assert.ok(representations.has('ic09'), '512 px macOS representation included');
    assert.ok(representations.has('ic10'), '1024 px Retina representation included');
    assert.deepEqual((await readdir(output)).sort(), ['128x128.png', '128x128@2x.png', '32x32.png', '64x64.png', 'icon.icns', 'icon.ico', 'icon.png', 'tray-icon.png'].sort());
  } finally {
    await rm(output, { recursive: true, force: true });
  }
});

test('actual installer generation keeps exact NSIS BMP dimensions and renders icons and outlined lettering', async () => {
  const output = await mkdtemp(join(tmpdir(), 'quickpick-installer-test-'));
  try {
    await generateInstallerAssets(output);
    const header = bmp(await readFile(join(output, 'installer-header.bmp')), 150, 57);
    const sidebar = bmp(await readFile(join(output, 'installer-sidebar.bmp')), 164, 314);
    assert.ok(header.pixel(0, 0)[0] > 240);
    assert.ok(header.pixel(25, 40)[2] > header.pixel(25, 40)[0] + 30, 'colored header icon rendered');
    assert.ok(sidebar.pixel(0, 0)[0] < sidebar.pixel(0, 313)[0] - 100, 'full-height blue/lavender gradient survives square padding and crop');
    assert.ok(sidebar.pixel(70, 122)[2] > sidebar.pixel(70, 122)[0] + 30, 'colored sidebar icon rendered');
    let headerLetters = 0;
    let sidebarLetters = 0;
    for (let y = 10; y < 43; y++) for (let x = 57; x < 147; x++) {
      const [r, g, b] = header.pixel(x, y);
      if (r < 130 && g < 155 && b < 205) headerLetters++;
    }
    for (let y = 158; y < 178; y++) for (let x = 35; x < 129; x++) {
      if (sidebar.pixel(x, y).every((channel) => channel > 245)) sidebarLetters++;
    }
    assert.ok(headerLetters > 200, 'title and subtitle contain visible dark paths');
    assert.ok(sidebarLetters > 200, 'white title paths rendered');
    for (const name of ['installer-header', 'installer-sidebar']) {
      const source = await readFile(join(projectRoot, 'tools', 'assets', `${name}.svg`), 'utf8');
      assert.ok(source.includes('aria-label="QuickPick"'));
      assert.ok(source.includes('aria-label="AI Selection &amp; Translation"'));
      assert.ok(!/<text\b|font-family|https?:\/\/(?!www\.w3\.org)/.test(source), 'self-contained glyph paths require no fonts or network');
    }
    assert.deepEqual((await readdir(output)).sort(), ['installer-header.bmp', 'installer-sidebar.bmp']);
  } finally {
    await rm(output, { recursive: true, force: true });
  }
});
