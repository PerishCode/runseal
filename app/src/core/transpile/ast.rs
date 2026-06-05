#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Program {
    pub version: u32,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Item {
    Function { name: String, body: Vec<Statement> },
    Statement { statement: Statement },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Statement {
    Assign {
        name: String,
        value: Value,
    },
    ExecChecked {
        argv: Vec<Value>,
    },
    CaptureChecked {
        name: String,
        argv: Vec<Value>,
    },
    StringTrim {
        name: String,
        value: Value,
    },
    JsonGet {
        name: String,
        json: Value,
        path: JsonPath,
    },
    If {
        predicate: Predicate,
        then_body: Vec<Statement>,
        else_body: Vec<Statement>,
    },
    Case {
        value: Value,
        arms: Vec<CaseArm>,
    },
    CallFunction {
        name: String,
        argv: Vec<Value>,
    },
    Print {
        value: Value,
    },
    Error {
        value: Value,
    },
    Fail {
        value: Value,
    },
    Exit {
        code: i32,
    },
    Sleep {
        seconds: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CaseArm {
    pub patterns: Vec<String>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JsonPath {
    pub segments: Vec<JsonPathSegment>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum JsonPathSegment {
    Field { name: String },
    Index { index: usize },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Value {
    Literal { text: String },
    Var { name: String },
    Env { name: String },
    EnvDefault { name: String, default: String },
    Concat { parts: Vec<Value> },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Predicate {
    Empty { value: Value },
    NotEmpty { value: Value },
    Eq { left: Value, right: Value },
    Neq { left: Value, right: Value },
    FileExists { path: Value },
    DirExists { path: Value },
    ToolExists { name: String },
}
use serde::{Deserialize, Serialize};
