use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    prompt::PromptArgs,
    schemas::agent::{AgentAction, AgentEvent},
    tools::Tool,
};

use super::AgentError;

// 定义一个异步 trait，表示一个可以并发执行的代理
#[async_trait]
pub trait Agent: Send + Sync {
    // 异步方法，用于规划代理的下一步行动
    // 参数:
    // - intermediate_steps: 代理已经执行的中间步骤，每个步骤包含一个 AgentAction 和一个字符串结果
    // - inputs: 输入参数，类型为 PromptArgs
    // 返回值:
    // - 返回一个 AgentEvent 表示规划的结果，或者返回一个 AgentError 表示错误
    async fn plan(
        &self,
        intermediate_steps: &[(AgentAction, String)],
        inputs: PromptArgs,
    ) -> Result<AgentEvent, AgentError>;

    // 获取代理可用的工具列表
    // 返回值:
    // - 返回一个包含 Arc<dyn Tool> 的 Vec，表示代理可以使用的工具
    fn get_tools(&self) -> Vec<Arc<dyn Tool>>;
}
