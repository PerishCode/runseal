use super::{Entry, Section};

pub const HASH: Entry = Entry {
    key: "hash",
    usage: "runseal @tool hash <command> [args]",
    about: None,
    sections: &[Section {
        title: "Hash helpers",
        items: &[(
            "tree <path>...",
            "hash one or more file trees deterministically",
        )],
    }],
    examples: &[],
};

pub const HASH_TREE: Entry = Entry {
    key: "hash.tree",
    usage: "runseal @tool hash tree <path>...",
    about: Some("Hash one or more file trees using stable path and content ordering."),
    sections: &[],
    examples: &["runseal @tool hash tree app/tests .runseal/wrappers"],
};

pub const VERSION: Entry = Entry {
    key: "version",
    usage: "runseal @tool version <command> [args]",
    about: None,
    sections: &[Section {
        title: "Version helpers",
        items: &[
            (
                "part <version> <major|minor|patch>",
                "print one stable semantic version part",
            ),
            (
                "compare <left> <right>",
                "compare two stable semantic versions",
            ),
        ],
    }],
    examples: &[],
};

pub const VERSION_PART: Entry = Entry {
    key: "version.part",
    usage: "runseal @tool version part <version> <major|minor|patch>",
    about: Some("Print one numeric part from a stable semantic version, with optional `v` prefix."),
    sections: &[],
    examples: &["runseal @tool version part v0.7.0 minor"],
};

pub const VERSION_COMPARE: Entry = Entry {
    key: "version.compare",
    usage: "runseal @tool version compare <left> <right>",
    about: Some("Compare two stable semantic versions and print `lt`, `eq`, or `gt`."),
    sections: &[],
    examples: &["runseal @tool version compare 0.6.1 0.6.0"],
};
