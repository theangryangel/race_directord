use std::{
    sync::{Arc, Mutex}, thread::sleep, time::{Duration, Instant}
};
use itertools::Itertools as _;

use chat::ChatHistory;

pub mod chat;
pub mod connection;
pub mod player;
pub mod race_result;
pub mod state;

pub type WorldState = Arc<Mutex<state::State>>;
pub type ChatState = Arc<Mutex<ChatHistory>>;

pub enum WorldChangeKind {
    Chat,
    State,
}

pub struct World {
    pub target_frame_time: Duration,
    pub last_tick: Instant,
    pub rx: flume::Receiver<insim::Packet>,
    pub state: WorldState,
    pub chat: ChatState,
    pub exit: bool,

    notifier: flume::Sender<WorldChangeKind>,
}

impl World {
    pub(crate) fn new(
        rx: flume::Receiver<insim::Packet>,
        tx: flume::Sender<WorldChangeKind>,
    ) -> Self {
        let state = WorldState::default();
        let chat = ChatState::default();

        Self {
            target_frame_time: Duration::from_secs_f32(1.0 / 60.0),
            last_tick: Instant::now(),
            rx,
            state,
            chat,
            exit: false,
            notifier: tx
        }
    }

    pub(crate) fn run(&mut self) -> anyhow::Result<()> {
        let mut i: usize = 0;

        loop {
            let start = Instant::now();

            self.tick(&mut i, start - self.last_tick)?;

            // // Calculate how long the logic took to run
            // let logic_duration = start.elapsed();
            //
            // println!("Elapsed {:?}", logic_duration);

            self.last_tick = Instant::now();

            if self.exit {
                break;
            }

            sleep(self.target_frame_time);
        }

        Ok(())
    }

    fn tick(&mut self, i: &mut usize, _delta: Duration) -> anyhow::Result<()> {
        loop {
            match self.rx.try_recv() {
                Ok(p) => {
                    if self.state.lock().unwrap().update(&p) {
                        let _ = self.notifier.send(WorldChangeKind::Chat);
                    }
                    if self.chat.lock().unwrap().update(&p) {
                        let _ = self.notifier.send(WorldChangeKind::State);
                    }
                }
                Err(flume::TryRecvError::Empty) => break,
                Err(e) => return Err(e.into()),
            }
        }

        // if recompute_gaps {
        //     println!("recomputing gaps");
        //     self.state.recompute_gaps(&*i);
        // }

        // Your 60Hz logic here
        // println!("Running at 60Hz {:?} {:?}", i, delta);

        if *i % 60_usize == 0_usize {
            // println!("State: {:?}", self.state);

            for player in self
                .state
                .lock()
                .unwrap()
                .players
                .iter()
                .filter(|p| p.1.position > 0)
                .sorted_by(|a, b| Ord::cmp(&a.1.position, &b.1.position))
                .map(|(key, value)| (*key, value.name.clone(), value.gap))
            {
                println!(
                    "{:?} {:?} gap={:?}",
                    player.0,
                    insim::core::string::colours::strip(&player.1),
                    player.2
                );
            }

            println!("\n");
        }

        *i = i.wrapping_add(1);

        Ok(())
    }
}
