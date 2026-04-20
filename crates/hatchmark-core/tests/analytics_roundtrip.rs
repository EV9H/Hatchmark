use hatchmark_core::db::{bindings, channels, events, settings, Db};
use hatchmark_core::model::{Binding, BindingAction};

#[test]
fn key_resolves_and_increments_today() {
    let db = Db::open_memory().unwrap();
    let cid = channels::create(&db.conn, "Water", "#7dd3fc", None, None).unwrap();
    bindings::upsert(
        &db.conn,
        &Binding {
            layer_id: 1,
            key_code: "F13".into(),
            action: BindingAction::Increment { channel_id: cid },
        },
    )
    .unwrap();
    let layer = settings::current_layer_id(&db.conn).unwrap();
    let action = bindings::resolve(&db.conn, layer, "F13").unwrap().unwrap();
    let BindingAction::Increment { channel_id } = action else {
        panic!("wrong action");
    };
    let total = events::insert(&db.conn, channel_id).unwrap();
    assert_eq!(total, 1);
    let rows = hatchmark_core::analytics::today_per_channel(&db.conn).unwrap();
    assert_eq!(rows.iter().find(|r| r.channel_id == cid).unwrap().count, 1);
}
