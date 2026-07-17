const fs = require("fs");
const path = require("path");
const { spawnSync } = require("child_process");

const rootDir = path.resolve(__dirname, "..");
const tauriDir = path.join(rootDir, "src-tauri");
const cargoTomlPath = path.join(tauriDir, "Cargo.toml");
const tauriConfigPath = path.join(tauriDir, "tauri.conf.json");

if (process.platform === "win32" && !process.env.CMAKE_GENERATOR) {
    process.env.CMAKE_GENERATOR = "Visual Studio 17 2022";
}

function run(command, args, options = {}) {
    const result = spawnSync(command, args, {
        cwd: options.cwd || rootDir,
        env: process.env,
        shell: false,
        stdio: "inherit",
    });

    if (result.error) {
        throw result.error;
    }

    if (result.status !== 0) {
        throw new Error(`${command} ${args.join(" ")} failed with exit code ${result.status}`);
    }
}

function readCargoPackageName() {
    const cargoToml = fs.readFileSync(cargoTomlPath, "utf8");
    const packageIndex = cargoToml.indexOf("[package]");

    if (packageIndex === -1) {
        throw new Error(`Could not find [package] in ${cargoTomlPath}`);
    }

    const nextSectionIndex = cargoToml.indexOf("\n[", packageIndex + "[package]".length);
    const packageSection =
        nextSectionIndex === -1
            ? cargoToml.slice(packageIndex)
            : cargoToml.slice(packageIndex, nextSectionIndex);
    const name = packageSection.match(/^\s*name\s*=\s*"([^"]+)"/m);

    if (!name) {
        throw new Error(`Could not read package.name from ${cargoTomlPath}`);
    }

    return name[1];
}

function toMsixVersion(version) {
    const parts = version.split(".").map((part) => Number.parseInt(part, 10));

    if (parts.some((part) => !Number.isInteger(part) || part < 0)) {
        throw new Error(`Version "${version}" must use numeric dot-separated parts for MSIX packaging`);
    }

    while (parts.length < 4) {
        parts.push(0);
    }

    return parts.slice(0, 4).join(".");
}

function shouldCopyReleaseFile(entry) {
    const ext = path.extname(entry.name).toLowerCase();
    return ext === ".exe" || ext === ".dll";
}

const tauriConfig = JSON.parse(fs.readFileSync(tauriConfigPath, "utf8"));
const executableName = readCargoPackageName();
const packageIdentityName =
    process.env.WINDOWS_MSIX_PACKAGE_NAME || "9154AndyLang.MatrixPrismPro";
const productName = process.env.WINDOWS_MSIX_DISPLAY_NAME || "Matrix Prism Pro";
const description =
    process.env.WINDOWS_MSIX_DESCRIPTION || "Monitor and manage your workstation.";
const publisherName =
    process.env.WINDOWS_MSIX_PUBLISHER_NAME || "CN=6FB2A914-EE54-42DB-81F4-B1939C1041FF";
const publisherDisplayName = process.env.WINDOWS_MSIX_PUBLISHER_DISPLAY_NAME || "Andy Lang";
const version = toMsixVersion(
    process.env.WINDOWS_MSIX_VERSION || tauriConfig.version || "1.0.0",
);
const exeName = `${executableName}.exe`;
const releaseDir = path.join(tauriDir, "target", "release");
const releaseExe = path.join(releaseDir, exeName);
const msixRoot = path.join(tauriDir, "target", "msix");
const stageDir = path.join(msixRoot, "package");
const manifestPath = path.join(stageDir, "Package.appxmanifest");
const outputFileName = `${productName.replace(/[^\w.-]+/g, "-")}_${version}.msix`;
const outputPath = path.join(msixRoot, outputFileName);
const releaseTag = process.env.WINDOWS_MSIX_RELEASE_TAG || `v${tauriConfig.version}`;
const packageUrl =
    process.env.WINDOWS_MSIX_PACKAGE_URL ||
    `https://github.com/AndrewLang/matrix-process-manager/releases/download/${releaseTag}/${outputFileName}`;
const logoPath = path.join(tauriDir, "icons", "icon.png");
const winappCliPath = path.join(
    rootDir,
    "node_modules",
    "@microsoft",
    "winappcli",
    "dist",
    "cli.js",
);

run("cargo", ["tauri", "build", "--no-bundle"], { cwd: tauriDir });

if (!fs.existsSync(releaseExe)) {
    throw new Error(`Expected release executable was not found: ${releaseExe}`);
}

fs.rmSync(stageDir, { recursive: true, force: true });
fs.mkdirSync(stageDir, { recursive: true });
fs.mkdirSync(msixRoot, { recursive: true });

for (const entry of fs.readdirSync(releaseDir, { withFileTypes: true })) {
    if (entry.isFile() && shouldCopyReleaseFile(entry)) {
        fs.copyFileSync(path.join(releaseDir, entry.name), path.join(stageDir, entry.name));
    }
}

run(
    process.execPath,
    [
        winappCliPath,
        "manifest",
        "generate",
        stageDir,
        "--package-name",
        packageIdentityName,
        "--publisher-name",
        publisherName,
        "--version",
        version,
        "--description",
        description,
        "--entrypoint",
        path.join(stageDir, exeName),
        "--logo-path",
        logoPath,
        "--if-exists",
        "Overwrite",
    ],
    { cwd: rootDir },
);

let manifest = fs.readFileSync(manifestPath, "utf8");
manifest = manifest
    .replace(/<DisplayName>[^<]*<\/DisplayName>/, `<DisplayName>${productName}</DisplayName>`)
    .replace(
        /<PublisherDisplayName>[^<]*<\/PublisherDisplayName>/,
        `<PublisherDisplayName>${publisherDisplayName}</PublisherDisplayName>`,
    )
    .replace(/(<uap:VisualElements[\s\S]*?\sDisplayName=")[^"]*(")/, `$1${productName}$2`);
fs.writeFileSync(manifestPath, manifest);

const packArgs = [
    winappCliPath,
    "package",
    stageDir,
    "--output",
    outputPath,
    "--executable",
    exeName,
];

if (process.env.WINDOWS_MSIX_CERT_PATH) {
    packArgs.push("--cert", process.env.WINDOWS_MSIX_CERT_PATH);

    if (process.env.WINDOWS_MSIX_CERT_PASSWORD) {
        packArgs.push("--cert-password", process.env.WINDOWS_MSIX_CERT_PASSWORD);
    }
}

if (process.env.WINDOWS_MSIX_GENERATE_CERT === "1") {
    packArgs.push("--generate-cert", "--publisher", publisherName);
}

if (process.env.WINDOWS_MSIX_INSTALL_CERT === "1") {
    packArgs.push("--install-cert");
}

if (process.env.WINDOWS_MSIX_SELF_CONTAINED === "1") {
    packArgs.push("--self-contained");
}

run(process.execPath, packArgs, { cwd: rootDir });

console.log(`MSIX created at ${outputPath}`);
console.log(`Package URL: ${packageUrl}`);
