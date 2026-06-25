import { env, fail, fileExists, jsonGet, print, runseal, runsealText } from "../lib/runseal.ts";

function usage(): void {
  print("Usage: runseal :cloudflare <command> [args]");
  print("");
  print("Commands:");
  print("  init                      create repo-local .local/secrets/cloudflare.env template");
  print("  check                     validate repo-local credentials and probe core account APIs");
  print("  manage-plan               print the desired manage redirect rule shape");
  print("  manage-inspect            inspect current dynamic redirect ruleset for manage rules");
  print(
    "  manage-ensure-redirect    create/update exact-path manage redirects (use --dry-run first)",
  );
  print(
    "  api                       use: runseal @tool cloudflare api request <method> <path> ...",
  );
  print("");
  print("Credentials:");
  print("  .local/secrets/cloudflare.env");
}

function rejectExtraArg(value: string | undefined, message: string): void {
  if (value !== undefined && value !== "") {
    fail(message);
  }
}

type ManageRules = {
  zoneName: string;
  requestHost: string;
  redirectHost: string;
  ruleSh: string;
  rulePs1: string;
};

async function loadManageRedirectRules(): Promise<ManageRules> {
  const zoneName = await runsealText(["@tool", "cloudflare", "config", "get", "zone_name"]);
  const requestHost = await runsealText(["@tool", "cloudflare", "config", "get", "manage_host"]);
  const redirectHost = await runsealText([
    "@tool",
    "cloudflare",
    "config",
    "get",
    "manage_origin_host",
  ]);
  const prefix = await runsealText([
    "@tool",
    "cloudflare",
    "config",
    "get",
    "manage_redirect_prefix",
  ]);
  const targetSh = prefix === ""
    ? `https://${redirectHost}/manage.sh`
    : `https://${redirectHost}/${prefix}/manage.sh`;
  const targetPs1 = prefix === ""
    ? `https://${redirectHost}/manage.ps1`
    : `https://${redirectHost}/${prefix}/manage.ps1`;
  const ruleSh = await runsealText([
    "@tool",
    "cloudflare",
    "redirect-rule",
    "exact",
    "--ref",
    "runseal_manage_sh_redirect",
    "--description",
    "Redirect runseal manage.sh to releases bucket asset",
    "--host",
    requestHost,
    "--path",
    "/manage.sh",
    "--target-url",
    targetSh,
  ]);
  const rulePs1 = await runsealText([
    "@tool",
    "cloudflare",
    "redirect-rule",
    "exact",
    "--ref",
    "runseal_manage_ps1_redirect",
    "--description",
    "Redirect runseal manage.ps1 to releases bucket asset",
    "--host",
    requestHost,
    "--path",
    "/manage.ps1",
    "--target-url",
    targetPs1,
  ]);
  return { zoneName, requestHost, redirectHost, ruleSh, rulePs1 };
}

async function printManageRedirectPlan(rules: ManageRules, zoneId?: string): Promise<void> {
  const prettySh = await runsealText(["@tool", "json", "pretty", "value", rules.ruleSh]);
  const prettyPs1 = await runsealText(["@tool", "json", "pretty", "value", rules.rulePs1]);
  print("manage redirect plan");
  print(`zone: ${rules.zoneName}`);
  if (zoneId !== undefined) {
    print(`zone id: ${zoneId}`);
  }
  print(`request host: ${rules.requestHost}`);
  print(`redirect host: ${rules.redirectHost}`);
  print("phase: http_request_dynamic_redirect");
  print("rules:");
  print(prettySh);
  print(prettyPs1);
}

const [command, ...rest] = Deno.args;
if (command === undefined || command === "help" || command === "--help") {
  usage();
  Deno.exit(0);
}

switch (command) {
  case "init": {
    rejectExtraArg(rest[0], "cloudflare: init does not accept arguments");
    const localDir = env("RUNSEAL_REPO_LOCAL_DIR", ".local");
    const secretsDir = env("RUNSEAL_REPO_SECRETS_DIR", ".local/secrets");
    const tmpDir = env("RUNSEAL_REPO_TMP_DIR", ".local/tmp");
    const tokenFile = `${secretsDir}/cloudflare.env`;
    await runseal(["@tool", "fs", "mkdir", localDir, "700"]);
    await runseal(["@tool", "fs", "mkdir", secretsDir, "700"]);
    await runseal(["@tool", "fs", "mkdir", tmpDir, "700"]);
    if (await fileExists(tokenFile)) {
      print(`exists ${tokenFile}`);
    } else {
      await runseal([
        "@tool",
        "fs",
        "write-base64",
        tokenFile,
        "IyBSZXBvLWxvY2FsIENsb3VkZmxhcmUgY3JlZGVudGlhbHMgZm9yIHJ1bnNlYWwgc3VwcG9ydCBjb21tYW5kcy4KIyBGaWxsIHRoZXNlIHZhbHVlcyBtYW51YWxseS4gVGhpcyBmaWxlIHN0YXlzIGxvY2FsIGFuZCBnaXRpZ25vcmVkLgpDTE9VREZMQVJFX0FDQ09VTlRfSUQ9CkNMT1VERkxBUkVfQVBJX1RPS0VOPQpDTE9VREZMQVJFX1pPTkVfTkFNRT1wZXJpc2gudWsKQ0xPVURGTEFSRV9NQU5BR0VfSE9TVD1ydW5zZWFsLnBlcmlzaC51awpDTE9VREZMQVJFX01BTkFHRV9PUklHSU5fSE9TVD1yZWxlYXNlcy5ydW5zZWFsLnBlcmlzaC51awpDTE9VREZMQVJFX01BTkFHRV9SRURJUkVDVF9QUkVGSVg9Cg==",
      ]);
      await runseal(["@tool", "fs", "chmod", tokenFile, "600"]);
      print(`created ${tokenFile}`);
    }
    break;
  }
  case "check": {
    rejectExtraArg(rest[0], "cloudflare: check does not accept arguments");
    const accountId = await runsealText(["@tool", "cloudflare", "config", "get", "account_id"]);
    const zoneName = await runsealText(["@tool", "cloudflare", "config", "get", "zone_name"]);
    const zone = await runsealText(["@tool", "cloudflare", "zone", "get", "--name", zoneName]);
    const zoneId = await jsonGet(zone, ".id");
    const rulesets = await runsealText([
      "@tool",
      "cloudflare",
      "zone",
      "ruleset",
      "list",
      "--zone-id",
      zoneId,
    ]);
    const rulesetCount = await runsealText(["@tool", "json", "len", rulesets]);
    const zonesPayload = await runsealText([
      "@tool",
      "cloudflare",
      "api",
      "request",
      "GET",
      "/zones",
      "--query",
      `account.id=${accountId}`,
      "--query",
      "per_page=50",
    ]);
    const zones = await jsonGet(zonesPayload, ".result");
    const zonesPretty = await runsealText(["@tool", "json", "pretty", "value", zones]);
    const account = await runsealText([
      "@tool",
      "cloudflare",
      "account",
      "get",
      "--account-id",
      accountId,
    ]);
    const accountName = await jsonGet(account, ".name");
    const buckets = await runsealText([
      "@tool",
      "cloudflare",
      "account",
      "r2",
      "bucket",
      "list",
      "--account-id",
      accountId,
    ]);
    const bucketsPretty = await runsealText(["@tool", "json", "pretty", "value", buckets]);
    print("cloudflare check: ok");
    print(`account id: ${accountId}`);
    print(`account name: ${accountName}`);
    print(`manage zone: ${zoneName} (${zoneId})`);
    print(`zone rulesets: ${rulesetCount}`);
    print("zones:");
    print(zonesPretty);
    print("r2 buckets:");
    print(bucketsPretty);
    break;
  }
  case "manage-plan": {
    rejectExtraArg(rest[0], "cloudflare: manage-plan does not accept arguments");
    await printManageRedirectPlan(await loadManageRedirectRules());
    break;
  }
  case "manage-inspect": {
    rejectExtraArg(rest[0], "cloudflare: manage-inspect does not accept arguments");
    const zoneName = await runsealText(["@tool", "cloudflare", "config", "get", "zone_name"]);
    const zone = await runsealText(["@tool", "cloudflare", "zone", "get", "--name", zoneName]);
    const zoneId = await jsonGet(zone, ".id");
    const rulesets = await runsealText([
      "@tool",
      "cloudflare",
      "zone",
      "ruleset",
      "list",
      "--zone-id",
      zoneId,
    ]);
    const ruleset = await runsealText([
      "@tool",
      "json",
      "find",
      rulesets,
      "phase",
      "http_request_dynamic_redirect",
    ]);
    if (ruleset === "") {
      print("manage inspect: no http_request_dynamic_redirect zone ruleset found");
      break;
    }
    const rulesetId = await jsonGet(ruleset, ".id");
    const fullRuleset = await runsealText([
      "@tool",
      "cloudflare",
      "zone",
      "ruleset",
      "get",
      "--zone-id",
      zoneId,
      "--ruleset-id",
      rulesetId,
    ]);
    const rulesetName = await jsonGet(fullRuleset, ".name");
    const rules = await jsonGet(fullRuleset, ".rules");
    const matched = await runsealText([
      "@tool",
      "json",
      "filter",
      rules,
      "ref",
      "runseal_manage_sh_redirect",
      "runseal_manage_ps1_redirect",
    ]);
    const matchedCount = await runsealText(["@tool", "json", "len", matched]);
    print(`zone id: ${zoneId}`);
    print(`ruleset id: ${rulesetId}`);
    print(`ruleset name: ${rulesetName}`);
    if (matchedCount === "0") {
      print("manage inspect: no manage redirect rules found");
      break;
    }
    const pretty = await runsealText(["@tool", "json", "pretty", "value", matched]);
    print("manage rules:");
    print(pretty);
    break;
  }
  case "manage-ensure-redirect": {
    let dryRun = false;
    if (rest[0] === "--dry-run") {
      dryRun = true;
      if (rest[1] !== undefined) {
        fail(`cloudflare: unknown manage-ensure-redirect argument: ${rest[1]}`);
      }
    } else if (rest[0] !== undefined) {
      fail(`cloudflare: unknown manage-ensure-redirect argument: ${rest[0]}`);
    }
    const rules = await loadManageRedirectRules();
    const zone = await runsealText([
      "@tool",
      "cloudflare",
      "zone",
      "get",
      "--name",
      rules.zoneName,
    ]);
    const zoneId = await jsonGet(zone, ".id");
    if (dryRun) {
      await printManageRedirectPlan(rules, zoneId);
      break;
    }
    const rulesets = await runsealText([
      "@tool",
      "cloudflare",
      "zone",
      "ruleset",
      "list",
      "--zone-id",
      zoneId,
    ]);
    let ruleset = await runsealText([
      "@tool",
      "json",
      "find",
      rulesets,
      "phase",
      "http_request_dynamic_redirect",
    ]);
    if (ruleset === "") {
      ruleset = await runsealText([
        "@tool",
        "cloudflare",
        "zone",
        "ruleset",
        "create",
        "--zone-id",
        zoneId,
        "--phase",
        "http_request_dynamic_redirect",
        "--name",
        "Single Redirects ruleset",
      ]);
    } else {
      const rulesetId = await jsonGet(ruleset, ".id");
      ruleset = await runsealText([
        "@tool",
        "cloudflare",
        "zone",
        "ruleset",
        "get",
        "--zone-id",
        zoneId,
        "--ruleset-id",
        rulesetId,
      ]);
    }
    const rulesetId = await jsonGet(ruleset, ".id");
    const existingRules = await jsonGet(ruleset, ".rules");
    const currentSh = await runsealText([
      "@tool",
      "json",
      "find",
      existingRules,
      "ref",
      "runseal_manage_sh_redirect",
    ]);
    const currentPs1 = await runsealText([
      "@tool",
      "json",
      "find",
      existingRules,
      "ref",
      "runseal_manage_ps1_redirect",
    ]);
    let changedSh: string;
    if (currentSh === "") {
      await runseal([
        "@tool",
        "cloudflare",
        "zone",
        "ruleset",
        "rule",
        "add",
        "--zone-id",
        zoneId,
        "--ruleset-id",
        rulesetId,
        "--json",
        rules.ruleSh,
      ]);
      changedSh = "created runseal_manage_sh_redirect";
    } else {
      const ruleId = await jsonGet(currentSh, ".id");
      await runseal([
        "@tool",
        "cloudflare",
        "zone",
        "ruleset",
        "rule",
        "update",
        "--zone-id",
        zoneId,
        "--ruleset-id",
        rulesetId,
        "--rule-id",
        ruleId,
        "--json",
        rules.ruleSh,
      ]);
      changedSh = "updated runseal_manage_sh_redirect";
    }
    let changedPs1: string;
    if (currentPs1 === "") {
      await runseal([
        "@tool",
        "cloudflare",
        "zone",
        "ruleset",
        "rule",
        "add",
        "--zone-id",
        zoneId,
        "--ruleset-id",
        rulesetId,
        "--json",
        rules.rulePs1,
      ]);
      changedPs1 = "created runseal_manage_ps1_redirect";
    } else {
      const ruleId = await jsonGet(currentPs1, ".id");
      await runseal([
        "@tool",
        "cloudflare",
        "zone",
        "ruleset",
        "rule",
        "update",
        "--zone-id",
        zoneId,
        "--ruleset-id",
        rulesetId,
        "--rule-id",
        ruleId,
        "--json",
        rules.rulePs1,
      ]);
      changedPs1 = "updated runseal_manage_ps1_redirect";
    }
    print("manage ensure redirect: ok");
    print(`  - ${changedSh}`);
    print(`  - ${changedPs1}`);
    break;
  }
  case "api": {
    if (rest[0] === undefined) {
      fail("cloudflare: api requires a method");
    }
    if (rest[1] === undefined) {
      fail("cloudflare: api requires a path");
    }
    await runseal(["@tool", "cloudflare", "api", "request", ...rest]);
    break;
  }
  default:
    fail(`cloudflare: unknown command: ${command}`);
}
