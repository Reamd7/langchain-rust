use thiserror::Error;

use crate::{chain::ChainError, language_models::LLMError, prompt::PromptError};

// 定义一个枚举类型 `AgentError`，用于表示代理可能遇到的各种错误
#[derive(Error, Debug)]
pub enum AgentError {
    // 表示语言模型错误，使用 `LLMError` 类型
    #[error("LLM error: {0}")]
    LLMError(#[from] LLMError),

    // 表示链错误，使用 `ChainError` 类型
    #[error("Chain error: {0}")]
    ChainError(#[from] ChainError),

    // 表示提示错误，使用 `PromptError` 类型
    #[error("Prompt error: {0}")]
    PromptError(#[from] PromptError),

    // 表示工具错误，使用 `String` 类型
    #[error("Tool error: {0}")]
    ToolError(String),

    // 表示在构建器中缺少对象，使用 `String` 类型
    #[error("Missing Object On Builder: {0}")]
    MissingObject(String),

    // 表示缺少输入变量，使用 `String` 类型
    #[error("Missing input variable: {0}")]
    MissingInputVariable(String),

    // 表示 Serde JSON 错误，使用 `serde_json::Error` 类型
    #[error("Serde json error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),

    // 表示其他错误，使用 `String` 类型
    #[error("Error: {0}")]
    OtherError(String),
}
