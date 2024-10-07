use std::collections::VecDeque;

use regex::Regex;
use serde::Deserialize;
use serde_json::Value;

use crate::{
    agent::AgentError,
    schemas::agent::{AgentAction, AgentEvent, AgentFinish},
};

use super::prompt::FORMAT_INSTRUCTIONS;

// 定义一个结构体，用于反序列化从JSON中提取的代理输出
#[derive(Debug, Deserialize)]
struct AgentOutput {
    action: String,
    action_input: String,
}

// 定义ChatOutputParser结构体，用于解析聊天输出
pub struct ChatOutputParser {}

impl ChatOutputParser {
    // 构造函数，创建一个新的ChatOutputParser实例
    pub fn new() -> Self {
        Self {}
    }
}

impl ChatOutputParser {
    // 解析输入文本并返回AgentEvent结果
    pub fn parse(&self, text: &str) -> Result<AgentEvent, AgentError> {
        log::debug!("Parsing to Agent Action: {}", text);
        match parse_json_markdown(text) {
            Some(value) => {
                // 将Value反序列化为AgentOutput结构体
                let agent_output: AgentOutput = serde_json::from_value(value)?;

                // 根据action字段的值决定返回AgentEvent::Finish或AgentEvent::Action
                if agent_output.action == "Final Answer" {
                    Ok(AgentEvent::Finish(AgentFinish {
                        output: agent_output.action_input,
                    }))
                } else {
                    Ok(AgentEvent::Action(vec![AgentAction {
                        tool: agent_output.action,
                        tool_input: agent_output.action_input,
                        log: text.to_string(),
                    }]))
                }
            }
            None => {
                log::debug!("No JSON found or malformed JSON in text: {}", text);
                Ok(AgentEvent::Finish(AgentFinish {
                    output: text.to_string(),
                }))
            }
        }
    }

    // 返回格式化指令字符串
    pub fn get_format_instructions(&self) -> &str {
        FORMAT_INSTRUCTIONS
    }
}

// 解析部分JSON字符串，修复不完整的JSON结构
fn parse_partial_json(s: &str, strict: bool) -> Option<Value> {
    // 首先尝试直接解析字符串
    match serde_json::from_str::<Value>(s) {
        Ok(val) => return Some(val),
        Err(_) if !strict => (),
        Err(_) => return None,
    }

    let mut new_s = String::new();
    let mut stack: VecDeque<char> = VecDeque::new();
    let mut is_inside_string = false;
    let mut escaped = false;

    // 遍历字符串中的每个字符，修复不完整的JSON结构
    for char in s.chars() {
        match char {
            '"' if !escaped => is_inside_string = !is_inside_string,
            '{' if !is_inside_string => stack.push_back('}'),
            '[' if !is_inside_string => stack.push_back(']'),
            '}' | ']' if !is_inside_string => {
                if let Some(c) = stack.pop_back() {
                    if c != char {
                        return None; // 不匹配的闭合字符
                    }
                } else {
                    return None; // 不平衡的闭合字符
                }
            }
            '\\' if is_inside_string => escaped = !escaped,
            _ => escaped = false,
        }
        new_s.push(char);
    }

    // 关闭任何未闭合的结构
    while let Some(c) = stack.pop_back() {
        new_s.push(c);
    }

    // 再次尝试解析修复后的字符串
    serde_json::from_str(&new_s).ok()
}

// 解析包含JSON的Markdown文本
fn parse_json_markdown(json_markdown: &str) -> Option<Value> {
    // 使用正则表达式匹配Markdown中的JSON代码块
    if let Some(caps) = re.captures(json_markdown) {
        if let Some(json_str) = caps.get(1) {
            return parse_partial_json(json_str.as_str(), false);
        }
    }
    None
}
