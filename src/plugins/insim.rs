use std::time::Duration;

use bevy_ecs::prelude::{Event, EventReader, EventWriter};
use bevy_ecs::system::{ResMut, Resource};
use tokio::time::sleep;

#[derive(Resource, Clone, Debug)]
pub struct InsimPluginRx(flume::Receiver<InsimEvent>);

#[derive(Event, Debug)]
pub(crate) enum InsimEvent {
    Connected,
    Disconnected,
    Packet(insim::Packet),
}

pub(crate) struct Plugin {
    pub(crate) config: crate::config::connection::Connection,
}

impl crate::ecs::Plugin for Plugin {
    fn name(&self) -> &'static str {
        "Insim"
    }

    fn register(&self, ecs: &mut crate::ecs::Ecs) {
        let (tx, rx) = flume::unbounded();

        let builder = self.config.as_insim_builder();

        tokio::spawn(async move {
            let mut attempt: u64 = 0;

            loop {
                if attempt > 0 {
                    let secs = Duration::from_millis(1000 * attempt);
                    sleep(secs).await;
                }

                attempt = attempt.wrapping_add(1);

                let mut conn = match builder.connect().await {
                    Ok(i) => i,
                    Err(_) => continue,
                };

                attempt = 0;

                tx.send(InsimEvent::Connected).unwrap();

                while let Ok(e) = conn.read().await {
                    tx.send(InsimEvent::Packet(e)).unwrap();
                }

                tx.send(InsimEvent::Disconnected).unwrap();
            }
        });

        ecs.add_resource(InsimPluginRx(rx));
        ecs.add_system(crate::ecs::PreTick, process_insim_event);
        ecs.add_event::<InsimEvent>();
    }
}

fn process_insim_event(mut events: EventWriter<InsimEvent>, rx: ResMut<InsimPluginRx>) {
    loop {
        match rx.0.try_recv() {
            Ok(i) => {
                events.send(i);
            }
            Err(flume::TryRecvError::Disconnected) => {
                panic!("FUCK")
            }
            Err(flume::TryRecvError::Empty) => {
                break;
            }
        }
    }
}

pub(crate) fn print_insim_events(mut events: EventReader<InsimEvent>) {
    for e in events.read() {
        println!("{:?}", e);
    }
}
