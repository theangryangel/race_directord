use std::{convert::Infallible, time::Duration};

use futures::stream::{self, Stream};
use itertools::Itertools;

use tokio_stream::StreamExt as _;

use axum::{
    extract::State,
    http::StatusCode,
    response::{sse::Event, Sse},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::world::{ChatState, WorldChangeKind, WorldState};

#[derive(Debug, Clone)]
pub(crate) struct WebState {
    pub world: WorldState,
    pub chat: ChatState,
    pub changes: flume::Receiver<WorldChangeKind>,
}

pub(crate) async fn live(
    State(state): State<WebState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {

    let stream = async_stream::stream! {
        while let Ok(kind) = state.changes.recv_async().await { 
            yield match kind {
                WorldChangeKind::Chat => Ok(Event::default().event("game").json_data(&(*state.world.lock().unwrap())).unwrap()),
                WorldChangeKind::State => Ok(Event::default().event("chat").json_data(&(*state.chat.lock().unwrap())).unwrap()),
            }
        }
    };

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}
