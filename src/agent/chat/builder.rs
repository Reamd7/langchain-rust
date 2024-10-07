use std::sync::Arc;

use crate::{
    agent::AgentError,
    chain::{llm_chain::LLMChainBuilder, options::ChainCallOptions},
    language_models::llm::LLM,
    tools::Tool,
};

use super::{
    output_parser::ChatOutputParser,
    prompt::{PREFIX, SUFFIX},
    ConversationalAgent,
};

/// 构建 `ConversationalAgent` 的构建器结构体
pub struct ConversationalAgentBuilder {
    /// 可选的工具列表
    tools: Option<Vec<Arc<dyn Tool>>>,
    /// 可选的前缀字符串
    prefix: Option<String>,
    /// 可选的后缀字符串
    suffix: Option<String>,
    /// 可选的链调用选项
    options: Option<ChainCallOptions>,
}

impl ConversationalAgentBuilder {
    /// 创建一个新的 `ConversationalAgentBuilder` 实例
    pub fn new() -> Self {
        Self {
            tools: None,
            prefix: None,
            suffix: None,
            options: None,
        }
    }

    /// 设置工具列表
    pub fn tools(mut self, tools: &[Arc<dyn Tool>]) -> Self {
        self.tools = Some(tools.to_vec());
        self
    }

    /// 设置前缀字符串
    pub fn prefix<S: Into<String>>(mut self, prefix: S) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// 设置后缀字符串
    pub fn suffix<S: Into<String>>(mut self, suffix: S) -> Self {
        self.suffix = Some(suffix.into());
        self
    }

    /// 设置链调用选项
    pub fn options(mut self, options: ChainCallOptions) -> Self {
        self.options = Some(options);
        self
    }

    /// 构建 `ConversationalAgent` 实例
    pub fn build<L: Into<Box<dyn LLM>>>(self, llm: L) -> Result<ConversationalAgent, AgentError> {
        // 获取工具列表，如果没有设置则使用默认值
        let tools = self.tools.unwrap_or_default();
        // 获取前缀字符串，如果没有设置则使用默认值
        let prefix = self.prefix.unwrap_or_else(|| PREFIX.to_string());
        // 获取后缀字符串，如果没有设置则使用默认值
        let suffix = self.suffix.unwrap_or_else(|| SUFFIX.to_string());

        // 创建提示信息
        let prompt = ConversationalAgent::create_prompt(&tools, &suffix, &prefix)?;
        // 创建默认的链调用选项，并设置最大令牌数为1000
        let default_options = ChainCallOptions::default().with_max_tokens(1000);
        // 构建 LLMChain
        let chain = Box::new(
            LLMChainBuilder::new()
                .prompt(prompt)
                .llm(llm)
                .options(self.options.unwrap_or(default_options))
                .build()?,
        );

        // 返回构建好的 `ConversationalAgent` 实例
        Ok(ConversationalAgent {
            chain,
            tools,
            output_parser: ChatOutputParser::new(),
        })
    }
}
