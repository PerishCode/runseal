use super::{Entry, Section};

pub const STRING: Entry = Entry {
    key: "string",
    usage: "runseal @tool string <command> [args]",
    about: None,
    sections: &[Section {
        title: "String helpers",
        items: &[
            ("trim <value>", "trim leading and trailing whitespace"),
            (
                "join <json-array> --separator <text>",
                "join a JSON string array",
            ),
            ("slug <value>", "normalize text for branch/file slugs"),
        ],
    }],
    examples: &[],
};

pub const STRING_TRIM: Entry = Entry {
    key: "string.trim",
    usage: "runseal @tool string trim <value>",
    about: None,
    sections: &[],
    examples: &[],
};

pub const STRING_JOIN: Entry = Entry {
    key: "string.join",
    usage: "runseal @tool string join <json-array> --separator <text|path>",
    about: None,
    sections: &[],
    examples: &[],
};

pub const STRING_SLUG: Entry = Entry {
    key: "string.slug",
    usage: "runseal @tool string slug <value> [--max-len <n>] [--fallback <text>]",
    about: None,
    sections: &[],
    examples: &[],
};

pub const REGEX: Entry = Entry {
    key: "regex",
    usage: "runseal @tool regex <command> [args]",
    about: None,
    sections: &[Section {
        title: "Regex helpers",
        items: &[(
            "capture <value> <pattern> <group>",
            "print regex capture group n, or empty",
        )],
    }],
    examples: &[],
};

pub const REGEX_CAPTURE: Entry = Entry {
    key: "regex.capture",
    usage: "runseal @tool regex capture <value> <pattern> <group>",
    about: None,
    sections: &[],
    examples: &[],
};

pub const INT: Entry = Entry {
    key: "int",
    usage: "runseal @tool int <command> [args]",
    about: None,
    sections: &[Section {
        title: "Integer helpers",
        items: &[("add <left> <right>", "print integer sum")],
    }],
    examples: &[],
};

pub const INT_ADD: Entry = Entry {
    key: "int.add",
    usage: "runseal @tool int add <left> <right>",
    about: None,
    sections: &[],
    examples: &[],
};

pub const PROCESS: Entry = Entry {
    key: "process",
    usage: "runseal @tool process <command> [args]",
    about: None,
    sections: &[Section {
        title: "Process helpers",
        items: &[
            ("exists <name>", "print true when command exists on PATH"),
            (
                "write <stdout|stderr> <path> [--append] -- <command> [args...]",
                "run one command and write one stream to a file",
            ),
        ],
    }],
    examples: &[],
};

pub const PROCESS_EXISTS: Entry = Entry {
    key: "process.exists",
    usage: "runseal @tool process exists <name>",
    about: None,
    sections: &[],
    examples: &[],
};

pub const PROCESS_WRITE: Entry = Entry {
    key: "process.write",
    usage: "runseal @tool process write <stdout|stderr> <path> [--append] -- <command> [args...]",
    about: Some(
        "Run one command, write one selected stream to a file, and pass the other stream through.",
    ),
    sections: &[Section {
        title: "Flags",
        items: &[("--append", "append instead of overwriting the target file")],
    }],
    examples: &[
        "runseal @tool process write stdout openapi.json -- cargo run -- export-openapi",
        "runseal @tool process write stderr build.log --append -- cargo build",
    ],
};

pub const ARCHIVE: Entry = Entry {
    key: "archive",
    usage: "runseal @tool archive <scope> <command> [args]",
    about: None,
    sections: &[Section {
        title: "Archive helpers",
        items: &[
            ("local export", "encrypt a .local-style directory archive"),
            ("local import", "decrypt and restore a .local-style archive"),
        ],
    }],
    examples: &[],
};

pub const ARCHIVE_LOCAL: Entry = Entry {
    key: "archive.local",
    usage: "runseal @tool archive local <command> [args]",
    about: None,
    sections: &[Section {
        title: "Archive local helpers",
        items: &[
            (
                "export --source <dir> --archive <path> (--password <text>|--password-env <name>) [--force]",
                "encrypt one source directory",
            ),
            (
                "import --source <dir> --archive <path> (--password <text>|--password-env <name>) [--force]",
                "decrypt one archive into the source directory",
            ),
        ],
    }],
    examples: &[],
};

pub const ARCHIVE_LOCAL_EXPORT: Entry = Entry {
    key: "archive.local.export",
    usage: "runseal @tool archive local export --source <dir> --archive <path> (--password <text>|--password-env <name>)",
    about: Some("Encrypt one .local-style directory archive."),
    sections: &[Section {
        title: "Flags",
        items: &[
            (
                "--source <dir>",
                "source directory to export or import into",
            ),
            ("--archive <path>", "archive file to write or read"),
            ("--password <text>", "explicit archive password"),
            (
                "--password-env <name>",
                "read the password from one environment variable",
            ),
        ],
    }],
    examples: &[
        "runseal @tool archive local export --source .local --archive backup.seal --password-env ESTATE_LOCAL_PASSWORD",
    ],
};

pub const ARCHIVE_LOCAL_IMPORT: Entry = Entry {
    key: "archive.local.import",
    usage: "runseal @tool archive local import --source <dir> --archive <path> (--password <text>|--password-env <name>) [--force]",
    about: Some("Decrypt one .local-style directory archive into the source directory."),
    sections: &[Section {
        title: "Flags",
        items: &[
            ("--source <dir>", "destination directory to restore into"),
            ("--archive <path>", "archive file to read"),
            ("--password <text>", "explicit archive password"),
            (
                "--password-env <name>",
                "read the password from one environment variable",
            ),
            (
                "--force",
                "allow replacing an existing destination directory",
            ),
        ],
    }],
    examples: &[
        "runseal @tool archive local import --source .local --archive backup.seal --password-env ESTATE_LOCAL_PASSWORD --force",
    ],
};

pub const FS: Entry = Entry {
    key: "fs",
    usage: "runseal @tool fs <command> [args]",
    about: None,
    sections: &[Section {
        title: "Filesystem helpers",
        items: &[
            ("mkdir <path> [mode]", "create a directory and parents"),
            ("write <path> <text> [mode]", "write text to a file"),
            (
                "write-base64 <path> <base64>",
                "write decoded bytes to a file",
            ),
            ("chmod <path> <mode>", "set a file mode on Unix"),
            ("mode <path>", "print file mode on Unix"),
            ("touch <path> [mode]", "create a file without truncating it"),
            (
                "list <path> [--glob <pattern>]",
                "list matching direct children as JSON",
            ),
            (
                "contains-any <path> <text>...",
                "print true when file contains any text",
            ),
            (
                "backup-numbered <path>",
                "move path to .bak or .bak.N and print it",
            ),
        ],
    }],
    examples: &[],
};

pub const FS_LIST: Entry = Entry {
    key: "fs.list",
    usage: "runseal @tool fs list <path> [--glob <pattern>] [--files] [--dirs] [--require-nonempty]",
    about: Some("List matching direct children and print canonical paths as a JSON array."),
    sections: &[Section {
        title: "Flags",
        items: &[
            (
                "--glob <pattern>",
                "match file names with a simple glob; default `*`",
            ),
            ("--files", "include files"),
            ("--dirs", "include directories"),
            ("--require-nonempty", "fail when the match set is empty"),
        ],
    }],
    examples: &["runseal @tool fs list .local/secrets --glob '*.env' --files"],
};

pub const GITEE: Entry = Entry {
    key: "gitee",
    usage: "runseal @tool gitee <scope> <command> [args]",
    about: None,
    sections: &[Section {
        title: "Gitee helpers",
        items: &[
            (
                "repo parse-origin <url>",
                "parse owner/repo from origin URL",
            ),
            ("pr find", "find one pull request by branch filters"),
            ("pr create", "create a pull request"),
            ("pr pass-gates", "best-effort pass PR gates"),
            ("pr merge", "merge a pull request"),
        ],
    }],
    examples: &[],
};

pub const GITEE_REPO: Entry = Entry {
    key: "gitee.repo",
    usage: "runseal @tool gitee repo <command> [args]",
    about: None,
    sections: &[Section {
        title: "Gitee repo helpers",
        items: &[(
            "parse-origin <url>",
            "parse owner/repo from a Gitee origin URL",
        )],
    }],
    examples: &[],
};

pub const GITEE_REPO_PARSE_ORIGIN: Entry = Entry {
    key: "gitee.repo.parse-origin",
    usage: "runseal @tool gitee repo parse-origin <url>",
    about: None,
    sections: &[],
    examples: &[],
};

pub const GITEE_PR: Entry = Entry {
    key: "gitee.pr",
    usage: "runseal @tool gitee pr <command> [args]",
    about: None,
    sections: &[Section {
        title: "Gitee PR helpers",
        items: &[
            ("find", "find one Gitee pull request"),
            ("create", "create a Gitee pull request"),
            ("pass-gates", "best-effort pass PR gates"),
            ("merge", "merge a Gitee pull request"),
        ],
    }],
    examples: &[],
};

pub const GITEE_PR_FIND: Entry = Entry {
    key: "gitee.pr.find",
    usage: "runseal @tool gitee pr find --owner <name> --repo <name> --head <branch> [--base <branch>] [--state <open|merged|closed|all>] [--token <text>|--token-file <path>|--token-env <name>]",
    about: Some(
        "Find one Gitee pull request by branch filters. Prints the PR JSON object or `null` when no match exists.",
    ),
    sections: &[Section {
        title: "Flags",
        items: &[
            ("--owner <name>", "Gitee repository owner"),
            ("--repo <name>", "Gitee repository name"),
            ("--head <branch>", "source branch to match"),
            ("--base <branch>", "optional target branch filter"),
            (
                "--state <open|merged|closed|all>",
                "PR state filter; default `open`",
            ),
            ("--token <text>", "explicit Gitee token"),
            (
                "--token-file <path>",
                "env-style file containing `GITEE_TOKEN`",
            ),
            (
                "--token-env <name>",
                "read the token from one named environment variable; default fallback is `GITEE_TOKEN`",
            ),
        ],
    }],
    examples: &[
        "runseal @tool gitee pr find --owner perishme --repo perish.top --head auto/demo",
        "runseal @tool gitee pr find --owner perishme --repo perish.top --head feat/resume --base main --state open",
    ],
};

pub const GITEE_PR_CREATE: Entry = Entry {
    key: "gitee.pr.create",
    usage: "runseal @tool gitee pr create --owner <name> --repo <name> --base <branch> --head <branch> --title <text> --body <text> [--token <text>|--token-file <path>|--token-env <name>]",
    about: Some("Create a Gitee pull request and print the API response JSON."),
    sections: &[Section {
        title: "Flags",
        items: &[
            ("--owner <name>", "Gitee repository owner"),
            ("--repo <name>", "Gitee repository name"),
            ("--base <branch>", "target branch"),
            ("--head <branch>", "source branch"),
            ("--title <text>", "pull request title"),
            ("--body <text>", "pull request body"),
            ("--token <text>", "explicit Gitee token"),
            (
                "--token-file <path>",
                "env-style file containing `GITEE_TOKEN`",
            ),
            (
                "--token-env <name>",
                "read the token from one named environment variable; default fallback is `GITEE_TOKEN`",
            ),
        ],
    }],
    examples: &[
        "runseal @tool gitee pr create --owner perishme --repo perish.top --base main --head auto/demo --title demo --body demo",
    ],
};

pub const GITEE_PR_PASS_GATES: Entry = Entry {
    key: "gitee.pr.pass-gates",
    usage: "runseal @tool gitee pr pass-gates --owner <name> --repo <name> --number <n> [--token <text>|--token-file <path>|--token-env <name>]",
    about: Some("Best-effort pass available Gitee PR gates and print the result JSON."),
    sections: &[Section {
        title: "Flags",
        items: &[
            ("--owner <name>", "Gitee repository owner"),
            ("--repo <name>", "Gitee repository name"),
            ("--number <n>", "pull request number"),
            ("--token <text>", "explicit Gitee token"),
            (
                "--token-file <path>",
                "env-style file containing `GITEE_TOKEN`",
            ),
            (
                "--token-env <name>",
                "read the token from one named environment variable; default fallback is `GITEE_TOKEN`",
            ),
        ],
    }],
    examples: &[
        "runseal @tool gitee pr pass-gates --owner perishme --repo perish.top --number 123",
    ],
};

pub const GITEE_PR_MERGE: Entry = Entry {
    key: "gitee.pr.merge",
    usage: "runseal @tool gitee pr merge --owner <name> --repo <name> --number <n> [--method <merge|rebase|squash>] [--token <text>|--token-file <path>|--token-env <name>]",
    about: Some("Merge a Gitee pull request and print the API response JSON."),
    sections: &[Section {
        title: "Flags",
        items: &[
            ("--owner <name>", "Gitee repository owner"),
            ("--repo <name>", "Gitee repository name"),
            ("--number <n>", "pull request number"),
            (
                "--method <merge|rebase|squash>",
                "merge method; default `squash`",
            ),
            ("--token <text>", "explicit Gitee token"),
            (
                "--token-file <path>",
                "env-style file containing `GITEE_TOKEN`",
            ),
            (
                "--token-env <name>",
                "read the token from one named environment variable; default fallback is `GITEE_TOKEN`",
            ),
        ],
    }],
    examples: &[
        "runseal @tool gitee pr merge --owner perishme --repo perish.top --number 123 --method squash",
    ],
};
