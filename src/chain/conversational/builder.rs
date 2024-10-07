use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{
    chain::{
        llm_chain::LLMChainBuilder, options::ChainCallOptions, ChainError, DEFAULT_OUTPUT_KEY,
    },
    language_models::llm::LLM,
    memory::SimpleMemory,
    output_parsers::OutputParser,
    prompt::{FormatPrompter, HumanMessagePromptTemplate},
    schemas::memory::BaseMemory,
    template_fstring,
};

use super::{prompt::DEFAULT_TEMPLATE, ConversationalChain, DEFAULT_INPUT_VARIABLE};

// 定义 ConversationalChainBuilder 结构体，用于构建 ConversationalChain 实例
pub struct ConversationalChainBuilder {
    llm: Option<Box<dyn LLM>>, // 可选的语言模型
    options: Option<ChainCallOptions>, // 可选的调用选项
    memory: Option<Arc<Mutex<dyn BaseMemory>>>, // 可选的内存对象
    output_key: Option<String>, // 可选的输出键
    output_parser: Option<Box<dyn OutputParser>>, // 可选的输出解析器
    input_key: Option<String>, // 可选的输入键
    prompt: Option<Box<dyn FormatPrompter>>, // 可选的提示模板
}

impl ConversationalChainBuilder {
    // 创建一个新的 ConversationalChainBuilder 实例
    pub fn new() -> Self {
        Self {
            llm: None,
            options: None,
            memory: None,
            output_key: None,
            output_parser: None,
            input_key: None,
            prompt: None,
        }
    }

    // 设置语言模型
    pub fn llm<L: Into<Box<dyn LLM>>>(mut self, llm: L) -> Self {
        self.llm = Some(llm.into());
        self
    }

    // 设置调用选项
    pub fn options(mut self, options: ChainCallOptions) -> Self {
        self.options = Some(options);
        self
    }

    // 设置输入键
    pub fn input_key<S: Into<String>>(mut self, input_key: S) -> Self {
        self.input_key = Some(input_key.into());
        self
    }

    // 设置输出解析器
    pub fn output_parser<P: Into<Box<dyn OutputParser>>>(mut self, output_parser: P) -> Self {
        self.output_parser = Some(output_parser.into());
        self
    }

    // 设置内存对象
    pub fn memory(mut self, memory: Arc<Mutex<dyn BaseMemory>>) -> Self {
        self.memory = Some(memory);
        self
    }

    // 设置输出键
    pub fn output_key<S: Into<String>>(mut self, output_key: S) -> Self {
        self.output_key = Some(output_key.into());
        self
    }

    /// 如果你想添加自定义提示，请记住哪些变量是必需的。
    pub fn prompt<P: Into<Box<dyn FormatPrompter>>>(mut self, prompt: P) -> Self {
        self.prompt = Some(prompt.into());
        self
    }

    // 构建 ConversationalChain 实例
    pub fn build(self) -> Result<ConversationalChain, ChainError> {
        // 获取语言模型，如果未设置则返回错误
        let llm = self
            .llm
            .ok_or_else(|| ChainError::MissingObject("LLM must be set".into()))?;

        // 获取提示模板，如果未设置则使用默认模板
        let prompt = match self.prompt {
            Some(prompt) => prompt,
            None => Box::new(HumanMessagePromptTemplate::new(template_fstring!(
                DEFAULT_TEMPLATE,
                "history",
                "input"
            ))),
        };

        // 构建 LLMChain
        let llm_chain = {
            // 创建一个新的 LLMChainBuilder 实例，并设置提示模板和语言模型
            let mut builder = LLMChainBuilder::new()
                .prompt(prompt)
                .llm(llm)
                // 设置输出键，如果未设置则使用默认输出键
                .output_key(self.output_key.unwrap_or_else(|| DEFAULT_OUTPUT_KEY.into()));

            // 如果调用选项已设置，则将其添加到 builder 中
            if let Some(options) = self.options {
                builder = builder.options(options);
            }

            // 如果输出解析器已设置，则将其添加到 builder 中
            if let Some(output_parser) = self.output_parser {
                builder = builder.output_parser(output_parser);
            }

            // 构建 LLMChain 实例并返回结果
            builder.build()?
        };

        // 获取内存对象，如果未设置则使用默认的 SimpleMemory
        let memory = self
            .memory
            .unwrap_or_else(|| Arc::new(Mutex::new(SimpleMemory::new())));

        // 返回构建的 ConversationalChain 实例
        Ok(ConversationalChain {
            llm: llm_chain, // 设置 LLMChain
            memory, // 设置内存对象
            input_key: self
                .input_key
                // 设置输入键，如果未设置则使用默认输入键
                .unwrap_or_else(|| DEFAULT_INPUT_VARIABLE.to_string()),
        })
    }
}

