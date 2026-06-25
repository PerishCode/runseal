import { fail, jsonEmpty, jsonGet, print, run, runsealText, runText } from "../lib/runseal.ts";

type Options = {
  channel: string;
  ref: string;
  version: string;
  watch: boolean;
  dryRun: boolean;
};

function usage(): void {
  print("Usage: runseal :release --channel=stable|beta [options]");
  print("");
  print("Trigger a release workflow.");
  print("");
  print("Options:");
  print("  --channel <name>      release channel: stable or beta");
  print("  --ref <ref>           git ref passed to the workflow (default: main)");
  print("  --version <version>   optional workflow version_override");
  print("  --watch              watch the triggered workflow run");
  print("  --dry-run            print planned action without triggering a workflow");
}

function parseArgs(args: string[]): Options & { help: boolean; argc: number } {
  const options = {
    channel: "",
    ref: "main",
    version: "",
    watch: false,
    dryRun: false,
    help: false,
    argc: args.length,
  };
  while (args.length > 0) {
    const arg = args.shift()!;
    if (arg === "--") {
      break;
    }
    if (arg === "-h" || arg === "--help" || arg === "help") {
      options.help = true;
      continue;
    }
    if (arg === "--watch") {
      options.watch = true;
      continue;
    }
    if (arg === "--dry-run") {
      options.dryRun = true;
      continue;
    }
    for (
      const [name, key] of [
        ["--channel", "channel"],
        ["--ref", "ref"],
        ["--version", "version"],
      ] as const
    ) {
      if (arg === name) {
        const value = args.shift();
        if (value === undefined) {
          fail(`missing value for ${name}`);
        }
        options[key] = value;
        continue;
      }
      const prefix = `${name}=`;
      if (arg.startsWith(prefix)) {
        options[key] = arg.slice(prefix.length);
        continue;
      }
    }
    if (
      arg === "--channel" || arg.startsWith("--channel=") ||
      arg === "--ref" || arg.startsWith("--ref=") ||
      arg === "--version" || arg.startsWith("--version=")
    ) {
      continue;
    }
    fail(`unknown option: ${arg}`);
  }
  return options;
}

const options = parseArgs([...Deno.args]);
if (options.argc === 0 || options.help) {
  usage();
  Deno.exit(0);
}
if (options.channel === "") {
  fail("release: --channel is required");
}

let workflow: string;
switch (options.channel) {
  case "stable":
    workflow = "release-stable.yml";
    break;
  case "beta":
    workflow = "release-beta.yml";
    break;
  default:
    fail(`invalid choice: ${options.channel}`, 2);
}

const dryRunCommand =
  `gh workflow run ${workflow} --ref ${options.ref} -f ref=${options.ref} -f version_override=${options.version}`;
if (options.dryRun) {
  print(dryRunCommand);
  Deno.exit(0);
}

await run("gh", ["--version"]);
await run("gh", ["auth", "status"]);
const refSha = await runText("git", ["rev-parse", options.ref]);
const triggerOutput = await runText("gh", [
  "workflow",
  "run",
  workflow,
  "--ref",
  options.ref,
  "-f",
  `ref=${options.ref}`,
  "-f",
  `version_override=${options.version}`,
]);
if (triggerOutput !== "") {
  print(triggerOutput);
}
print(`triggered ${workflow} for ref ${options.ref}`);

if (options.watch) {
  let runId = await runsealText([
    "@tool",
    "regex",
    "capture",
    triggerOutput,
    "/actions/runs/([0-9]+)",
    "1",
  ]);
  if (runId === "") {
    let raw = "[]";
    for (let attempt = 0; attempt < 6; attempt += 1) {
      raw = await runText("gh", [
        "run",
        "list",
        "--workflow",
        workflow,
        "--branch",
        options.ref,
        "--commit",
        refSha,
        "--event",
        "workflow_dispatch",
        "--limit",
        "1",
        "--json",
        "databaseId",
      ]);
      if (!(await jsonEmpty(raw))) {
        runId = await jsonGet(raw, ".[0].databaseId");
        break;
      }
      await new Promise((resolve) => setTimeout(resolve, 2000));
    }
  }
  if (runId === "") {
    fail(`release: could not find a recent run for ${workflow} on ${options.ref}`);
  }
  await run("gh", ["run", "watch", runId, "--interval", "10"]);
}
