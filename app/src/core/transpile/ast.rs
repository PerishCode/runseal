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
    ExecWrite {
        stream: OutputStream,
        path: Value,
        append: bool,
        argv: Vec<Value>,
    },
    ExecChecked {
        argv: Vec<Value>,
    },
    EnvExecChecked {
        env: Vec<EnvAssign>,
        argv: Vec<Value>,
    },
    Shift {
        count: usize,
    },
    ArgvParse {
        specs: Vec<ArgvSpec>,
    },
    CaptureChecked {
        name: String,
        argv: Vec<Value>,
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
pub struct EnvAssign {
    pub name: String,
    pub value: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ArgvKind {
    String,
    Flag,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Value {
    Literal { text: String },
    Argc,
    Var { name: String },
    Args,
    Env { name: String },
    EnvDefault { name: String, default: String },
    Concat { parts: Vec<Value> },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Predicate {
    Command { argv: Vec<Value> },
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OutputStream {
    Stdout,
    Stderr,
}
use serde::{Deserialize, Serialize};
