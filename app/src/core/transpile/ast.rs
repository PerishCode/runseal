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
    ArgvParse {
        specs: Vec<ArgvSpec>,
    },
    CaptureChecked {
        name: String,
        argv: Vec<Value>,
    },
    ToolExec {
        invocation: ToolInvocation,
    },
    ToolPassthrough {
        start: usize,
        invocation: ToolInvocation,
    },
    ToolCapture {
        name: String,
        invocation: ToolInvocation,
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
    RegexCapture {
        name: String,
        value: Value,
        pattern: String,
        group: usize,
    },
    IntAdd {
        name: String,
        left: Value,
        right: Value,
    },
    If {
        predicate: Predicate,
        then_body: Vec<Statement>,
        else_body: Vec<Statement>,
    },
    While {
        predicate: Predicate,
        body: Vec<Statement>,
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
    Break,
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
pub struct ArgvSpec {
    pub name: String,
    pub kind: ArgvKind,
    pub default: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolInvocation {
    pub path: Vec<String>,
    pub argv: Vec<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ArgvKind {
    String,
    Flag,
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
    IntLt { left: Value, right: Value },
    IntLte { left: Value, right: Value },
    IntGt { left: Value, right: Value },
    IntGte { left: Value, right: Value },
    JsonEmpty { value: Value },
    JsonNotEmpty { value: Value },
    FileExists { path: Value },
    DirExists { path: Value },
    ToolExists { name: String },
}
use serde::{Deserialize, Serialize};
