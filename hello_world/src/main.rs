use core::fmt;
use std::io:: {self, Write};
use std::env;
// use reqwest::Error;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Clone)]
struct Message {
    role: String,
    content: String
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Role: {}, Content: {}", self.role, self.content)
    }
}

#[derive(Serialize)]
struct Body {
    model: String,
    messages: Messages,
    max_tokens: i32
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: MessageResponse
}

#[derive(Debug, Deserialize)]
struct MessageResponse {
    content: String
}

#[derive(Debug, Deserialize)]
struct Response {
    choices: Vec<Choice>
}

#[derive(Serialize, Clone)]
struct Messages(Vec<Message>);

impl fmt::Display for Messages {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        for message in &self.0 {
            write!(f, "{}\n", message)?;
        }
        Ok(())
    }
}

impl Messages {
    fn push(&mut self, message:Message){
        self.0.push(message);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let mut history = Messages(vec![]);
    loop{
        let mut prompt = String::new();

        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut prompt).unwrap();

        let input = prompt.trim();

        match input {
            "quit" => break,
            _ => {
                let user_message = Message {
                    role: "user".to_string(),
                    content: prompt
                };
                history.push(user_message);
                let response = receive_response(history.clone()).await?;
                let response_message = Message {
                    role: "assistant".to_string(),
                    content: response
                };
                history.push(response_message);
            }
        }
    }
    Ok(())
}

async fn receive_response(history: Messages) -> Result<String, Box<dyn std::error::Error>> {
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let url = "https://api.openai.com/v1/chat/completions";

    let body = Body {
        model:"gpt-4".to_string(),
        messages: history.clone(),
        max_tokens:200,
    };

    let client = reqwest::Client::new();

    let res = client.post(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    let text = res.text().await?;

    let response: Response = serde_json::from_str(&text)?;

    println!("Agent: {}", response.choices[0].message.content);

    Ok(response.choices[0].message.content.clone())
}

