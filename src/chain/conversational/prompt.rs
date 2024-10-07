// 定义一个公共常量 `DEFAULT_TEMPLATE`，它是一个字符串切片，表示默认的对话模板。
// 该模板用于生成人类与AI之间的友好对话。
// AI在对话中表现得健谈，并提供大量来自其上下文的具体细节。
// 如果AI不知道问题的答案，它会如实说不知道。
// 模板中的 `{history}` 将被替换为当前对话的历史记录。
// `{input}` 将被替换为人类的输入。
pub const DEFAULT_TEMPLATE: &str = r#"The following is a friendly conversation between a human and an AI. The AI is talkative and provides lots of specific details from its context. If the AI does not know the answer to a question, it truthfully says it does not know.

Current conversation:
{history}
Human: {input}
AI:
"#;
