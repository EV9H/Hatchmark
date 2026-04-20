use anyhow::{Context, Result};
use hatchmark_core::ipc::{DaemonMsg, UiMsg, IPC_HOST, IPC_PROTO_VERSION};
use hatchmark_core::paths::AppPaths;
use serde::Serialize;
use std::sync::mpsc::{Receiver, Sender as StdSender};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;

#[derive(Serialize)]
struct DaemonState<'a> {
    port: u16,
    version: &'a str,
}

pub fn spawn(
    paths: AppPaths,
    rx: Receiver<DaemonMsg>,
    reload_notify: StdSender<()>,
) -> Result<()> {
    std::thread::Builder::new()
        .name("ipc-server".into())
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("tokio rt");
            if let Err(e) = rt.block_on(run(paths, rx, reload_notify)) {
                eprintln!("ipc server exited: {e}");
            }
        })?;
    Ok(())
}

async fn run(
    paths: AppPaths,
    rx: Receiver<DaemonMsg>,
    reload_notify: StdSender<()>,
) -> Result<()> {
    let listener = TcpListener::bind((IPC_HOST, 0))
        .await
        .context("bind ipc listener")?;
    let port = listener.local_addr()?.port();
    std::fs::write(
        &paths.state_path,
        serde_json::to_vec(&DaemonState {
            port,
            version: IPC_PROTO_VERSION,
        })?,
    )?;
    tracing::info!("ipc listening on 127.0.0.1:{port}");

    let (broadcast_tx, _) = broadcast::channel::<DaemonMsg>(64);

    // Bridge std::mpsc -> tokio::broadcast.
    let bcast = broadcast_tx.clone();
    std::thread::Builder::new()
        .name("ipc-bridge".into())
        .spawn(move || {
            while let Ok(msg) = rx.recv() {
                let _ = bcast.send(msg);
            }
        })?;

    loop {
        let (stream, _) = listener.accept().await?;
        let sub = broadcast_tx.subscribe();
        let notify = reload_notify.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_client(stream, sub, notify).await {
                tracing::debug!("client error: {e}");
            }
        });
    }
}

async fn handle_client(
    stream: TcpStream,
    mut sub: broadcast::Receiver<DaemonMsg>,
    reload_notify: StdSender<()>,
) -> Result<()> {
    let (read_half, mut write_half) = stream.into_split();
    let mut reader = BufReader::new(read_half).lines();

    let hello = DaemonMsg::Hello {
        version: IPC_PROTO_VERSION.to_string(),
        current_layer_id: 0,
    };
    let mut line = serde_json::to_string(&hello)?;
    line.push('\n');
    write_half.write_all(line.as_bytes()).await?;

    loop {
        tokio::select! {
            line = reader.next_line() => {
                match line? {
                    Some(text) => {
                        match serde_json::from_str::<UiMsg>(&text) {
                            Ok(UiMsg::Ping) => {}
                            Ok(UiMsg::Subscribe) => {}
                            Ok(UiMsg::ReloadBindings) => { let _ = reload_notify.send(()); }
                            Ok(UiMsg::SwitchLayer { .. }) => { let _ = reload_notify.send(()); }
                            Err(e) => tracing::debug!("bad ui msg: {e}: {text}"),
                        }
                    }
                    None => return Ok(()),
                }
            }
            Ok(msg) = sub.recv() => {
                let mut line = serde_json::to_string(&msg)?;
                line.push('\n');
                write_half.write_all(line.as_bytes()).await?;
            }
        }
    }
}
