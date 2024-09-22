use std::time::Duration;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct RaceResult {
    pub uname: String,
    pub pname: String,

    pub ttime: Duration,
    pub btime: Duration,

    pub laps: u16,

    pub pit_stops: u16,
}
