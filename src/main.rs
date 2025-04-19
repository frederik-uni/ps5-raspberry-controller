use bluer::Session;
use bluer::adv::Advertisement;
use bluer::gatt::local::{
    Application, Characteristic, CharacteristicDescriptorPerm, CharacteristicNotify,
    CharacteristicNotifyMethod, CharacteristicRead, Descriptor, Service,
};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;
use uuid::Uuid;

const HID_SERVICE_UUID: Uuid = Uuid::from_u128(0x1812_0000_0000_1000_8000_00805f9b34fb);
const HID_REPORT_UUID: Uuid = Uuid::from_u128(0x2a4d_0000_0000_1000_8000_00805f9b34fb);
const HID_REPORT_MAP_UUID: Uuid = Uuid::from_u128(0x2a4b_0000_0000_1000_8000_00805f9b34fb);
const CCCD_UUID: Uuid = Uuid::from_u128(0x2902_0000_0000_1000_8000_00805f9b34fb);

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
    let session = Session::new().await?;
    let adapter = session.default_adapter().await?;
    adapter.set_powered(true).await?;

    // Create HID Service
    let mut service = Service {
        uuid: HID_SERVICE_UUID,
        primary: true,
        ..Default::default()
    };

    // Report Map Characteristic (Readable)
    service.characteristics.push(Characteristic {
        uuid: HID_REPORT_MAP_UUID,
        read: Some(CharacteristicRead {
            read: true,
            encrypt_read: false,
            encrypt_authenticated_read: false,
            secure_read: false,
            fun: Box::new(|| Box::pin(async { Ok(HID_REPORT_MAP.to_vec()) })),
            _non_exhaustive: (),
        }),
        ..Default::default()
    });

    // Input Report Characteristic (Notify)
    let (tx, _) = mpsc::channel(32);
    let report_tx = tx.clone();
    service.characteristics.push(Characteristic {
        uuid: HID_REPORT_UUID,
        notify: Some(CharacteristicNotify {
            notify: true,
            indicate: false,
            method: CharacteristicNotifyMethod::Channel(tx),
            _non_exhaustive: (),
        }),
        descriptors: vec![Descriptor {
            uuid: CCCD_UUID,
            read: Some(CharacteristicDescriptorPerm {
                permitted: true,
                requires_encryption: false,
            }),
            write: Some(CharacteristicDescriptorPerm {
                permitted: true,
                requires_encryption: false,
            }),
            ..Default::default()
        }],
        ..Default::default()
    });

    // Create GATT Application
    let app = Application {
        services: vec![service],
        ..Default::default()
    };

    let _app_handle = adapter.serve_gatt_application(app).await?;

    // Configure Advertising
    let adv = Advertisement {
        service_uuids: vec![HID_SERVICE_UUID].into_iter().collect(),
        local_name: Some("PS5 Gamepad".into()),
        discoverable: Some(true),
        ..Default::default()
    };

    let _adv_handle = adapter.advertise(adv).await?;
    println!("🕹 BLE HID Gamepad Advertising");

    // Simulation loop
    let mut state = false;
    loop {
        state = !state;
        let report = vec![if state { 0x01 } else { 0x00 }];

        match report_tx.send(report).await {
            Ok(_) => sleep(Duration::from_secs(1)).await,
            Err(e) => {
                eprintln!("Notification failed: {}", e);
                sleep(Duration::from_secs(1)).await;
            }
        }
    }
}
