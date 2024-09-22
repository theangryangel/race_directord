use insim::{identifiers::RequestId, insim::IsiFlags, Packet, WithRequestId};
use tokio::time::sleep;
use tracing_subscriber::filter::EnvFilter;

mod collections;
mod web;
mod world;

use std::{
    net::{SocketAddr, ToSocketAddrs},
    path::PathBuf,
    thread,
    time::Duration,
};
use tower_http::{services::ServeDir, trace::TraceLayer};

async fn insim_runner(tx: flume::Sender<::insim::Packet>) -> anyhow::Result<()> {
    let mut attempts = 0_u64;

    'outer: loop {
        attempts = attempts.wrapping_add(1);

        if attempts >= 2 {
            break;
        }

        sleep(Duration::from_secs(attempts)).await;

        let mut c = match ::insim::tcp("172.24.64.1:29999".parse::<SocketAddr>().unwrap())
            // let mut c = match ::insim::relay()
            // .relay_select_host(Some("Nubbins AU Demo".into()))
            .isi_flags(IsiFlags::MCI)
            .isi_interval(Duration::from_secs(1))
            .connect_async()
            .await
        {
            Ok(c) => {
                attempts = 0;
                c
            }
            Err(e) => {
                tracing::warn!("{:?}", e);
                continue;
            }
        };

        c.write(insim::insim::TinyType::Sst.with_request_id(RequestId(1)))
            .await?;
        c.write(insim::insim::TinyType::Ncn.with_request_id(RequestId(1)))
            .await?;
        c.write(insim::insim::TinyType::Npl.with_request_id(RequestId(1)))
            .await?;
        c.write(insim::insim::TinyType::Rst.with_request_id(RequestId(1)))
            .await?;
        c.write(insim::insim::TinyType::Res.with_request_id(RequestId(1)))
            .await?;
        c.write(insim::insim::TinyType::Reo.with_request_id(RequestId(1)))
            .await?;

        loop {
            match c.read().await {
                Ok(packet) => {
                    tx.send(packet)?;
                }
                Err(e) => {
                    tracing::warn!("{:?}", e);
                    continue 'outer;
                }
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("echo=trace".parse()?))
        .init();

    let (tx, rx) = flume::unbounded();
    let mut w = world::World::new(rx);

    let state = w.state.clone();

    thread::spawn(move || {
        w.run().unwrap();
    });

    let runner = tokio::spawn(insim_runner(tx));

    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
    let static_files_service = ServeDir::new(assets_dir).append_index_html_on_directories(true);

    let app = axum::Router::new()
        .fallback_service(static_files_service)
        .route("/live", axum::routing::get(web::live))
        .with_state(state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;

    tokio::select! {
        ctrl_c = tokio::signal::ctrl_c() => {
            ctrl_c?;
        },
        res = axum::serve(listener, app) => {
            res?;
        },
        // res = w.run() => {
        //     res?;
        // },
        res = runner => {
            res??;
        }
    }

    Ok(())
}
