use std::{error::Error, time::Duration};

use serde::Deserialize;
use serde_json::{Value, json};
use ureq::Agent;

use crate::{
    agent::{Action, Direction},
    observation::Observation,
};

/// The model's reply. Field order here doesn't matter (serde matches by name);
/// generation order is set by the schema below.
#[derive(Deserialize, Debug)]
pub struct Reply {
    pub thought: String,
    pub action: Choice,
}

/// The action vocabulary the model sees. Deliberately flat and lowercase.
#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Choice {
    North,
    South,
    East,
    West,
    Eat,
    Wait,
}

impl From<Choice> for Action {
    fn from(choice: Choice) -> Self {
        match choice {
            Choice::North => Action::Move(Direction::North),
            Choice::South => Action::Move(Direction::South),
            Choice::East => Action::Move(Direction::East),
            Choice::West => Action::Move(Direction::West),
            Choice::Eat => Action::Eat,
            Choice::Wait => Action::Wait,
        }
    }
}

/// Key order is generation order (verified against Ollama): thought, then action.
/// Relies on serde_json's `preserve_order` feature.
fn schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "thought": { "type": "string" },
            "action": {
                "type": "string",
                "enum": ["north", "south", "east", "west", "eat", "wait"]
            }
        },
        "required": ["thought", "action"]
    })
}

const SYSTEM_PROMPT: &str = "\
You are an agent living in a 2D grid world. Each turn you choose exactly one action.

Rules:
- north/south/east/west moves you one tile. Walls (#) and other agents block you; \
moving into them does nothing.
- eat, while standing on a berry, gives +25 energy. Energy cannot exceed 150; \
anything above that is lost.
- eat anywhere else does nothing.
- You lose 1 energy every turn, whatever you do. At 0 you die.

Think briefly, then choose.";

const CHAT_URL: &str = "http://localhost:11434/api/chat";

/// One Ollama connection, owned by one worker thread.
pub struct Client {
    http: Agent,
    model: String,
}

#[derive(Deserialize)]
struct Message {
    content: String,
}

/// Ollama's envelope. Only the fields we use; serde ignores the rest.
#[derive(Deserialize)]
struct ChatResponse {
    message: Message,
}

impl Client {
    pub fn new(model: &str) -> Self {
        // Generous timeout: a cold call loads the model first (measured ~3s, can be more).
        let http: Agent = Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(60)))
            .build()
            .into();
        Client {
            http,
            model: model.to_string(),
        }
    }

    pub fn decide(&self, obs: &Observation) -> Result<Reply, Box<dyn Error>> {
        let body = json!({
            "model": self.model,
            "stream": false,
            "format": schema(),
            "options": { "temperature": 0, "seed": 1 },
            "messages": [
                { "role": "system", "content": SYSTEM_PROMPT },
                { "role": "user", "content": obs.to_string() },
            ],
        });

        let chat: ChatResponse = self
            .http
            .post(CHAT_URL)
            .send_json(&body)?
            .body_mut()
            .read_json()?;
        let reply: Reply = serde_json::from_str(&chat.message.content)?;
        Ok(reply)
    }
}
