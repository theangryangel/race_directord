use std::{collections::BTreeMap, hash::Hash, time::Duration};

use insim::{core::point::Point, insim::PenaltyInfo};
use serde::{Serialize, Serializer};

#[derive(Debug, Clone, Copy, Serialize)]
pub enum GapKind {
    Laps(usize),
    Duration(Duration),
}

pub fn serialize_with<S, K1, K2, V>(
    map: &BTreeMap<(K1, K2), V>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    K1: Serialize + Ord + Clone,
    K2: Serialize + Ord + Clone,
    V: Serialize + Clone,
{
    let vec: Vec<((K1, K2), V)> = map
        .iter()
        .map(|((k1, k2), v)| ((k1.clone(), k2.clone()), (*v).clone()))
        .collect();
    vec.serialize(serializer)
}

#[derive(Debug, Default, Serialize, Clone)]
pub struct Player {
    /// Player name
    pub name: String,

    /// Number plate
    pub plate: String,

    /// Lap history
    pub laps: BTreeMap<u16, Lap>,

    /// Current Lap
    pub current_lap: u16,

    /// Current position
    pub position: u8,

    /// Gap between players
    pub gap: Option<GapKind>,

    // Split by Lap and Split number
    #[serde(serialize_with = "serialize_with")]
    pub splits: BTreeMap<(u16, u8), Split>,

    /// Pit stop by lap
    pub pit_stops: BTreeMap<u16, Duration>,

    /// Position on track, if None then in garage
    pub xyz: Option<Point<i32>>,

    /// Is the place on track in the pit lane?
    pub in_pit_lane: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct Split {
    pub stime: Duration,
    pub etime: Duration,

    pub penalty: PenaltyInfo,
}

#[derive(Debug, Default, Serialize, Clone)]
pub struct Lap {
    pub ltime: Duration,
    pub etime: Duration,
    pub penalty: PenaltyInfo,
}
