use std::collections::VecDeque;

use serde::Serialize;

#[derive(Debug, Default, Serialize)]
pub enum ChatMessageKind {
    #[default]
    System,
    Player,
}

#[derive(Debug, Default, Serialize)]
pub struct ChatMessage {
    pub kind: ChatMessageKind,
    pub msg: String,
}

#[derive(Debug, Default, Serialize)]
#[serde(transparent)]
pub struct ChatHistory {
    pub inner: VecDeque<ChatMessage>
}

impl ChatHistory {
    pub fn update(&mut self, data: &insim::Packet) -> bool {
        match data {
            insim::Packet::Mso(mso) => self.mso(mso),
            _ => { false },
        }
    }

    fn mso(&mut self, mso: &insim::insim::Mso) -> bool {
        if self.inner.len() >= 20 {
            self.inner.pop_back();
        }

        let kind = if *mso.ucid == 0 {
            ChatMessageKind::System
        } else {
            ChatMessageKind::Player
        };

        let msg = insim::core::string::colours::strip(&mso.msg).into_owned();

        let _ = self.inner.push_front(ChatMessage { kind, msg });

        true
    }
}
