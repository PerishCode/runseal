import { fail, jsonEmpty, jsonGet, print, run, runsealText, runText } from "../lib/runseal.ts";

type Options = {
  base: string;
  title: string;
  bodyFile: string;
  draft: boolean;
  noWatch: boolean;
  noMerge: boolean;
  noPush: boolean;
  dryRun: boolean;
};

function usage(): void {
  print("Usage: runseal :pr [options]");
  print("");
  print("Create or update, watch, and squash-merge the GitHub PR for the current branch.");
  print("");
  print("Options:");
  print("  --base <branch>       PR base branch (default: main)");
  print("  --title <title>       title when creating a new PR");
  print("  --body-file <path>    body file when creating a new PR");
  print("  --draft              create the PR as draft and require --no-merge");
  print("  --no-watch           do not watch PR checks");
  print("  --no-merge           do not squash-merge after checks");
  print("  --no-push            do not push the current branch first");
  print("  --dry-run            print planned actions without changing remote state");
}

function parseArgs(args: string[]): Options & { help: boolean } {
  const options = {
    base: "main",
    title: "",
    bodyFile: "",
    draft: false,
    noWatch: false,
    noMerge: false,
    noPush: false,
    dryRun: false,
    help: false,
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
    if (arg === "--draft") {
      options.draft = true;
      continue;
    }
    if (arg === "--no-watch") {
      options.noWatch = true;
      continue;
    }
    if (arg === "--no-merge") {
      options.noMerge = true;
      continue;
    }
    if (arg === "--no-push") {
      options.noPush = true;
      continue;
    }
    if (arg === "--dry-run") {
      options.dryRun = true;
      continue;
    }
    for (
      const [name, key] of [
        ["--base", "base"],
        ["--title", "title"],
        ["--body-file", "bodyFile"],
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
      arg === "--base" || arg.startsWith("--base=") ||
      arg === "--title" || arg.startsWith("--title=") ||
      arg === "--body-file" || arg.startsWith("--body-file=")
    ) {
      continue;
    }
    fail(`unknown option: ${arg}`);
  }
  return options;
}

function printDryRun(options: Options, branch: string): void {
  print(`branch: ${branch}`);
  print(`base: ${options.base}`);
  print(`push: ${options.noPush ? "False" : "True"}`);
  print("pr: create if missing, otherwise reuse existing");
  if (options.draft) {
    print("draft: True");
    print("ready: False");
  } else {
    print("draft: False");
    print("ready: True");
  }
  print(`watch: ${options.noWatch ? "False" : "True"}`);
  print(`squash_merge: ${options.noMerge ? "False" : "True"}`);
}

async function createPr(options: Options, branch: string): Promise<void> {
  const args = ["pr", "create"];
  if (options.draft) {
    args.push("--draft");
  }
  args.push("--base", options.base, "--head", branch);
  if (options.title !== "") {
    args.push("--title", options.title);
  }
  if (options.bodyFile !== "") {
    args.push("--body-file", options.bodyFile);
  } else {
    args.push("--fill");
  }
  await run("gh", args);
}

async function watchChecks(number: string): Promise<void> {
  let checksSeen = false;
  for (let attempt = 0; attempt < 12; attempt += 1) {
    checksSeen = (await runsealText(["@tool", "github", "pr", "checks", "probe", number])) ===
      "true";
    if (checksSeen) {
      break;
    }
    await new Promise((resolve) => setTimeout(resolve, 5000));
  }
  if (!checksSeen) {
    print(`no checks reported on PR #${number}; skipping watch`);
  } else {
    await run("gh", ["pr", "checks", number, "--watch", "--interval", "10"]);
  }
}

const options = parseArgs([...Deno.args]);
if (options.help) {
  usage();
  Deno.exit(0);
}

await run("git", ["--version"]);
await run("gh", ["--version"]);
await run("gh", ["auth", "status"]);

const branch = await runText("git", ["branch", "--show-current"]);
if (branch === "") {
  fail("pr: not on a branch");
}
if (branch === options.base || branch === "main" || branch === "master") {
  fail(`pr: refusing to open a PR from base branch: ${branch}`);
}
if (options.draft && !options.noMerge) {
  fail("pr: --draft requires --no-merge");
}

if (options.dryRun) {
  printDryRun(options, branch);
  Deno.exit(0);
}

if (!options.noPush) {
  await run("git", ["push", "-u", "origin", branch]);
}

let created = false;
let prRaw = await runText("gh", [
  "pr",
  "list",
  "--head",
  branch,
  "--json",
  "number,title,state,url,isDraft",
]);
if (await jsonEmpty(prRaw)) {
  await createPr(options, branch);
  created = true;
  prRaw = await runText("gh", [
    "pr",
    "list",
    "--head",
    branch,
    "--json",
    "number,title,state,url,isDraft",
  ]);
  if (await jsonEmpty(prRaw)) {
    fail(`pr: created PR for ${branch}, but could not find it afterward`);
  }
}

const number = await jsonGet(prRaw, ".[0].number");
const url = await jsonGet(prRaw, ".[0].url");
const isDraft = await jsonGet(prRaw, ".[0].isDraft");

if (created) {
  print(`created PR #${number}: ${url}`);
} else {
  print(`found PR #${number}: ${url}`);
}

if (isDraft === "true" && !options.draft) {
  await run("gh", ["pr", "ready", number]);
  print(`marked PR #${number} ready`);
}

if (!options.noWatch) {
  await watchChecks(number);
}

if (!options.noMerge) {
  await run("gh", ["pr", "merge", number, "--squash", "--delete-branch"]);
  print(`squash-merged PR #${number}`);
}
