import { mkdir, copyFile, readFile, writeFile } from "node:fs/promises";
import toIco from "to-ico";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { Jimp } from "jimp";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, "..");
const logoSrc = path.join(root, "share/brand/intent-kernel-logo.png");
const iconsDir = path.join(root, "shell/tauri-app/src-tauri/icons");
const publicDir = path.join(root, "shell/tauri-app/public");
const brandShare = path.join(root, "share/brand");

async function writeIcon(image, size, dest) {
  const icon = image.clone().cover({ w: size, h: size });
  await icon.write(dest);
}

async function writeDarkHero(image, dest) {
  const hero = image.clone().resize({ w: 512, h: 512 });
  const canvas = new Jimp({ width: 512, height: 512, color: 0x0b1020ff });
  canvas.composite(hero, 0, 0);
  await canvas.write(dest);
}

async function main() {
  await mkdir(iconsDir, { recursive: true });
  await mkdir(publicDir, { recursive: true });
  await mkdir(brandShare, { recursive: true });

  const logo = await Jimp.read(logoSrc);
  const mark = logo.clone().crop({
    x: Math.floor(logo.bitmap.width * 0.18),
    y: Math.floor(logo.bitmap.height * 0.08),
    w: Math.floor(logo.bitmap.width * 0.64),
    h: Math.floor(logo.bitmap.height * 0.55),
  });

  await writeIcon(mark, 32, path.join(iconsDir, "32x32.png"));
  await writeIcon(mark, 128, path.join(iconsDir, "128x128.png"));
  await writeIcon(mark, 256, path.join(iconsDir, "128x128@2x.png"));
  await copyFile(path.join(iconsDir, "128x128.png"), path.join(iconsDir, "icon.png"));

  const ico = await toIco([
    await readFile(path.join(iconsDir, "32x32.png")),
    await readFile(path.join(iconsDir, "128x128.png")),
  ]);
  await writeFile(path.join(iconsDir, "icon.ico"), ico);
  await copyFile(path.join(iconsDir, "128x128@2x.png"), path.join(iconsDir, "icon.icns"));

  await writeDarkHero(logo, path.join(brandShare, "intent-kernel-logo-dark.png"));
  await copyFile(logoSrc, path.join(publicDir, "logo.png"));
  await copyFile(
    path.join(brandShare, "intent-kernel-logo-dark.png"),
    path.join(publicDir, "logo-dark.png"),
  );

  console.log("Brand assets generated:");
  console.log(`  icons: ${iconsDir}`);
  console.log(`  public: ${publicDir}`);
  console.log(`  dark hero: ${path.join(brandShare, "intent-kernel-logo-dark.png")}`);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});