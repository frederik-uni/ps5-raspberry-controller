use bluer::Session;
use bluer::adv::Advertisement;
use bluer::gatt::local::{
    Application, Characteristic, CharacteristicNotify, CharacteristicNotifyMethod,
    CharacteristicRead, CharacteristicWrite, Descriptor, DescriptorRead, DescriptorWrite, Service,
};
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::time::sleep;
use uuid::Uuid;

// Correct UUIDs (16-bit UUIDs in proper 128-bit format)
const HID_SERVICE_UUID: Uuid = Uuid::from_u128(0x00001812_0000_1000_8000_00805f9b34fb);
const HID_REPORT_UUID: Uuid = Uuid::from_u128(0x00002A4D_0000_1000_8000_00805f9b34fb);
const HID_REPORT_MAP_UUID: Uuid = Uuid::from_u128(0x00002A4B_0000_1000_8000_00805f9b34fb);
const HID_PROTOCOL_MODE_UUID: Uuid = Uuid::from_u128(0x00002A4E_0000_1000_8000_00805f9b34fb);
const HID_INFORMATION_UUID: Uuid = Uuid::from_u128(0x00002A4A_0000_1000_8000_00805f9b34fb);
const CCCD_UUID: Uuid = Uuid::from_u128(0x00002902_0000_1000_8000_00805f9b34fb);

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

    // Create HID Service with mandatory characteristics
    let mut service = Service {
        uuid: HID_SERVICE_UUID,
        primary: true,
        ..Default::default()
    };

    // Protocol Mode Characteristic (Mandatory)
    service.characteristics.push(Characteristic {
        uuid: HID_PROTOCOL_MODE_UUID,
        read: Some(CharacteristicRead {
            read: true,
            ..Default::default()
        }),
        write: Some(CharacteristicWrite {
            write: true,
            ..Default::default()
        }),
        ..Default::default()
    });

    // HID Information Characteristic (Mandatory)
    service.characteristics.push(Characteristic {
        uuid: HID_INFORMATION_UUID,
        read: Some(CharacteristicRead {
            read: true,
            ..Default::default()
        }),
        ..Default::default()
    });

    // Report Map Characteristic
    service.characteristics.push(Characteristic {
        uuid: HID_REPORT_MAP_UUID,
        read: Some(CharacteristicRead {
            read: true,
            ..Default::default()
        }),
        ..Default::default()
    });

    // Input Report Characteristic (Notify)
    let (report_tx, mut report_rx) = broadcast::channel(32);
    service.characteristics.push(Characteristic {
        uuid: HID_REPORT_UUID,
        notify: Some(CharacteristicNotify {
            notify: true,
            indicate: false,
            method: CharacteristicNotifyMethod::Fun(Box::new(move |mut stream| {
                let report_rx = report_rx.resubscribe();
                Box::pin(async move {
                    while let Ok(report) = report_rx.recv().await {
                        if let Err(e) = stream.notify(report).await {
                            eprintln!("Failed to send notification: {}", e);
                            break;
                        }
                    }
                })
            })),
            _non_exhaustive: (),
        }),
        descriptors: vec![Descriptor {
            uuid: CCCD_UUID,
            read: Some(DescriptorRead {
                read: true,
                ..Default::default()
            }),
            write: Some(DescriptorWrite {
                write: true,
                ..Default::default()
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

        if let Err(e) = report_tx.send(report) {
            eprintln!("Failed to queue notification: {}", e);
        }
        sleep(Duration::from_secs(1)).await;
    }
}
