use snafu::prelude::*;

/// Each variant wraps the line numberi the error was found on
#[derive(Debug, Clone, strum::Display)]
pub enum ET {
    ExpectedCParen,
    ExpectedExpression,
    ExpectedId,
    UndeclaredId,
    ExpectedAssignment,
    ExpectedStatement,
    ExpectedCondition,
    ExpectedOParen,
    ExpectedCCurl,
    ExpectedStrLiteral,
    ExpectedType,
    ExpectedSemi,
}

#[derive(Debug, Clone)]
pub struct RhErr {
    pub err: ET,
    pub line: i32,
}
