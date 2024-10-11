use std::time::Duration;

use insim::core::track::Track;
use insim::identifiers::{ConnectionId, PlayerId};
use serde::Serialize;

use crate::collections::NoHashMap;
use crate::world::player::GapKind;

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
    pub players: NoHashMap<PlayerId, Player>,
    connections: NoHashMap<ConnectionId, Connection>,

    player_to_connection: NoHashMap<PlayerId, ConnectionId>,
    connection_to_player: NoHashMap<ConnectionId, PlayerId>,

    results: NoHashMap<u8, RaceResult>,

    game: Game,
}

impl State {
    pub fn update(&mut self, packet: &insim::Packet) -> bool {
        // FIXME: change the returns on the following to be more sensible
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
            insim::Packet::Reo(reo) => self.reo(reo),
            _ => { false }
        }
    }

    pub fn npl(&mut self, npl: &insim::insim::Npl) -> bool {
        self.players.insert(
            npl.plid,
            Player {
                name: npl.pname.clone(),
                plate: npl.plate.clone(),
                ..Default::default()
            },
        );

        self.player_to_connection.insert(npl.plid, npl.ucid);
        self.connection_to_player.insert(npl.ucid, npl.plid);

        true
    }

    pub fn ncn(&mut self, ncn: &insim::insim::Ncn) -> bool {
        self.connections.insert(
            ncn.ucid,
            Connection {
                uname: ncn.uname.clone(),
                pname: ncn.pname.clone(),
            },
        );

        true
    }

    pub fn cnl(&mut self, cnl: &insim::insim::Cnl) -> bool {
        self.connections.remove(&cnl.ucid);
        // ensure that we clean up the relative lookup tables
        // this shouldnt occur as LFS should have told us about the player leaving ahead of time
        if let Some(old_player) = self.connection_to_player.remove(&cnl.ucid) {
            self.players.remove(&old_player);
            self.player_to_connection.remove(&old_player);

            true
        } else {
            false
        }
    }

    pub fn pll(&mut self, pll: &insim::insim::Pll) -> bool {
        self.players.remove(&pll.plid);

        if let Some(ucid) = self.player_to_connection.remove(&pll.plid) {
            self.connection_to_player.remove(&ucid);
        }

        true
    }

    pub fn lap(&mut self, lap: &insim::insim::Lap) -> bool {
        if let Some(entry) = self.players.get_mut(&lap.plid) {
            entry.current_lap = lap.lapsdone + 1;

            entry.laps.insert(
                lap.lapsdone,
                Lap {
                    ltime: lap.ltime,
                    etime: lap.etime,
                    penalty: lap.penalty,
                },
            );

            let current_lap = entry.current_lap.clone();
            self.recompute_gap(&lap.plid, current_lap, 4_u8, lap.etime);

            return true;
        }

        false
    }

    pub fn mci(&mut self, mci: &insim::insim::Mci) -> bool {
        for info in mci.info.iter() {
            if let Some(entry) = self.players.get_mut(&info.plid) {
                entry.xyz = Some(info.xyz);
                entry.position = info.position;
            }
        }

        true
    }

    pub fn plp(&mut self, plp: &insim::insim::Plp) -> bool {
        if let Some(entry) = self.players.get_mut(&plp.plid) {
            entry.laps.clear();
            entry.splits.clear();
            entry.pit_stops.clear();
            entry.in_pit_lane = false;
            entry.current_lap = 0;
            entry.xyz = None;

            return true;
        }

        false
    }

    pub fn pla(&mut self, pla: &insim::insim::Pla) -> bool {
        if let Some(entry) = self.players.get_mut(&pla.plid) {
            if pla.entered_pitlane() {
                entry.in_pit_lane = true;
            }

            if pla.exited_pitlane() {
                entry.in_pit_lane = false;
            }

            return true;
        }

        false
    }

    pub fn spx(&mut self, spx: &insim::insim::Spx) -> bool {
        // TODO: if_chain!
        if let Some(ref mut entry) = self.players.get_mut(&spx.plid) {
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
                return true;
            }
            let current_lap = entry.current_lap.clone();
            self.recompute_gap(&spx.plid, current_lap, spx.split, spx.etime);

            return true;
        }

        return false;
    }

    pub fn recompute_gap(&mut self, current_player: &PlayerId, lap: u16, split: u8, etime: Duration) {
        let seek = {
            if let Some(ref mut entry) = self.players.get_mut(current_player) {
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
            if let Some(ahead_player) = self.players.iter().find_map(|p| {
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

        if let Some(ref mut entry) = self.players.get_mut(&current_player) {
            entry.gap = gap;
        }
    }

    pub fn psf(&mut self, psf: &insim::insim::Psf) -> bool {
        if let Some(entry) = self.players.get_mut(&psf.plid) {
            if entry.current_lap > 0 {
                let current = entry.current_lap;

                entry.pit_stops.insert(current, psf.stime);
            }

            return true;
        }

        false
    }

    pub fn toc(&mut self, toc: &insim::insim::Toc) -> bool {
        self.player_to_connection
            .insert(toc.plid, toc.newucid);
        self.connection_to_player.remove(&toc.olducid);
        self.connection_to_player
            .insert(toc.newucid, toc.plid);

        true
    }

    pub fn cpr(&mut self, cpr: &insim::insim::Cpr) -> bool {
        if let Some(connection) = self.connections.get_mut(&cpr.ucid) {
            connection.pname = cpr.pname.clone();
        }

        // TODO: if_chain!
        if let Some(plid) = self.connection_to_player.get(&cpr.ucid) {
            if let Some(player) = self.players.get_mut(&plid) {
                player.name = cpr.pname.clone();
                player.plate = cpr.plate.clone();
            }
        }

        true
    }

    pub fn res(&mut self, res: &insim::insim::Res) -> bool {
        self.results.insert(
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

        true
    }

    pub fn rst(&mut self, rst: &insim::insim::Rst) -> bool {
        if rst.flags.is_cruise() {
            self.game = Game::Cruise(rst.track.clone());
            return true;
        }

        if rst.qualmins > 0 {
            self.game =
                Game::Qualifying(rst.track.clone(), Duration::from_secs(rst.qualmins as u64));
            return true;
        }

        let stage = match rst.racelaps {
            insim::insim::RaceLaps::Laps(laps) => Game::RacingLaps(rst.track.clone(), laps),
            insim::insim::RaceLaps::Hours(hours) => {
                Game::RacingTime(rst.track.clone(), Duration::from_secs(hours as u64 * 3600))
            }
            _ => Game::Practise(rst.track.clone()),
        };

        self.game = stage;

        for (_key, ele) in self.players.iter_mut() {
            ele.splits.clear();
            ele.laps.clear();
            ele.current_lap = 1;
            ele.gap = None;
        }

        true
    }

    pub fn sta(&mut self, sta: &insim::insim::Sta) -> bool {
        if !sta.flags.is_in_game() {
            self.game = Game::None;
            true
        } else {
            false
        }
    }

    pub fn tiny(&mut self, tiny: &insim::insim::Tiny) -> bool {
        if tiny.subt == insim::insim::TinyType::Ren {
            self.game = Game::None;
            return true;
        }

        false
    }

    pub fn reo(&mut self, reo: &insim::insim::Reo) -> bool {
        for (idx, plid) in reo.plid.iter().enumerate() {
            if let Some(entry) = self.players.get_mut(plid) {
                entry.current_lap = 1;
                entry.position = (idx + 1) as u8;
            }
        }

        true
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
