
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::json;

use crate::{
    agent::{Agent, AgentError},
    chain::Chain,
    fmt_message, fmt_placeholder, fmt_template, message_formatter,
    prompt::{HumanMessagePromptTemplate, MessageFormatterStruct, PromptArgs},
    schemas::{
        agent::{AgentAction, AgentEvent, AgentFinish, LogTools},
        messages::Message,
        FunctionCallResponse,
    },
    template_jinja2,
    tools::Tool,
};

// 定义 OpenAiToolAgent 结构体，包含一个 Chain 和一个 Tool 的向量
pub struct OpenAiToolAgent {
    pub(crate) chain: Box<dyn Chain>,
    pub(crate) tools: Vec<Arc<dyn Tool>>,
}

impl OpenAiToolAgent {
    // 创建提示信息的方法，返回一个 MessageFormatterStruct 或 AgentError
    pub fn create_prompt(prefix: &str) -> Result<MessageFormatterStruct, AgentError> {
        let prompt = message_formatter![
            fmt_message!(Message::new_system_message(prefix)),
            fmt_placeholder!("chat_history"),
            fmt_template!(HumanMessagePromptTemplate::new(template_jinja2!(
                "{{input}}",
                "input"
            ))),
            fmt_placeholder!("agent_scratchpad")
        ];

        Ok(prompt)
    }

    // 构造 scratchpad 的方法，将中间步骤转换为 Message 向量
    fn construct_scratchpad(
        &self,
        intermediate_steps: &[(AgentAction, String)],
    ) -> Result<Vec<Message>, AgentError> {
        let mut thoughts: Vec<Message> = Vec::new();

        for (action, observation) in intermediate_steps {
            // 直接反序列化并嵌入方法调用中以简化代码
            // 从日志中提取工具 ID 和工具调用
            let LogTools { tool_id, tools } = serde_json::from_str(&action.log)?;
            let tools: Vec<FunctionCallResponse> = serde_json::from_str(&tools)?;

            // 对于第一个动作，添加一个包含所有工具调用的 AI 消息
            if thoughts.is_empty() {
                thoughts.push(Message::new_ai_message("").with_tool_calls(json!(tools)));
            }

            // 为每个观察结果添加一个工具消息。观察结果是工具调用的输出。
            // tool_id 是工具的 ID。
            thoughts.push(Message::new_tool_message(observation, tool_id));
        }

        Ok(thoughts)
    }
}

// 实现 Agent trait 的异步方法
#[async_trait]
impl Agent for OpenAiToolAgent {
    // 规划方法，根据中间步骤和输入生成 AgentEvent
    async fn plan(
        &self,
        intermediate_steps: &[(AgentAction, String)],
        inputs: PromptArgs,
    ) -> Result<AgentEvent, AgentError> {
        let mut inputs = inputs.clone();
        let scratchpad = self.construct_scratchpad(intermediate_steps)?;
        inputs.insert("agent_scratchpad".to_string(), json!(scratchpad));
        let output = self.chain.call(inputs).await?.generation;
        match serde_json::from_str::<Vec<FunctionCallResponse>>(&output) {
            Ok(tools) => {
                let mut actions: Vec<AgentAction> = Vec::new();
                for tool in tools {
                    // 日志工具将作为日志发送
                    let log: LogTools = LogTools {
                        tool_id: tool.id.clone(),
                        tools: output.clone(), // 我们发送完整的工具输出，在 open ai 调用中需要它
                    };
                    actions.push(AgentAction {
                        tool: tool.function.name.clone(),
                        tool_input: tool.function.arguments.clone(),
                        log: serde_json::to_string(&log)?, // 我们将其作为字符串发送以最小化更改
                    });
                }
                return Ok(AgentEvent::Action(actions));
            }
            Err(_) => return Ok(AgentEvent::Finish(AgentFinish { output })),
        }
    }

    // 获取工具的方法，返回工具的向量
    fn get_tools(&self) -> Vec<Arc<dyn Tool>> {
        self.tools.clone()
    }
}
