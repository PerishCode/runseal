const decoder = new TextDecoder();
const encoder = new TextEncoder();

export type CommandOptions = {
  cwd?: string;
  env?: Record<string, string>;
  stdin?: "inherit" | "null" | "piped";
  stdout?: "inherit" | "null" | "piped";
  stderr?: "inherit" | "null" | "piped";
};

const blockedInheritedEnv = new Set([
  "DYLD_FALLBACK_LIBRARY_PATH",
  "DYLD_INSERT_LIBRARIES",
  "DYLD_LIBRARY_PATH",
  "LD_PRELOAD",
  "LD_LIBRARY_PATH",
]);

function hasBlockedInheritedEnv(): boolean {
  for (const key of blockedInheritedEnv) {
    if (Deno.env.get(key) !== undefined) {
      return true;
    }
  }
  return false;
}

function sanitizedEnv(extra: Record<string, string> | undefined): Record<string, string> {
  const env = Deno.env.toObject();
  for (const key of blockedInheritedEnv) {
    delete env[key];
  }
  return { ...env, ...(extra ?? {}) };
}

function commandEnvOptions(
  extra: Record<string, string> | undefined,
): Pick<Deno.CommandOptions, "clearEnv" | "env"> {
  if (hasBlockedInheritedEnv()) {
    return { clearEnv: true, env: sanitizedEnv(extra) };
  }
  return extra === undefined ? {} : { env: extra };
}

export function print(value = ""): void {
  console.log(value);
}

export function error(value: string): void {
  console.error(value);
}

export function fail(message: string, code = 1): never {
  error(message);
  Deno.exit(code);
}

export function env(name: string, fallback = ""): string {
  return Deno.env.get(name) ?? fallback;
}

export function requireEnv(name: string): string {
  const value = Deno.env.get(name);
  if (value === undefined || value === "") {
    fail(`missing required env: ${name}`);
  }
  return value;
}

export function isHelp(args: string[]): boolean {
  return args.length === 1 && ["-h", "--help", "help"].includes(args[0]);
}

export async function run(command: string, args: string[] = [], options: CommandOptions = {}) {
  const status = await new Deno.Command(command, {
    args,
    cwd: options.cwd,
    ...commandEnvOptions(options.env),
    stdin: options.stdin ?? "inherit",
    stdout: options.stdout ?? "inherit",
    stderr: options.stderr ?? "inherit",
  }).spawn().status;
  if (!status.success) {
    Deno.exit(status.code);
  }
}

export async function runText(
  command: string,
  args: string[] = [],
  options: Omit<CommandOptions, "stdout"> = {},
): Promise<string> {
  const output = await new Deno.Command(command, {
    args,
    cwd: options.cwd,
    ...commandEnvOptions(options.env),
    stdin: options.stdin ?? "null",
    stdout: "piped",
    stderr: options.stderr ?? "inherit",
  }).output();
  if (!output.success) {
    Deno.exit(output.code);
  }
  return decoder.decode(output.stdout).trimEnd();
}

export async function runInput(
  command: string,
  args: string[],
  input: string,
  options: Omit<CommandOptions, "stdin"> = {},
): Promise<string> {
  const child = new Deno.Command(command, {
    args,
    cwd: options.cwd,
    ...commandEnvOptions(options.env),
    stdin: "piped",
    stdout: options.stdout ?? "piped",
    stderr: options.stderr ?? "inherit",
  }).spawn();
  const writer = child.stdin.getWriter();
  await writer.write(encoder.encode(input));
  await writer.close();
  const output = await child.output();
  if (!output.success) {
    Deno.exit(output.code);
  }
  return decoder.decode(output.stdout).trimEnd();
}

export async function runsealText(args: string[]): Promise<string> {
  return await runText("runseal", args);
}

export async function runseal(args: string[]): Promise<void> {
  await run("runseal", args);
}

export async function commandExists(name: string): Promise<boolean> {
  return (await runsealText(["@tool", "process", "exists", name])) === "true";
}

export async function jsonGet(json: string, path: string): Promise<string> {
  return await runsealText(["@tool", "json", "get", json, path]);
}

export async function jsonEmpty(json: string): Promise<boolean> {
  return (await runsealText(["@tool", "json", "empty", json])) === "true";
}

export async function fileExists(path: string): Promise<boolean> {
  try {
    const stat = await Deno.stat(path);
    return stat.isFile;
  } catch (err) {
    if (err instanceof Deno.errors.NotFound) {
      return false;
    }
    throw err;
  }
}

export async function dirExists(path: string): Promise<boolean> {
  try {
    const stat = await Deno.stat(path);
    return stat.isDirectory;
  } catch (err) {
    if (err instanceof Deno.errors.NotFound) {
      return false;
    }
    throw err;
  }
}

export function pathJoin(...parts: string[]): string {
  const separator = Deno.build.os === "windows" ? "\\" : "/";
  const joined = parts
    .filter((part) => part !== "")
    .map((part, index) =>
      index === 0 ? part.replace(/[\\/]+$/g, "") : part.replace(/^[\\/]+|[\\/]+$/g, "")
    )
    .filter((part) => part !== "")
    .join(separator);
  return joined === "" ? "." : joined;
}

export function pathListSeparator(): string {
  return Deno.build.os === "windows" ? ";" : ":";
}

export async function readTextIfExists(path: string): Promise<string> {
  try {
    return await Deno.readTextFile(path);
  } catch (err) {
    if (err instanceof Deno.errors.NotFound) {
      return "";
    }
    throw err;
  }
}
