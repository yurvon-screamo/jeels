use openai::{
    Credentials,
    chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole},
};

use crate::AppConfig;
use anyhow::Result;

pub async fn generate_video_content(topic: &str, config: &AppConfig) -> Result<String> {
    let credentials = Credentials::new(
        config.openai_api_key.clone(),
        config.openai_api_base.clone(),
    );
    let messages = vec![
        ChatCompletionMessage {
            role: ChatCompletionMessageRole::System,
            content: Some(
                "You are a helpful assistant.
Your task is to generate text for the audio of the video that will be used to teach Russian-speaking students to the N5 level of Japanese.
The video will be from 15 to 40 seconds. 

THE CONTENT MUST BE IN RUSSIAN.

You need to create a live and interesting content that should interest the student.

Only text, no headers, notes, transliteration, or greetings like 'welcome to the lesson'.

DON'T WRITE THE TRANSLITERATION WORDS, IT WILL SOUND BAD IN AUDIO. For example not write: 'Глагол 助ける (тасукэру)', insted write: 'Глагол 助ける'.

Don't use common phrases that clutter the content, such as good luck in learning, welcome to the lesson, create your own example and etc. Such phrases simply take time from the viewer.

Don't suggest the user to do something themselves, don't give any tasks.

We assume that the viewer already knows the basic words and grammar, so use simple words and grammar in examples.
"

                    .to_string(),
            ),
            name: None,
            function_call: None,
            tool_call_id: None,
            tool_calls: None,
        },
        ChatCompletionMessage {
            role: ChatCompletionMessageRole::User,
            content: Some(topic.to_string()),
            name: None,
            function_call: None,
            tool_call_id: None,
            tool_calls: None,
        },
    ];

    let chat_completion = ChatCompletion::builder(config.openai_model.as_str(), messages.clone())
        .credentials(credentials.clone())
        .create()
        .await?;

    let returned_message = chat_completion
        .choices
        .first()
        .ok_or(anyhow::anyhow!("No message returned"))?
        .message
        .content
        .clone()
        .ok_or(anyhow::anyhow!("No content returned"))?;

    let returned_message = returned_message
        .split_once("</think>")
        .map(|(_, content)| content)
        .unwrap_or(returned_message.as_str())
        .trim()
        .replace("\\n", "\n")
        .to_string();

    if returned_message.is_empty() {
        return Err(anyhow::anyhow!("No content returned"));
    }

    Ok(returned_message)
}
