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

use crate::world::WorldState;

// basic handler that responds with a static string
pub(crate) async fn root(State(state): State<WorldState>) -> String {
    serde_json::to_string(&(*state)).unwrap()
}

pub(crate) async fn live(
    State(state): State<WorldState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = stream::repeat_with(move || Event::default().json_data(&(*state)).unwrap())
        .map(Ok)
        .throttle(Duration::from_secs(1));

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}
