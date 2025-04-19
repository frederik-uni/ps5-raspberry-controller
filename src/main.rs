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
const DUALSHOCK_SERVICE_UUID: Uuid = Uuid::from_u128(0x00040000_0000_1000_8000_00805f9b34fb);
const HID_REPORT_UUID: Uuid = Uuid::from_u128(0x00002A4D_0000_1000_8000_00805f9b34fb);
const CCCD_UUID: Uuid = Uuid::from_u128(0x00002902_0000_1000_8000_00805f9b34fb);
const REPORT_CHARACTERISTIC_UUID: Uuid = Uuid::from_u128(0x00040001_0000_1000_8000_00805f9b34fb);
const REPORT_MAP_CHARACTERISTIC_UUID: Uuid =
    Uuid::from_u128(0x00040002_0000_1000_8000_00805f9b34fb);

const HID_REPORT_MAP: &[u8] = &[
    0x05, 0x01, // Usage Page (Generic Desktop)
    0x09, 0x05, // Usage (Gamepad)
    0xA1, 0x01, // Collection (Application)
    0x85, 0x01, //   Report ID (1)
    0x09, 0x30, //   Usage (X)
    0x09, 0x31, //   Usage (Y)
    0x15, 0x00, //   Logical Minimum (0)
    0x26, 0xFF, 0x00, //   Logical Maximum (255)
    0x75, 0x08, //   Report Size (8)
    0x95, 0x02, //   Report Count (2)
    0x81, 0x02, //   Input (Data,Var,Abs)
    0x05, 0x09, //   Usage Page (Button)
    0x19, 0x01, //   Usage Minimum (1)
    0x29, 0x0E, //   Usage Maximum (14)
    0x15, 0x00, //   Logical Minimum (0)
    0x25, 0x01, //   Logical Maximum (1)
    0x75, 0x01, //   Report Size (1)
    0x95, 0x0E, //   Report Count (14)
    0x81, 0x02, //   Input (Data,Var,Abs)
    0x05, 0x01, //   Usage Page (Generic Desktop)
    0x09, 0x39, //   Usage (Hat switch)
    0x15, 0x00, //   Logical Minimum (0)
    0x25, 0x07, //   Logical Maximum (7)
    0x35, 0x00, //   Physical Minimum (0)
    0x46, 0x3B, 0x01, //   Physical Maximum (315)
    0x65, 0x14, //   Unit (English Rotation: Degrees)
    0x75, 0x04, //   Report Size (4)
    0x95, 0x01, //   Report Count (1)
    0x81, 0x02, //   Input (Data,Var,Abs)
    0x05, 0x02, //   Usage Page (Simulation Controls)
    0x09, 0xC5, //   Usage (Brake)
    0x09, 0xC4, //   Usage (Accelerator)
    0x15, 0x00, //   Logical Minimum (0)
    0x26, 0xFF, 0x00, //   Logical Maximum (255)
    0x75, 0x08, //   Report Size (8)
    0x95, 0x02, //   Report Count (2)
    0x81, 0x02, //   Input (Data,Var,Abs)
    0x06, 0x00, 0xFF, //   Usage Page (Vendor Defined)
    0x09, 0x20, //   Usage (0x20)
    0x75, 0x08, //   Report Size (8)
    0x95, 0x3F, //   Report Count (63)
    0x81, 0x02, //   Input (Data,Var,Abs)
    0xC0, // End Collection
];

#[tokio::main]
async fn main() -> bluer::Result<()> {
    let session = Session::new().await?;
    let adapter = session.default_adapter().await?;
    adapter.set_powered(true).await?;

    // Create HID Service with mandatory characteristics
    let mut service = Service {
        uuid: DUALSHOCK_SERVICE_UUID,
        primary: true,
        ..Default::default()
    };

    // Protocol Mode Characteristic (Mandatory)
    service.characteristics.push(Characteristic {
        uuid: REPORT_CHARACTERISTIC_UUID,
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

    // Report Map Characteristic
    service.characteristics.push(Characteristic {
        uuid: REPORT_MAP_CHARACTERISTIC_UUID,
        read: Some(CharacteristicRead {
            read: true,
            fun: Some(Box::new(|| {
                Box::pin(async move { Ok(HID_REPORT_MAP.to_vec()) })
            })),
            ..Default::default()
        }),
        ..Default::default()
    });

    // Input Report Characteristic (Notify)
    let (report_tx, report_rx) = broadcast::channel(32);
    service.characteristics.push(Characteristic {
        uuid: REPORT_CHARACTERISTIC_UUID,
        notify: Some(CharacteristicNotify {
            notify: true,
            indicate: false,
            method: CharacteristicNotifyMethod::Fun(Box::new(move |mut stream| {
                let mut report_rx = report_rx.resubscribe();
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
        service_uuids: vec![DUALSHOCK_SERVICE_UUID].into_iter().collect(),
        local_name: Some("Wireless Controller".into()),
        discoverable: Some(true),
        manufacturer_data: vec![
            (0x054C, vec![0x09, 0x05, 0xC0, 0xCA, 0x2C, 0x00]), // Sony's company ID (0x054C)
        ]
        .into_iter()
        .collect(),
        appearance: Some(0x03C4), // HID Major (0x03) + Gamepad (0xC4)
        ..Default::default()
    };

    let _adv_handle = adapter.advertise(adv).await?;
    println!("🎮 PS5 DualSense Controller Advertising");

    // Simulation loop
    let mut counter = 0;
    loop {
        // Create a basic report matching PS5 controller structure (78 bytes)
        let mut report = vec![0; 78];
        report[0] = 0x01; // Report ID
        report[1] = 0x80; // Left stick X
        report[2] = 0x80; // Left stick Y
        report[3] = 0x80; // Right stick X
        report[4] = 0x80; // Right stick Y
        report[5] = (counter % 2) << 1; // Buttons state
        report[6] = 0x08; // Buttons state continued
        report[7] = 0x00; // Buttons state continued

        // Add vibration data
        report[8] = 0xFF; // Left motor
        report[9] = 0xFF; // Right motor

        // Add timestamp
        let timestamp = (counter % 65536) as u16;
        report[10..12].copy_from_slice(&timestamp.to_le_bytes());

        if let Err(e) = report_tx.send(report) {
            eprintln!("Failed to queue notification: {}", e);
        }

        counter += 1;
        sleep(Duration::from_millis(16)).await; // ~60Hz update rate
    }
}
