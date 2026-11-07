import { existsSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("..", import.meta.url));

function readVersion(packageJsonPath) {
  const pkg = JSON.parse(readFileSync(packageJsonPath, "utf8"));
  if (typeof pkg.version !== "string" || pkg.version.length === 0) {
    throw new Error(`${packageJsonPath} has no version`);
  }
  return pkg.version;
}

// Stop at the next table so a missing package version is an error, not a dependency rewrite.
function setCargoPackageVersion(cargoPath, version) {
  const text = readFileSync(cargoPath, "utf8");
  const pattern = /(\[package\][^\[]*?^version = ")[^"]+(")/m;
  if (!pattern.test(text)) {
    throw new Error(`no [package] version in ${cargoPath}`);
  }
  const next = text.replace(pattern, `$1${version}$2`);
  if (next !== text) {
    writeFileSync(cargoPath, next);
  }
}

const cratesDir = join(root, "crates");
for (const entry of readdirSync(cratesDir, { withFileTypes: true })) {
  if (!entry.isDirectory()) continue;
  const dir = join(cratesDir, entry.name);
  const cargoPath = join(dir, "Cargo.toml");
  if (!existsSync(cargoPath)) continue;
  const packageJsonPath = join(dir, "package.json");
  if (!existsSync(packageJsonPath)) {
    throw new Error(`${dir} has a Cargo.toml and no package.json`);
  }
  setCargoPackageVersion(cargoPath, readVersion(packageJsonPath));
}

setCargoPackageVersion(
  join(root, "apps", "nakama-desktop", "src-tauri", "Cargo.toml"),
  readVersion(join(root, "apps", "nakama-desktop", "package.json")),
);
