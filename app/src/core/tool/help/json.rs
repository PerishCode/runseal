use super::{Entry, Section};

pub const JSON: Entry = Entry {
    key: "json",
    usage: "runseal @tool json <command> [args]",
    about: None,
    sections: &[Section {
        title: "JSON helpers",
        items: &[
            ("get <json> <path>", "print one JSON value"),
            ("empty <json>", "print true when JSON length is zero"),
            ("len <json>", "print JSON array/object/string length"),
            ("pretty <mode> ...", "print formatted JSON"),
            (
                "find <array> <field> <value>",
                "print first object with field=value",
            ),
            (
                "filter <array> <field> <value>...",
                "print objects with field matching values",
            ),
        ],
    }],
    examples: &[],
};

pub const JSON_GET: Entry = Entry {
    key: "json.get",
    usage: "runseal @tool json get <json> <path>",
    about: Some("Print one JSON value selected by the path expression."),
    sections: &[Section {
        title: "Arguments",
        items: &[
            ("<json>", "input JSON text"),
            ("<path>", "path expression such as `.[0].databaseId`"),
        ],
    }],
    examples: &["runseal @tool json get '[{\"databaseId\":123}]' '.[0].databaseId'"],
};

pub const JSON_EMPTY: Entry = Entry {
    key: "json.empty",
    usage: "runseal @tool json empty <json>",
    about: Some("Print `true` when the JSON array, object, or string length is zero."),
    sections: &[],
    examples: &["runseal @tool json empty '[]'"],
};

pub const JSON_LEN: Entry = Entry {
    key: "json.len",
    usage: "runseal @tool json len <json>",
    about: Some("Print the JSON array, object, or string length as an integer."),
    sections: &[],
    examples: &["runseal @tool json len '[1,2,3]'"],
};

pub const JSON_PRETTY: Entry = Entry {
    key: "json.pretty",
    usage: "runseal @tool json pretty <mode> [args]",
    about: Some("Print formatted JSON with explicit input mode selection."),
    sections: &[Section {
        title: "Modes",
        items: &[
            ("value <json>", "pretty-print one JSON argument"),
            ("stdin", "read JSON from stdin and print formatted output"),
            (
                "file <input> <output>",
                "read one file and write formatted JSON",
            ),
        ],
    }],
    examples: &[
        "runseal @tool json pretty value '{\"a\":1}'",
        "echo '{\"a\":1}' | runseal @tool json pretty stdin",
        "runseal @tool json pretty file input.json output.json",
    ],
};

pub const JSON_PRETTY_VALUE: Entry = Entry {
    key: "json.pretty.value",
    usage: "runseal @tool json pretty value <json>",
    about: Some("Pretty-print one JSON argument with indentation."),
    sections: &[],
    examples: &["runseal @tool json pretty value '{\"a\":1}'"],
};

pub const JSON_PRETTY_STDIN: Entry = Entry {
    key: "json.pretty.stdin",
    usage: "runseal @tool json pretty stdin",
    about: Some("Read JSON from stdin and print formatted JSON."),
    sections: &[],
    examples: &["echo '{\"a\":1}' | runseal @tool json pretty stdin"],
};

pub const JSON_PRETTY_FILE: Entry = Entry {
    key: "json.pretty.file",
    usage: "runseal @tool json pretty file <input> <output>",
    about: Some("Read one JSON file and write formatted JSON to the output path."),
    sections: &[Section {
        title: "Arguments",
        items: &[
            ("<input>", "source JSON file path"),
            ("<output>", "destination file path"),
        ],
    }],
    examples: &["runseal @tool json pretty file input.json output.json"],
};

pub const JSON_FIND: Entry = Entry {
    key: "json.find",
    usage: "runseal @tool json find <array> <field> <value>",
    about: Some("Print the first object in the JSON array with `<field> == <value>`."),
    sections: &[],
    examples: &["runseal @tool json find '[{\"id\":1}]' id 1"],
};

pub const JSON_FILTER: Entry = Entry {
    key: "json.filter",
    usage: "runseal @tool json filter <array> <field> <value>...",
    about: Some("Print objects in the JSON array whose `<field>` matches any provided value."),
    sections: &[],
    examples: &["runseal @tool json filter '[{\"env\":\"dev\"},{\"env\":\"prod\"}]' env dev prod"],
};
