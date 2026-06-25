import {
  booleanOption,
  helpRequested,
  parseArgs as parseCliArgs,
  requireNoPositionals,
  stringOption,
} from "@/lib/cli.ts";
import { cmd } from "@/lib/std/cmd.ts";
import { io } from "@/lib/std/io.ts";
import { json } from "@/lib/std/json.ts";
import { runseal } from "@/lib/std/runseal.ts";

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
  io.print("Usage: runseal :pr [options]");
  io.print("");
  io.print("Create or update, watch, and squash-merge the GitHub PR for the current branch.");
  io.print("");
  io.print("Options:");
  io.print("  --base <branch>       PR base branch (default: main)");
  io.print("  --title <title>       title when creating a new PR");
  io.print("  --body-file <path>    body file when creating a new PR");
  io.print("  --draft              create the PR as draft and require --no-merge");
  io.print("  --no-watch           do not watch PR checks");
  io.print("  --no-merge           do not squash-merge after checks");
  io.print("  --no-push            do not push the current branch first");
  io.print("  --dry-run            print planned actions without changing remote state");
}

function parseArgs(args: string[]): Options & { help: boolean } {
  const parsed = parseCliArgs(args, {
    string: ["base", "title", "body-file"],
    boolean: ["draft", "no-watch", "no-merge", "no-push", "dry-run", "help", "h"],
  });
  requireNoPositionals(parsed, "pr", { allowHelp: true });
  return {
    base: stringOption(parsed, "base", "main"),
    title: stringOption(parsed, "title"),
    bodyFile: stringOption(parsed, "body-file"),
    draft: booleanOption(parsed, "draft"),
    noWatch: booleanOption(parsed, "no-watch"),
    noMerge: booleanOption(parsed, "no-merge"),
    noPush: booleanOption(parsed, "no-push"),
    dryRun: booleanOption(parsed, "dry-run"),
    help: helpRequested(parsed),
  };
}

function printDryRun(options: Options, branch: string): void {
  io.print(`branch: ${branch}`);
  io.print(`base: ${options.base}`);
  io.print(`push: ${options.noPush ? "False" : "True"}`);
  io.print("pr: create if missing, otherwise reuse existing");
  if (options.draft) {
    io.print("draft: True");
    io.print("ready: False");
  } else {
    io.print("draft: False");
    io.print("ready: True");
  }
  io.print(`watch: ${options.noWatch ? "False" : "True"}`);
  io.print(`squash_merge: ${options.noMerge ? "False" : "True"}`);
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
  await cmd.run("gh", args);
}

async function watchChecks(number: string): Promise<void> {
  let checksSeen = false;
  for (let attempt = 0; attempt < 12; attempt += 1) {
    checksSeen = (await runseal.text(["@tool", "github", "pr", "checks", "probe", number])) ===
      "true";
    if (checksSeen) {
      break;
    }
    await new Promise((resolve) => setTimeout(resolve, 5000));
  }
  if (!checksSeen) {
    io.print(`no checks reported on PR #${number}; skipping watch`);
  } else {
    await cmd.run("gh", ["pr", "checks", number, "--watch", "--interval", "10"]);
  }
}

const options = parseArgs([...Deno.args]);
if (options.help) {
  usage();
  Deno.exit(0);
}

await cmd.run("git", ["--version"]);
await cmd.run("gh", ["--version"]);
await cmd.run("gh", ["auth", "status"]);

const branch = await cmd.text("git", ["branch", "--show-current"]);
if (branch === "") {
  io.fail("pr: not on a branch");
}
if (branch === options.base || branch === "main" || branch === "master") {
  io.fail(`pr: refusing to open a PR from base branch: ${branch}`);
}
if (options.draft && !options.noMerge) {
  io.fail("pr: --draft requires --no-merge");
}

if (options.dryRun) {
  printDryRun(options, branch);
  Deno.exit(0);
}

if (!options.noPush) {
  await cmd.run("git", ["push", "-u", "origin", branch]);
}

let created = false;
let prRaw = await cmd.text("gh", [
  "pr",
  "list",
  "--head",
  branch,
  "--json",
  "number,title,state,url,isDraft",
]);
if (json.empty(prRaw)) {
  await createPr(options, branch);
  created = true;
  prRaw = await cmd.text("gh", [
    "pr",
    "list",
    "--head",
    branch,
    "--json",
    "number,title,state,url,isDraft",
  ]);
  if (json.empty(prRaw)) {
    io.fail(`pr: created PR for ${branch}, but could not find it afterward`);
  }
}

const number = json.get(prRaw, ".[0].number");
const url = json.get(prRaw, ".[0].url");
const isDraft = json.get(prRaw, ".[0].isDraft");

if (created) {
  io.print(`created PR #${number}: ${url}`);
} else {
  io.print(`found PR #${number}: ${url}`);
}

if (isDraft === "true" && !options.draft) {
  await cmd.run("gh", ["pr", "ready", number]);
  io.print(`marked PR #${number} ready`);
}

if (!options.noWatch) {
  await watchChecks(number);
}

if (!options.noMerge) {
  await cmd.run("gh", ["pr", "merge", number, "--squash", "--delete-branch"]);
  io.print(`squash-merged PR #${number}`);
}
