use serde::Serialize;

#[derive(Debug, Default, Serialize)]
pub enum ChatMessageKind {
    #[default]
    System,
    Player(String),
}

#[derive(Debug, Default, Serialize)]
pub struct ChatMessage {
    pub kind: ChatMessageKind,
    pub msg: String,
}
