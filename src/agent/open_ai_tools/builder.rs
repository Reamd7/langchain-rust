use std::sync::Arc;

use crate::{
    agent::AgentError,
    chain::{options::ChainCallOptions, LLMChainBuilder},
    language_models::{llm::LLM, options::CallOptions},
    schemas::FunctionDefinition,
    tools::Tool,
};

use super::{prompt::PREFIX, OpenAiToolAgent};

// 定义 OpenAiToolAgentBuilder 结构体，用于构建 OpenAiToolAgent 实例
pub struct OpenAiToolAgentBuilder {
    // 可选的工具列表，每个工具都是一个 Arc<dyn Tool> 类型
    tools: Option<Vec<Arc<dyn Tool>>>,
    // 可选的前缀字符串
    prefix: Option<String>,
    // 可选的链调用选项
    options: Option<ChainCallOptions>,
}

impl OpenAiToolAgentBuilder {
    // 创建一个新的 OpenAiToolAgentBuilder 实例
    pub fn new() -> Self {
        Self {
            tools: None,
            prefix: None,
            options: None,
        }
    }

    // 设置工具列表
    pub fn tools(mut self, tools: &[Arc<dyn Tool>]) -> Self {
        self.tools = Some(tools.to_vec());
        self
    }

    // 设置前缀字符串
    pub fn prefix<S: Into<String>>(mut self, prefix: S) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    // 设置链调用选项
    pub fn options(mut self, options: ChainCallOptions) -> Self {
        self.options = Some(options);
        self
    }

    // 构建 OpenAiToolAgent 实例
    pub fn build<L: LLM + 'static>(self, llm: L) -> Result<OpenAiToolAgent, AgentError> {
        // 获取工具列表，如果没有设置则使用空列表
        let tools = self.tools.unwrap_or_default();
        // 获取前缀字符串，如果没有设置则使用默认前缀
        let prefix = self.prefix.unwrap_or_else(|| PREFIX.to_string());
        // 获取 LLM 实例
        let mut llm = llm;

        // 创建提示信息
        let prompt = OpenAiToolAgent::create_prompt(&prefix)?;
        // 创建默认的链调用选项，并设置最大令牌数为 1000
        let default_options = ChainCallOptions::default().with_max_tokens(1000);
        // 将工具列表转换为 FunctionDefinition 列表
        let functions = tools
            .iter()
            .map(FunctionDefinition::from_langchain_tool)
            .collect::<Vec<FunctionDefinition>>();
        // 为 LLM 添加调用选项
        llm.add_options(CallOptions::new().with_functions(functions));
        // 构建 LLMChain 实例
        let chain = Box::new(
            LLMChainBuilder::new()
                .prompt(prompt)
                .llm(llm)
                .options(self.options.unwrap_or(default_options))
                .build()?,
        );

        // 返回构建好的 OpenAiToolAgent 实例
        Ok(OpenAiToolAgent { chain, tools })
    }
}
