import { helpRequested, parseArgs, requireNoPositionals } from "@/lib/cli.ts";
import { cmd } from "@/lib/std/cmd.ts";
import { fs } from "@/lib/std/fs.ts";
import { io } from "@/lib/std/io.ts";
import { path } from "@/lib/std/path.ts";

const HOOKS_PATH = ".runseal/hooks";

function usage(): void {
  io.print("Usage: runseal :init");
  io.print("");
  io.print("Validate the repository and install versioned git hooks.");
}

async function requireTool(name: string): Promise<void> {
  if (!(await cmd.exists(name))) {
    io.fail(`init: missing required tool: ${name}`);
  }
}

async function requirePath(root: string, relPath: string): Promise<void> {
  if (!(await fs.file.exists(path.join(root, relPath)))) {
    io.fail(`init: missing required path: ${relPath}`);
  }
}

const args = parseArgs(Deno.args, { boolean: ["help", "h"] });
requireNoPositionals(args, "init", { allowHelp: true });
if (helpRequested(args)) {
  usage();
  Deno.exit(0);
}

io.print("==> resolving repository");
const root = await cmd.text("git", ["rev-parse", "--show-toplevel"]);
io.print(`repository: ${root}`);

io.print("==> checking required tools");
for (
  const tool of [
    "git",
    "deno",
    "python3",
    "cargo",
    "runseal",
    "flavor",
    "sh",
    "bash",
    "sed",
    "grep",
  ]
) {
  await requireTool(tool);
}
io.print("ok: git, deno, python3, cargo, runseal, flavor, sh, bash, sed, grep");

io.print("==> checking repository entrypoints");
for (
  const path of [
    "Cargo.toml",
    "Cargo.lock",
    "flavor.toml",
    "manage.sh",
    "manage.ps1",
    "runseal.toml",
    ".runseal/deno.json",
    ".runseal/deno.lock",
    ".runseal/hooks/pre-commit",
    ".runseal/hooks/commit-msg",
    ".runseal/lib/cli.ts",
    ".runseal/lib/hash.ts",
    ".runseal/lib/std/cmd.ts",
    ".runseal/lib/std/env.ts",
    ".runseal/lib/std/fs.ts",
    ".runseal/lib/std/io.ts",
    ".runseal/lib/std/json.ts",
    ".runseal/lib/std/path.ts",
    ".runseal/lib/std/runseal.ts",
    ".runseal/lib/version.ts",
    ".runseal/templates/cloudflare.env",
    ".runseal/wrappers/cloudflare.ts",
    ".runseal/wrappers/guard.ts",
    ".runseal/wrappers/init.ts",
    ".runseal/wrappers/land.ts",
    ".runseal/wrappers/release.ts",
    ".github/workflows/guard.yml",
    ".github/workflows/release-beta.yml",
    ".github/workflows/release-stable.yml",
    ".github/scripts/release/assets/checksums.sh",
    ".github/scripts/release/assets/package.sh",
    ".github/scripts/release/assets/package.ps1",
    ".github/scripts/release/assets/verify.sh",
    ".github/scripts/release/github/cleanup-artifacts.sh",
    ".github/scripts/release/metadata/beta.py",
    ".github/scripts/release/metadata/stable.py",
    ".github/scripts/release/r2/check.sh",
    ".github/scripts/release/r2/publish.sh",
    ".github/scripts/release/r2/summary.sh",
    ".github/scripts/release/r2/verify.sh",
    ".github/scripts/release/smoke/smoke.sh",
    ".github/scripts/release/smoke/smoke.ps1",
  ]
) {
  await requirePath(root, path);
}
io.print("ok: repository entrypoints");

io.print("==> installing git hooks");
await cmd.run("git", ["config", "core.hooksPath", HOOKS_PATH], { cwd: root });
const current = await cmd.text("git", ["config", "--get", "core.hooksPath"], { cwd: root });
io.print(`core.hooksPath = ${current}`);

await cmd.run("deno", ["--version"], { stdout: "null" });
io.print("development environment ready");
