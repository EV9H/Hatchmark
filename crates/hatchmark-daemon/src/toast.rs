use hatchmark_core::db::{channels, settings};
use hatchmark_core::db::Db;
use hatchmark_core::model::Channel;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

pub enum ToastEvent {
    Show { channel: Channel, new_total: i64 },
    Quit,
}

pub fn spawn() -> Sender<ToastEvent> {
    let (tx, rx) = channel::<ToastEvent>();
    std::thread::Builder::new()
        .name("toast".into())
        .spawn(move || run(rx))
        .expect("spawn toast thread");
    tx
}

fn run(rx: Receiver<ToastEvent>) {
    // v1: log-only toast. Replacing the body of this loop with a Win32
    // layered window + GDI draw lights up real on-screen toasts; the
    // channel/queue shape already supports it.
    while let Ok(ev) = rx.recv() {
        match ev {
            ToastEvent::Show { channel, new_total } => {
                tracing::info!("toast: {} = {}", channel.name, new_total);
                std::thread::sleep(Duration::from_millis(1200));
            }
            ToastEvent::Quit => break,
        }
    }
}

pub fn maybe_send(db: &Db, channel_id: i64, new_total: i64, tx: &Sender<ToastEvent>) {
    if settings::get_bool(&db.conn, "toast_enabled").ok().flatten() != Some(true) {
        return;
    }
    if let Ok(ch) = channels::get(&db.conn, channel_id) {
        let _ = tx.send(ToastEvent::Show {
            channel: ch,
            new_total,
        });
    }
}
