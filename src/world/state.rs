use std::collections::{BTreeMap, VecDeque};
use std::time::Duration;

use if_chain::if_chain;
use insim::core::track::Track;
use insim::identifiers::{ConnectionId, PlayerId};
use itertools::Itertools;
use parking_lot::RwLock;
use serde::Serialize;

use crate::collections::NoHashMap;
use crate::world::player::GapKind;

use super::chat::{ChatMessage, ChatMessageKind};
use super::connection::Connection;
use super::player::{Lap, Player, Split};
use super::race_result::RaceResult;

#[derive(Debug, Default, Serialize)]
pub enum Game {
    #[default]
    None,
    Practise(Track),
    Qualifying(Track, Duration),
    RacingLaps(Track, usize),
    RacingTime(Track, Duration),
    Cruise(Track),
}

#[derive(Debug, Default, Serialize)]
pub struct State {
    pub players: RwLock<NoHashMap<PlayerId, Player>>,
    connections: RwLock<NoHashMap<ConnectionId, Connection>>,

    player_to_connection: RwLock<NoHashMap<PlayerId, ConnectionId>>,
    connection_to_player: RwLock<NoHashMap<ConnectionId, PlayerId>>,

    results: RwLock<NoHashMap<u8, RaceResult>>,

    game: RwLock<Game>,

    chat: RwLock<VecDeque<ChatMessage>>,
}

impl State {
    pub fn update(&self, packet: &insim::Packet) {
        match packet {
            insim::Packet::Ncn(ncn) => self.ncn(ncn),
            insim::Packet::Cnl(cnl) => self.cnl(cnl),
            insim::Packet::Npl(npl) => self.npl(npl),
            insim::Packet::Pll(pll) => self.pll(pll),
            insim::Packet::Lap(lap) => self.lap(lap),
            insim::Packet::Mci(mci) => self.mci(mci),
            insim::Packet::Plp(plp) => self.plp(plp),
            insim::Packet::Pla(pla) => self.pla(pla),
            insim::Packet::Spx(spx) => self.spx(spx),
            insim::Packet::Psf(psf) => self.psf(psf),
            insim::Packet::Toc(toc) => self.toc(toc),
            insim::Packet::Cpr(cpr) => self.cpr(cpr),
            insim::Packet::Res(res) => self.res(res),
            insim::Packet::Rst(rst) => self.rst(rst),
            insim::Packet::Sta(sta) => self.sta(sta),
            insim::Packet::Tiny(tiny) => self.tiny(tiny),
            insim::Packet::Mso(mso) => self.mso(mso),
            insim::Packet::Reo(reo) => self.reo(reo),
            _ => {}
        }
    }

    pub fn npl(&self, npl: &insim::insim::Npl) {
        self.players.write().insert(
            npl.plid,
            Player {
                name: npl.pname.clone(),
                plate: npl.plate.clone(),
                ..Default::default()
            },
        );

        self.player_to_connection.write().insert(npl.plid, npl.ucid);
        self.connection_to_player.write().insert(npl.ucid, npl.plid);
    }

    pub fn ncn(&self, ncn: &insim::insim::Ncn) {
        self.connections.write().insert(
            ncn.ucid,
            Connection {
                uname: ncn.uname.clone(),
                pname: ncn.pname.clone(),
            },
        );
    }

    pub fn cnl(&self, cnl: &insim::insim::Cnl) {
        self.connections.write().remove(&cnl.ucid);
        // ensure that we clean up the relative lookup tables
        // this shouldnt occur as LFS should have told us about the player leaving ahead of time
        if let Some(old_player) = self.connection_to_player.write().remove(&cnl.ucid) {
            self.players.write().remove(&old_player);
            self.player_to_connection.write().remove(&old_player);
        }
    }

    pub fn pll(&self, pll: &insim::insim::Pll) {
        self.players.write().remove(&pll.plid);

        if let Some(ucid) = self.player_to_connection.write().remove(&pll.plid) {
            self.connection_to_player.write().remove(&ucid);
        }
    }

    pub fn lap(&self, lap: &insim::insim::Lap) {
        if let Some(entry) = self.players.write().get_mut(&lap.plid) {
            entry.current_lap = lap.lapsdone + 1;

            entry.laps.insert(
                lap.lapsdone,
                Lap {
                    ltime: lap.ltime,
                    etime: lap.etime,
                    penalty: lap.penalty,
                },
            );

            self.recompute_gap(&lap.plid, entry.current_lap, 4, lap.etime);
        }
    }

    pub fn mci(&self, mci: &insim::insim::Mci) {
        for info in mci.info.iter() {
            if let Some(entry) = self.players.write().get_mut(&info.plid) {
                entry.xyz = Some(info.xyz);
                entry.position = info.position;
            }
        }
    }

    pub fn plp(&self, plp: &insim::insim::Plp) {
        if let Some(entry) = self.players.write().get_mut(&plp.plid) {
            entry.laps.clear();
            entry.splits.clear();
            entry.pit_stops.clear();
            entry.in_pit_lane = false;
            entry.current_lap = 0;
            entry.xyz = None;
        }
    }

    pub fn pla(&self, pla: &insim::insim::Pla) {
        if let Some(entry) = self.players.write().get_mut(&pla.plid) {
            if pla.entered_pitlane() {
                entry.in_pit_lane = true;
            }

            if pla.exited_pitlane() {
                entry.in_pit_lane = false;
            }
        }
    }

    pub fn spx(&self, spx: &insim::insim::Spx) {
        // TODO: if_chain!
        if let Some(ref mut entry) = &mut self.players.write().get_mut(&spx.plid) {
            if entry.current_lap > 0 {
                entry.splits.insert(
                    (entry.current_lap, spx.split),
                    Split {
                        stime: spx.stime,
                        etime: spx.etime,
                        penalty: spx.penalty,
                    },
                );
            }

            if entry.position <= 1 {
                entry.gap = None;
                return;
            }
            self.recompute_gap(&spx.plid, entry.current_lap, spx.split, spx.etime);
        }
    }

    pub fn recompute_gap(&self, current_player: &PlayerId, lap: u16, split: u8, etime: Duration) {
        let seek = {
            if let Some(ref mut entry) = &mut self.players.write().get_mut(current_player) {
                if entry.position <= 1 {
                    entry.gap = None;
                    return;
                }
                entry.position - 1
            } else {
                return;
            }
        };

        let gap = {
            if let Some(ahead_player) = self.players.read().iter().find_map(|p| {
                if p.1.position == seek {
                    Some(p.1)
                } else {
                    None
                }
            }) {
                if let Some(split) = ahead_player.splits.get(&(lap, split)) {
                    if split.etime <= etime {
                        Some(GapKind::Duration(etime - split.etime))
                    } else {
                        None
                    }
                } else if ahead_player.current_lap > lap {
                    Some(GapKind::Laps(
                        ahead_player.current_lap as usize - lap as usize,
                    ))
                } else {
                    None
                }
            } else {
                None
            }
        };

        if let Some(ref mut entry) = &mut self.players.write().get_mut(&current_player) {
            entry.gap = gap;
        }
    }

    pub fn psf(&self, psf: &insim::insim::Psf) {
        if let Some(entry) = self.players.write().get_mut(&psf.plid) {
            if entry.current_lap > 0 {
                let current = entry.current_lap;

                entry.pit_stops.insert(current, psf.stime);
            }
        }
    }

    pub fn toc(&self, toc: &insim::insim::Toc) {
        self.player_to_connection
            .write()
            .insert(toc.plid, toc.newucid);
        self.connection_to_player.write().remove(&toc.olducid);
        self.connection_to_player
            .write()
            .insert(toc.newucid, toc.plid);
    }

    pub fn cpr(&self, cpr: &insim::insim::Cpr) {
        if let Some(connection) = self.connections.write().get_mut(&cpr.ucid) {
            connection.pname = cpr.pname.clone();
        }

        // TODO: if_chain!
        if let Some(plid) = self.connection_to_player.read().get(&cpr.ucid) {
            if let Some(player) = self.players.write().get_mut(&plid) {
                player.name = cpr.pname.clone();
                player.plate = cpr.plate.clone();
            }
        }
    }

    pub fn res(&self, res: &insim::insim::Res) {
        self.results.write().insert(
            res.resultnum,
            RaceResult {
                uname: res.uname.clone(),
                pname: res.pname.clone(),
                ttime: res.ttime,
                btime: res.btime,
                laps: res.lapsdone,
                pit_stops: res.numstops as u16,
            },
        );
    }

    pub fn rst(&self, rst: &insim::insim::Rst) {
        if rst.flags.is_cruise() {
            *self.game.write() = Game::Cruise(rst.track.clone());
            return;
        }

        if rst.qualmins > 0 {
            *self.game.write() =
                Game::Qualifying(rst.track.clone(), Duration::from_secs(rst.qualmins as u64));
            return;
        }

        let stage = match rst.racelaps {
            insim::insim::RaceLaps::Laps(laps) => Game::RacingLaps(rst.track.clone(), laps),
            insim::insim::RaceLaps::Hours(hours) => {
                Game::RacingTime(rst.track.clone(), Duration::from_secs(hours as u64 * 3600))
            }
            _ => Game::Practise(rst.track.clone()),
        };

        *self.game.write() = stage;

        for (_key, ele) in self.players.write().iter_mut() {
            ele.splits.clear();
            ele.laps.clear();
            ele.current_lap = 1;
            ele.gap = None;
        }
    }

    pub fn sta(&self, sta: &insim::insim::Sta) {
        if !sta.flags.is_in_game() {
            *self.game.write() = Game::None;
        }
    }

    pub fn tiny(&self, tiny: &insim::insim::Tiny) {
        if tiny.subt == insim::insim::TinyType::Ren {
            *self.game.write() = Game::None;
        }
    }

    pub fn mso(&self, mso: &insim::insim::Mso) {
        if mso.msg.len() <= 0 {
            return;
        }

        if let Some(connection) = self.connections.read().get(&mso.ucid) {
            let mut chat = self.chat.write();
            if chat.len() >= 20 {
                chat.pop_back();
            }

            let kind = if *mso.ucid == 0 {
                ChatMessageKind::System
            } else {
                ChatMessageKind::Player(connection.uname.to_owned())
            };

            let msg = insim::core::string::colours::strip(&mso.msg).into_owned();

            let _ = chat.push_front(ChatMessage { kind, msg });
        }
    }

    pub fn reo(&self, reo: &insim::insim::Reo) {
        for (idx, plid) in reo.plid.iter().enumerate() {
            if let Some(entry) = self.players.write().get_mut(plid) {
                entry.current_lap = 1;
                entry.position = (idx + 1) as u8;
            }
        }
    }

    // pub fn recompute_gaps(&self, i: &usize) {
    //     // recompute gaps, order the players in reverse order
    //     let players: Vec<(PlayerId, Option<Duration>)> = self.players.read()
    //         .iter()
    //         .filter(|p| p.1.position > 0)
    //         .sorted_by(|a, b| Ord::cmp(&a.1.position, &b.1.position))
    //         .tuples()
    //         .map(|(a, b)| {
    //             if_chain::if_chain! {
    //                 if let Some((key, current)) = a.1.splits.last_key_value();
    //                 if let Some(ahead) = b.1.splits.get(key);
    //                 then {
    //                     println!(" ahead={:?} {} {:?} current={:?} {} {:?}", a.1.name, a.1.position, ahead.etime, b.1.name, b.1.position, current.etime);
    //
    //                     (*a.0, Some(ahead.etime - current.etime))
    //                 } else {
    //                     let (key, _) = a.1.splits.last_key_value().unwrap();
    //
    //                     if let Some(((lap, _), _)) = b.1.splits.last_key_value() {
    //                         return (*a.0, GapKind::Lap(key.0 - lap));
    //                     }
    //                     (*a.0, None)
    //                 }
    //             }
    //         })
    //         .collect();
    //
    //     for (player, gap) in players.iter() {
    //         self.players.write().get_mut(&player).unwrap().gap = *gap;
    //     }
    //
    //     if *i % 60_usize == 0_usize {
    //         for (player, _gap) in players.iter() {
    //             let players = self.players.read();
    //             let c = players.get(&player).unwrap();
    //             println!("player={:?}, gap={:?}", c.name, c.gap);
    //         }
    //     }
    // }
}
