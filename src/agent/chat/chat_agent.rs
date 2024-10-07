
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::json;

use crate::{
    agent::{agent::Agent, chat::prompt::FORMAT_INSTRUCTIONS, AgentError},
    chain::chain_trait::Chain,
    message_formatter,
    prompt::{
        HumanMessagePromptTemplate, MessageFormatterStruct, MessageOrTemplate, PromptArgs,
        PromptFromatter,
    },
    prompt_args,
    schemas::{
        agent::{AgentAction, AgentEvent},
        messages::Message,
    },
    template_jinja2,
    tools::Tool,
};

use super::{output_parser::ChatOutputParser, prompt::TEMPLATE_TOOL_RESPONSE};

// 定义对话代理结构体
pub struct ConversationalAgent {
    pub(crate) chain: Box<dyn Chain>, // 代理使用的链
    pub(crate) tools: Vec<Arc<dyn Tool>>, // 代理可用的工具
    pub(crate) output_parser: ChatOutputParser, // 输出解析器
}

impl ConversationalAgent {
    // 创建提示信息的方法
    pub fn create_prompt(
        tools: &[Arc<dyn Tool>], // 工具列表
        suffix: &str, // 后缀模板
        prefix: &str, // 前缀模板
    ) -> Result<MessageFormatterStruct, AgentError> {
        // 生成工具字符串
        let tool_string = tools
            .iter()
            .map(|tool| format!("> {}: {}", tool.name(), tool.description()))
            .collect::<Vec<_>>()
            .join("\n");
        // 生成工具名称字符串
        let tool_names = tools
            .iter()
            .map(|tool| tool.name())
            .collect::<Vec<_>>()
            .join(", ");

        // 生成后缀提示
        let sufix_prompt = template_jinja2!(suffix, "tools", "format_instructions");

        // 生成输入变量
        let input_variables_fstring = prompt_args! {
            "tools" => tool_string,
            "format_instructions" => FORMAT_INSTRUCTIONS,
            "tool_names"=>tool_names
        };

        // 格式化后缀提示
        let sufix_prompt = sufix_prompt.format(input_variables_fstring)?;
        // 生成消息格式化器
        let formatter = message_formatter![
            MessageOrTemplate::Message(Message::new_system_message(prefix)),
            MessageOrTemplate::MessagesPlaceholder("chat_history".to_string()),
            MessageOrTemplate::Template(
                HumanMessagePromptTemplate::new(template_jinja2!(
                    &sufix_prompt.to_string(),
                    "input"
                ))
                .into()
            ),
            MessageOrTemplate::MessagesPlaceholder("agent_scratchpad".to_string()),
        ];
        Ok(formatter)
    }

    // 构建临时工作区的方法
    fn construct_scratchpad(
        &self,
        intermediate_steps: &[(AgentAction, String)], // 中间步骤
    ) -> Result<Vec<Message>, AgentError> {
        let mut thoughts: Vec<Message> = Vec::new();
        // 遍历中间步骤
        for (action, observation) in intermediate_steps.iter() {
            // 添加AI消息
            thoughts.push(Message::new_ai_message(&action.log));
            // 生成工具响应
            let tool_response = template_jinja2!(TEMPLATE_TOOL_RESPONSE, "observation")
                .format(prompt_args!("observation"=>observation))?;
            // 添加人类消息
            thoughts.push(Message::new_human_message(&tool_response));
        }
        Ok(thoughts)
    }
}

// 实现Agent trait
#[async_trait]
impl Agent for ConversationalAgent {
    // 计划方法
    async fn plan(
        &self,
        intermediate_steps: &[(AgentAction, String)], // 中间步骤
        inputs: PromptArgs, // 输入参数
    ) -> Result<AgentEvent, AgentError> {
        // 构建临时工作区
        let scratchpad = self.construct_scratchpad(intermediate_steps)?;
        let mut inputs = inputs.clone();
        // 插入临时工作区
        inputs.insert("agent_scratchpad".to_string(), json!(scratchpad));
        // 调用链
        let output = self.chain.call(inputs.clone()).await?.generation;
        // 解析输出
        let parsed_output = self.output_parser.parse(&output)?;
        Ok(parsed_output)
    }

    // 获取工具的方法
    fn get_tools(&self) -> Vec<Arc<dyn Tool>> {
        self.tools.clone()
    }
}

// 测试模块
#[cfg(test)]
mod tests {
    use std::{error::Error, sync::Arc};

    use async_trait::async_trait;
    use serde_json::Value;

    use crate::{
        agent::{chat::builder::ConversationalAgentBuilder, executor::AgentExecutor},
        chain::chain_trait::Chain,
        llm::openai::{OpenAI, OpenAIModel},
        memory::SimpleMemory,
        prompt_args,
        tools::Tool,
    };

    // 定义计算器工具
    struct Calc {}

    #[async_trait]
    impl Tool for Calc {
        fn name(&self) -> String {
            "Calculator".to_string()
        }
        fn description(&self) -> String {
            "Usefull to make calculations".to_string()
        }
        async fn run(&self, _input: Value) -> Result<String, Box<dyn Error>> {
            Ok("25".to_string())
        }
    }

    // 测试调用代理
    #[tokio::test]
    #[ignore]
    async fn test_invoke_agent() {
        // 创建OpenAI实例
        let llm = OpenAI::default().with_model(OpenAIModel::Gpt4.to_string());
        // 创建简单内存实例
        let memory = SimpleMemory::new();
        // 创建计算器工具实例
        let tool_calc = Calc {};
        // 构建对话代理
        let agent = ConversationalAgentBuilder::new()
            .tools(&[Arc::new(tool_calc)])
            .build(llm)
            .unwrap();
        // 定义输入变量
        let input_variables = prompt_args! {
            "input" => "hola,Me llamo luis, y tengo 10 anos, y estudio Computer scinence",
        };
        // 创建代理执行器
        let executor = AgentExecutor::from_agent(agent).with_memory(memory.into());
        // 调用代理执行器
        match executor.invoke(input_variables).await {
            Ok(result) => {
                println!("Result: {:?}", result);
            }
            Err(e) => panic!("Error invoking LLMChain: {:?}", e),
        }
        // 定义新的输入变量
        let input_variables = prompt_args! {
            "input" => "cuanta es la edad de luis +10 y que estudia",
        };
        // 调用代理执行器
        match executor.invoke(input_variables).await {
            Ok(result) => {
                println!("Result: {:?}", result);
            }
            Err(e) => panic!("Error invoking LLMChain: {:?}", e),
        }
    }
}
