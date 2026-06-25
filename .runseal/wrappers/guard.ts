import { env, fail, jsonGet, print, run, runsealText, runText } from "../lib/runseal.ts";

function usage(): void {
  print("Usage: runseal :guard [version-check|version-hash]");
  print("");
  print("Run repository guard checks or one explicit version-policy helper.");
  print("");
  print("Commands:");
  print("  version-check    validate version policy against stable metadata");
  print("  version-hash     print the current guard.version.hash value");
}

let mode = "full";
const args = [...Deno.args];
if (args.length > 0) {
  const arg = args.shift()!;
  switch (arg) {
    case "version-check":
      mode = "version-check";
      break;
    case "version-hash":
      mode = "version-hash";
      break;
    case "-h":
    case "--help":
    case "help":
      usage();
      Deno.exit(0);
    default:
      fail(`guard: unknown command: ${arg}`);
  }
}
if (args.length > 0) {
  fail("guard: unexpected arguments");
}

async function currentHash(): Promise<string> {
  return await runsealText(["@tool", "hash", "tree", "app/tests"]);
}

async function versionPolicy(): Promise<void> {
  const publicUrl = env("RUNSEAL_RELEASES_PUBLIC_URL", "https://releases.runseal.perish.uk");
  const metadataUrl = env(
    "RUNSEAL_STABLE_METADATA_URL",
    `${publicUrl}/stable/latest/metadata.json`,
  );

  const cargoMetadata = await runText("cargo", ["metadata", "--no-deps", "--format-version", "1"]);
  const currentVersion = await jsonGet(cargoMetadata, ".packages[0].version");
  const hash = await currentHash();
  const response = await fetch(`${metadataUrl}?version=${encodeURIComponent(currentVersion)}`);
  if (response.status === 404) {
    print("guard version policy: no stable metadata; skipping");
    return;
  }
  if (response.status !== 200) {
    fail(`guard version policy: failed to fetch stable metadata: HTTP ${response.status}`);
  }

  const metadata = await response.text();
  const hasPriorHash = await runsealText(["@tool", "json", "has", metadata, ".guard.version.hash"]);
  const priorHash = hasPriorHash === "true" ? await jsonGet(metadata, ".guard.version.hash") : "";
  if (priorHash === "") {
    print("guard version policy: stable metadata has no guard.version.hash; skipping");
    return;
  }

  const hasStableVersion = await runsealText(["@tool", "json", "has", metadata, ".stableVersion"]);
  let priorVersion = hasStableVersion === "true" ? await jsonGet(metadata, ".stableVersion") : "";
  if (priorVersion === "") {
    const hasReleaseVersion = await runsealText([
      "@tool",
      "json",
      "has",
      metadata,
      ".releaseVersion",
    ]);
    if (hasReleaseVersion === "true") {
      priorVersion = await jsonGet(metadata, ".releaseVersion");
    }
  }
  if (priorVersion === "") {
    fail("guard version policy: stable metadata is missing stableVersion/releaseVersion");
  }

  const currentOrder = await runsealText([
    "@tool",
    "version",
    "compare",
    currentVersion,
    priorVersion,
  ]);
  const priorMajor = await runsealText(["@tool", "version", "part", priorVersion, "major"]);
  const priorMinor = await runsealText(["@tool", "version", "part", priorVersion, "minor"]);
  const currentMajor = await runsealText(["@tool", "version", "part", currentVersion, "major"]);
  const currentMinor = await runsealText(["@tool", "version", "part", currentVersion, "minor"]);
  const sameMinorLineage = currentMajor === priorMajor && currentMinor === priorMinor;

  if (currentOrder === "lt") {
    fail(`guard version policy: version regressed below prior stable ${priorVersion}`);
  }
  if (currentOrder === "eq") {
    fail(`guard version policy: version matches prior stable ${priorVersion}`);
  }

  if (hash === priorHash) {
    if (!sameMinorLineage) {
      fail(
        `guard version policy: unchanged guard.version.hash requires a patch-only bump above ${priorVersion}`,
      );
    }
    print(
      `guard version policy: hash unchanged -> patch bump ok (${priorVersion} -> ${currentVersion})`,
    );
  } else {
    if (sameMinorLineage) {
      fail(
        `guard version policy: changed guard.version.hash requires a minor-or-higher bump above ${priorVersion}`,
      );
    }
    print(
      `guard version policy: hash changed -> minor-or-higher bump ok (${priorVersion} -> ${currentVersion})`,
    );
  }
}

if (mode === "version-hash") {
  print(await currentHash());
  Deno.exit(0);
}

await versionPolicy();
if (mode === "version-check") {
  Deno.exit(0);
}

print("==> cargo fmt");
await run("cargo", ["fmt", "--all", "--check"]);

print("==> cargo clippy");
await run("cargo", ["clippy", "--locked", "--workspace", "--all-targets", "--", "-D", "warnings"]);

print("==> cargo test");
await run("cargo", ["test", "--locked", "--workspace"]);

print("==> deno fmt");
await run("deno", ["fmt", "--check", ".runseal"]);

print("==> deno check");
await run("deno", [
  "check",
  "--config",
  ".runseal/deno.json",
  "--lock",
  "deno.lock",
  "--frozen=true",
  ".runseal/wrappers/cloudflare.ts",
  ".runseal/wrappers/guard.ts",
  ".runseal/wrappers/init.ts",
  ".runseal/wrappers/pr.ts",
  ".runseal/wrappers/release.ts",
]);

print("==> flavor self-check");
await run("flavor", ["check", "--root", ".", "--config", "flavor.toml"]);

print("==> shell syntax");
for (
  const [command, script] of [
    ["sh", "manage.sh"],
    ["sh", ".github/scripts/release/assets/checksums.sh"],
    ["sh", ".github/scripts/release/assets/package.sh"],
    ["sh", ".github/scripts/release/assets/verify.sh"],
    ["sh", ".github/scripts/release/github/cleanup-artifacts.sh"],
    ["bash", ".github/scripts/release/r2/check.sh"],
    ["bash", ".github/scripts/release/r2/publish.sh"],
    ["bash", ".github/scripts/release/r2/summary.sh"],
    ["bash", ".github/scripts/release/r2/verify.sh"],
    ["sh", ".github/scripts/release/smoke/smoke.sh"],
  ]
) {
  await run(command, ["-n", script]);
}

print("==> python syntax");
await run("python3", ["-m", "py_compile", ".github/scripts/release/metadata/beta.py"]);
await run("python3", ["-m", "py_compile", ".github/scripts/release/metadata/stable.py"]);

const hasPwsh = await runsealText(["@tool", "process", "exists", "pwsh"]);
print("==> PowerShell syntax");
if (hasPwsh === "true") {
  await run("pwsh", [
    "-NoProfile",
    "-NonInteractive",
    "-Command",
    "[scriptblock]::Create((Get-Content -Raw 'manage.ps1')) | Out-Null",
  ]);
  await run("pwsh", [
    "-NoProfile",
    "-NonInteractive",
    "-Command",
    "[scriptblock]::Create((Get-Content -Raw '.github/scripts/release/assets/package.ps1')) | Out-Null",
  ]);
  await run("pwsh", [
    "-NoProfile",
    "-NonInteractive",
    "-Command",
    "[scriptblock]::Create((Get-Content -Raw '.github/scripts/release/smoke/smoke.ps1')) | Out-Null",
  ]);
} else {
  print("skip: pwsh not found");
}
