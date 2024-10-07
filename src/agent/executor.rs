use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use serde_json::json;
use tokio::sync::Mutex;

use crate::{
    chain::{chain_trait::Chain, ChainError},
    language_models::GenerateResult,
    memory::SimpleMemory,
    prompt::PromptArgs,
    schemas::{
        agent::{AgentAction, AgentEvent},
        memory::BaseMemory,
    },
    tools::Tool,
};

use super::{agent::Agent, AgentError};

// 定义AgentExecutor结构体，泛型参数A必须实现Agent trait
pub struct AgentExecutor<A>
where
    A: Agent,
{
    agent: A, // 代理实例
    max_iterations: Option<i32>, // 最大迭代次数，默认为10
    break_if_error: bool, // 是否在工具调用出错时中断
    pub memory: Option<Arc<Mutex<dyn BaseMemory>>>, // 可选的内存实例
}

// 为AgentExecutor实现方法
impl<A> AgentExecutor<A>
where
    A: Agent,
{
    // 从代理实例创建AgentExecutor
    pub fn from_agent(agent: A) -> Self {
        Self {
            agent,
            max_iterations: Some(10),
            break_if_error: false,
            memory: None,
        }
    }

    // 设置最大迭代次数
    pub fn with_max_iterations(mut self, max_iterations: i32) -> Self {
        self.max_iterations = Some(max_iterations);
        self
    }

    // 设置内存实例
    pub fn with_memory(mut self, memory: Arc<Mutex<dyn BaseMemory>>) -> Self {
        self.memory = Some(memory);
        self
    }

    // 设置是否在工具调用出错时中断
    pub fn with_break_if_error(mut self, break_if_error: bool) -> Self {
        self.break_if_error = break_if_error;
        self
    }

    // 获取工具名称到工具实例的映射
    fn get_name_to_tools(&self) -> HashMap<String, Arc<dyn Tool>> {
        let mut name_to_tool = HashMap::new();
        for tool in self.agent.get_tools().iter() {
            log::debug!("Loading Tool:{}", tool.name());
            name_to_tool.insert(tool.name().trim().replace(" ", "_"), tool.clone());
        }
        name_to_tool
    }
}

// 为AgentExecutor实现Chain trait
#[async_trait]
impl<A> Chain for AgentExecutor<A>
where
    A: Agent + Send + Sync,
{
    // 调用代理执行器，返回生成结果
    async fn call(&self, input_variables: PromptArgs) -> Result<GenerateResult, ChainError> {
        let mut input_variables = input_variables.clone();
        let name_to_tools = self.get_name_to_tools(); // 获取工具名称到工具实例的映射
        let mut steps: Vec<(AgentAction, String)> = Vec::new(); // 初始化步骤列表
        log::debug!("steps: {:?}", steps); // 记录当前步骤

        // 如果存在内存实例，则获取聊天历史记录
        if let Some(memory) = &self.memory {
            let memory = memory.lock().await; // 获取内存锁
            input_variables.insert("chat_history".to_string(), json!(memory.messages())); // 插入聊天历史记录
        } else {
            input_variables.insert(
                "chat_history".to_string(),
                json!(SimpleMemory::new().messages()), // 使用默认的简单内存实例
            );
        }

        loop {
            // 代理规划下一步动作
            let agent_event = self
                .agent
                .plan(&steps, input_variables.clone())
                .await
                .map_err(|e| ChainError::AgentError(format!("Error in agent planning: {}", e)))?; // 规划代理动作

            match agent_event {
                // 处理代理动作
                AgentEvent::Action(actions) => {
                    for action in actions {
                        log::debug!("Action: {:?}", action.tool_input); // 记录当前动作
                        let tool = name_to_tools
                            .get(&action.tool)
                            .ok_or_else(|| {
                                AgentError::ToolError(format!("Tool {} not found", action.tool)) // 工具未找到错误
                            })
                            .map_err(|e| ChainError::AgentError(e.to_string()))?; // 转换错误类型

                        let observation_result = tool.call(&action.tool_input).await; // 调用工具

                        let observation = match observation_result {
                            Ok(result) => result, // 工具调用成功
                            Err(err) => {
                                log::info!(
                                    "The tool return the following error: {}",
                                    err.to_string()
                                ); // 记录工具错误
                                if self.break_if_error {
                                    return Err(ChainError::AgentError(
                                        AgentError::ToolError(err.to_string()).to_string(), // 工具错误中断
                                    ));
                                } else {
                                    format!("The tool return the following error: {}", err) // 记录工具错误
                                }
                            }
                        };

                        steps.push((action, observation)); // 记录步骤
                    }
                }
                AgentEvent::Finish(finish) => {
                    if let Some(memory) = &self.memory {
                        let mut memory = memory.lock().await; // 获取内存锁
                        memory.add_user_message(&input_variables["input"]); // 添加用户消息
                        memory.add_ai_message(&finish.output); // 添加AI消息
                    }
                    return Ok(GenerateResult {
                        generation: finish.output, // 返回生成结果
                        ..Default::default()
                    });
                }
            }

            if let Some(max_iterations) = self.max_iterations {
                if steps.len() >= max_iterations as usize {
                    return Ok(GenerateResult {
                        generation: "Max iterations reached".to_string(), // 达到最大迭代次数
                        ..Default::default()
                    });
                }
            }
        }
    }

    async fn invoke(&self, input_variables: PromptArgs) -> Result<String, ChainError> {
        let result = self.call(input_variables).await?; // 调用call方法
        Ok(result.generation) // 返回生成结果
    }
}