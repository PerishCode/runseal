use super::{Entry, Section};

pub const SSH: Entry = Entry {
    key: "ssh",
    usage: "runseal @tool ssh <scope> <command> [args]",
    about: None,
    sections: &[Section {
        title: "SSH helpers",
        items: &[
            (
                "config host <host> --config <path>",
                "print true when Host patterns match",
            ),
            (
                "config identities --config <path>",
                "print IdentityFile paths as JSON",
            ),
            (
                "script run --config <path>",
                "run a local script on an SSH host",
            ),
            (
                "script capture --config <path>",
                "run a local script and print stdout",
            ),
        ],
    }],
    examples: &[],
};

pub const SSH_CONFIG: Entry = Entry {
    key: "ssh.config",
    usage: "runseal @tool ssh config <command> [args]",
    about: None,
    sections: &[Section {
        title: "SSH config helpers",
        items: &[
            (
                "host <host> --config <path>",
                "print true when any Host pattern matches",
            ),
            (
                "identities --config <path> [--base <path>]",
                "print IdentityFile paths as JSON",
            ),
        ],
    }],
    examples: &[
        "runseal @tool ssh config host bandwagon --config ~/.ssh/config",
        "runseal @tool ssh config identities --config ~/.ssh/config",
    ],
};

pub const SSH_CONFIG_IDENTITIES: Entry = Entry {
    key: "ssh.config.identities",
    usage: "runseal @tool ssh config identities --config <path> [--base <path>]",
    about: Some(
        "Print IdentityFile entries as a JSON string array. Relative IdentityFile paths resolve from `--base`, or from the config directory by default.",
    ),
    sections: &[Section {
        title: "Flags",
        items: &[
            ("--config <path>", "SSH config file to inspect"),
            (
                "--base <path>",
                "resolve relative IdentityFile paths from this base path",
            ),
        ],
    }],
    examples: &["runseal @tool ssh config identities --config .local/ssh/config --base .local/ssh"],
};

pub const SSH_CONFIG_HOST: Entry = Entry {
    key: "ssh.config.host",
    usage: "runseal @tool ssh config host <host> --config <path>",
    about: Some(
        "Print `true` when `<host>` matches any declared `Host` pattern in the SSH config.",
    ),
    sections: &[Section {
        title: "Flags",
        items: &[("--config <path>", "SSH config file to inspect")],
    }],
    examples: &["runseal @tool ssh config host 10m.hk.zxi --config .local/ssh/config"],
};

pub const SSH_SCRIPT: Entry = Entry {
    key: "ssh.script",
    usage: "runseal @tool ssh script <command> [options] -- [args...]",
    about: None,
    sections: &[Section {
        title: "SSH script helpers",
        items: &[
            (
                "run --config <path> --host <host> --file <path> -- [args...]",
                "send a local script to `ssh host bash -s -- ...`",
            ),
            (
                "capture --config <path> --host <host> --file <path> -- [args...]",
                "run the script and print stdout",
            ),
        ],
    }],
    examples: &[
        "runseal @tool ssh script run --config .local/ssh/config --host 10m.hk.zxi --file scripts/check.sh -- arg1 arg2",
    ],
};

pub const SSH_SCRIPT_RUN: Entry = Entry {
    key: "ssh.script.run",
    usage: "runseal @tool ssh script run --config <path> --host <host> --file <path> -- [args...]",
    about: Some("Send one local script file to `ssh -F <config> <host> bash -s -- ...`."),
    sections: &[Section {
        title: "Flags",
        items: &[
            ("--config <path>", "SSH config file to use"),
            (
                "--host <host>",
                "target host; must match a declared Host pattern",
            ),
            ("--file <path>", "local script file to stream over stdin"),
        ],
    }],
    examples: &[
        "runseal @tool ssh script run --config .local/ssh/config --host 10m.hk.zxi --file scripts/check.sh -- arg1 arg2",
    ],
};

pub const SSH_SCRIPT_CAPTURE: Entry = Entry {
    key: "ssh.script.capture",
    usage: "runseal @tool ssh script capture --config <path> --host <host> --file <path> -- [args...]",
    about: Some("Run one local script on the SSH host and print stdout to the caller."),
    sections: &[Section {
        title: "Flags",
        items: &[
            ("--config <path>", "SSH config file to use"),
            (
                "--host <host>",
                "target host; must match a declared Host pattern",
            ),
            ("--file <path>", "local script file to stream over stdin"),
        ],
    }],
    examples: &[
        "runseal @tool ssh script capture --config .local/ssh/config --host 10m.hk.zxi --file scripts/uptime.sh",
    ],
};
