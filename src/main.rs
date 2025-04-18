use bluer::adv::Advertisement;
use bluer::gatt::{
    CharacteristicWrite,
    local::{Application, Characteristic, CharacteristicNotify, Service},
};
use bluer::{Adapter, Address};
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

const HID_SERVICE_UUID: Uuid = Uuid::from_u128(0x1812_0000_0000_1000_8000_00805f9b34fb);
const HID_REPORT_UUID: Uuid = Uuid::from_u128(0x2a4d_0000_0000_1000_8000_00805f9b34fb);
const HID_REPORT_MAP_UUID: Uuid = Uuid::from_u128(0x2a4b_0000_0000_1000_8000_00805f9b34fb);

const HID_REPORT_MAP: &[u8] = &[
    0x05, 0x01, // Usage Page (Generic Desktop)
    0x09, 0x05, // Usage (Gamepad)
    0xA1, 0x01, // Collection (Application)
    0x85, 0x01, // Report ID (1)
    0x05, 0x09, // Usage Page (Buttons)
    0x19, 0x01, // Usage Minimum (1)
    0x29, 0x08, // Usage Maximum (8)
    0x15, 0x00, // Logical Minimum (0)
    0x25, 0x01, // Logical Maximum (1)
    0x95, 0x08, // Report Count (8)
    0x75, 0x01, // Report Size (1)
    0x81, 0x02, // Input (Data, Variable, Absolute)
    0xC0, // End Collection
];

#[tokio::main]
async fn main() -> bluer::Result<()> {
    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;
    adapter.set_powered(true).await?;

    println!(
        "Adapter {} powered: {}",
        adapter.name(),
        adapter.is_powered().await?
    );

    let mut app = Application::new();

    let mut service = Service::new(HID_SERVICE_UUID, true);

    let report_map_char = Characteristic::new(
        HID_REPORT_MAP_UUID,
        bluer::gatt::local::CharacteristicFlags::READ,
    )
    .with_value(HID_REPORT_MAP.to_vec());

    service.add_characteristic(report_map_char);

    let (tx, mut rx) = CharacteristicNotify::new();
    let input_report_char = Characteristic::new(
        HID_REPORT_UUID,
        bluer::gatt::local::CharacteristicFlags::NOTIFY,
    )
    .with_notify(tx);

    service.add_characteristic(input_report_char);

    app.add_service(service);

    let gatt = adapter.gatt_application();
    gatt.insert_application("/gamepad".into(), app).await?;

    let adv = adapter.advertisement().await?;
    adv.set_service_uuids(vec![HID_SERVICE_UUID]).await?;
    adv.set_local_name(Some("PS5 Gamepad")).await?;
    adv.set_discoverable(true).await?;
    adv.set_connectable(true).await?;
    adv.activate().await?;

    println!("🔵 Advertising as BLE HID gamepad...");

    let mut toggle = true;
    while let Some(notifier) = rx.recv().await {
        println!("🔔 Client connected, starting input loop");
        loop {
            let report: Vec<u8> = if toggle { vec![0x02] } else { vec![0x00] };
            toggle = !toggle;
            notifier.notify_value(report).await?;
            sleep(Duration::from_secs(1)).await;
        }
    }

    Ok(())
}
