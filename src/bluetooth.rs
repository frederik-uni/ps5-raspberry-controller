use bluer::Session;
use bluer::adv::Advertisement;
use bluer::agent::Agent;
use bluer::gatt::local::{
    Application, Characteristic, CharacteristicNotify, CharacteristicNotifyMethod,
    CharacteristicRead, CharacteristicWrite, Descriptor, DescriptorRead, DescriptorWrite, Service,
};
use futures_util::StreamExt;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::time::sleep;
use uuid::Uuid;

use crate::interfaces::bluetooth::ControllerState;

// Correct UUIDs (16-bit UUIDs in proper 128-bit format)
const DUALSHOCK_SERVICE_UUID: Uuid = bluetooth_uuid_from_u16(0x1812);
const HID_REPORT_UUID: Uuid = Uuid::from_u128(0x00002A4D_0000_1000_8000_00805f9b34fb);
const CCCD_UUID: Uuid = Uuid::from_u128(0x00002902_0000_1000_8000_00805f9b34fb);
const REPORT_CHARACTERISTIC_UUID: Uuid = Uuid::from_u128(0x00040001_0000_1000_8000_00805f9b34fb);
const REPORT_MAP_CHARACTERISTIC_UUID: Uuid =
    Uuid::from_u128(0x00040002_0000_1000_8000_00805f9b34fb);

const HID_REPORT_MAP: &[u8] = &[
    0x05, 0x01, // Usage Page (Generic Desktop Ctrls)
    0x09, 0x05, // Usage (Game Pad)
    0xA1, 0x01, // Collection (Application)
    0x85, 0x01, //   Report ID (1)
    0x09, 0x30, //   Usage (X)
    0x09, 0x31, //   Usage (Y)
    0x09, 0x32, //   Usage (Z)
    0x09, 0x35, //   Usage (Rz)
    0x15, 0x00, //   Logical Minimum (0)
    0x26, 0xFF, 0x00, //   Logical Maximum (255)
    0x75, 0x08, //   Report Size (8)
    0x95, 0x04, //   Report Count (4)
    0x81, 0x02, //   Input (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    0x09, 0x39, //   Usage (Hat switch)
    0x15, 0x00, //   Logical Minimum (0)
    0x25, 0x07, //   Logical Maximum (7)
    0x35, 0x00, //   Physical Minimum (0)
    0x46, 0x3B, 0x01, //   Physical Maximum (315)
    0x65, 0x14, //   Unit (System: English Rotation, Length: Centimeter)
    0x75, 0x04, //   Report Size (4)
    0x95, 0x01, //   Report Count (1)
    0x81, 0x42, //   Input (Data,Var,Abs,No Wrap,Linear,Preferred State,Null State)
    0x65, 0x00, //   Unit (None)
    0x05, 0x09, //   Usage Page (Button)
    0x19, 0x01, //   Usage Minimum (0x01)
    0x29, 0x0E, //   Usage Maximum (0x0E)
    0x15, 0x00, //   Logical Minimum (0)
    0x25, 0x01, //   Logical Maximum (1)
    0x75, 0x01, //   Report Size (1)
    0x95, 0x0E, //   Report Count (14)
    0x81, 0x02, //   Input (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    0x75, 0x06, //   Report Size (6)
    0x95, 0x01, //   Report Count (1)
    0x81, 0x01, //   Input (Const,Array,Abs,No Wrap,Linear,Preferred State,No Null Position)
    0x05, 0x01, //   Usage Page (Generic Desktop Ctrls)
    0x09, 0x33, //   Usage (Rx)
    0x09, 0x34, //   Usage (Ry)
    0x15, 0x00, //   Logical Minimum (0)
    0x26, 0xFF, 0x00, //   Logical Maximum (255)
    0x75, 0x08, //   Report Size (8)
    0x95, 0x02, //   Report Count (2)
    0x81, 0x02, //   Input (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    0x06, 0x00, 0xFF, //   Usage Page (Vendor Defined 0xFF00)
    0x15, 0x00, //   Logical Minimum (0)
    0x26, 0xFF, 0x00, //   Logical Maximum (255)
    0x75, 0x08, //   Report Size (8)
    0x95, 0x4D, //   Report Count (77)
    0x85, 0x31, //   Report ID (49)
    0x09, 0x31, //   Usage (0x31)
    0x91,
    0x02, //   Output (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
    0x09, 0x3B, //   Usage (0x3B)
    0x81, 0x02, //   Input (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    0x85, 0x32, //   Report ID (50)
    0x09, 0x32, //   Usage (0x32)
    0x95, 0x8D, //   Report Count (-115)
    0x91,
    0x02, //   Output (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
    0x85, 0x33, //   Report ID (51)
    0x09, 0x33, //   Usage (0x33)
    0x95, 0xCD, //   Report Count (-51)
    0x91,
    0x02, //   Output (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
    0x85, 0x34, //   Report ID (52)
    0x09, 0x34, //   Usage (0x34)
    0x96, 0x0D, 0x01, //   Report Count (269)
    0x91,
    0x02, //   Output (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
    0x85, 0x35, //   Report ID (53)
    0x09, 0x35, //   Usage (0x35)
    0x96, 0x4D, 0x01, //   Report Count (333)
    0x91,
    0x02, //   Output (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
    0x85, 0x36, //   Report ID (54)
    0x09, 0x36, //   Usage (0x36)
    0x96, 0x8D, 0x01, //   Report Count (397)
    0x91,
    0x02, //   Output (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
    0x85, 0x37, //   Report ID (55)
    0x09, 0x37, //   Usage (0x37)
    0x96, 0xCD, 0x01, //   Report Count (461)
    0x91,
    0x02, //   Output (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
    0x85, 0x38, //   Report ID (56)
    0x09, 0x38, //   Usage (0x38)
    0x96, 0x0D, 0x02, //   Report Count (525)
    0x91,
    0x02, //   Output (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
    0x85, 0x39, //   Report ID (57)
    0x09, 0x39, //   Usage (0x39)
    0x96, 0x22, 0x02, //   Report Count (546)
    0x91,
    0x02, //   Output (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
    0x06, 0x80, 0xFF, //   Usage Page (Vendor Defined 0xFF80)
    0x85, 0x05, //   Report ID (5)
    0x09, 0x33, //   Usage (0x33)
    0x95, 0x28, //   Report Count (40)
    0xB1,
    0x02, //   Feature (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
    0x85, 0x08, //   Report ID (8)
    0x09, 0x34, //   Usage (0x34)
    0x95, 0x2F, //   Report Count (47)
    0xB1,
    0x02, //   Feature (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
    0x85, 0x09, //   Report ID (9)
    0x09, 0x24, //   Usage (0x24)
    0x95, 0x13, //   Report Count (19)
    0xB1,
    0x02, //   Feature (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
    0x85, 0x20, //   Report ID (32)
    0x09, 0x26, //   Usage (0x26)
    0x95, 0x3F, //   Report Count (63)
    0xB1,
    0x02, //   Feature (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
    0x85, 0x22, //   Report ID (34)
    0x09, 0x40, //   Usage (0x40)
    0x95, 0x3F, //   Report Count (63)
    0xB1,
    0x02, //   Feature (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
    0x85, 0x80, //   Report ID (-128)
    0x09, 0x28, //   Usage (0x28)
    0x95, 0x3F, //   Report Count (63)
    0xB1,
    0x02, //   Feature (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
    0x85, 0x81, //   Report ID (-127)
    0x09, 0x29, //   Usage (0x29)
    0x95, 0x3F, //   Report Count (63)
    0xB1,
    0x02, //   Feature (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
    0x85, 0x82, //   Report ID (-126)
    0x09, 0x2A, //   Usage (0x2A)
    0x95, 0x09, //   Report Count (9)
    0xB1,
    0x02, //   Feature (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
    0x85, 0x83, //   Report ID (-125)
    0x09, 0x2B, //   Usage (0x2B)
    0x95, 0x3F, //   Report Count (63)
    0xB1,
    0x02, //   Feature (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
    0x85, 0xF1, //   Report ID (-15)
    0x09, 0x31, //   Usage (0x31)
    0x95, 0x3F, //   Report Count (63)
    0xB1,
    0x02, //   Feature (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
    0x85, 0xF2, //   Report ID (-14)
    0x09, 0x32, //   Usage (0x32)
    0x95, 0x0F, //   Report Count (15)
    0xB1,
    0x02, //   Feature (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
    0x85, 0xF0, //   Report ID (-16)
    0x09, 0x30, //   Usage (0x30)
    0x95, 0x3F, //   Report Count (63)
    0xB1,
    0x02, //   Feature (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
    0xC0, // End Collection
    0x00, // Unknown (bTag: 0x00, bType: 0x00)
];
const fn bluetooth_uuid_from_u16(uuid16: u16) -> Uuid {
    const BASE: u128 = 0x00000000_0000_1000_8000_00805F9B34FB;
    Uuid::from_u128(((uuid16 as u128) << 96) | BASE)
}

pub struct DualSenseController {
    state: Arc<Mutex<ControllerState>>,
    report_tx: Arc<Mutex<broadcast::Sender<Vec<u8>>>>,
}

impl DualSenseController {
    pub fn new() -> Self {
        let (report_tx, _) = broadcast::channel(32);
        Self {
            state: Arc::new(Mutex::new(ControllerState::default())),
            report_tx: Arc::new(Mutex::new(report_tx)),
        }
    }

    pub fn get_state(&self) -> ControllerState {
        let state = self.state.lock().unwrap();
        state.clone()
    }

    pub fn update_state<F>(&self, update_fn: F)
    where
        F: FnOnce(&mut ControllerState),
    {
        let mut state = self.state.lock().unwrap();
        update_fn(&mut state);
    }

    pub async fn run_report_loop(&self) {
        let report_tx = self.report_tx.lock().unwrap().clone();
        loop {
            {
                let state = self.state.lock().unwrap();
                let _ = report_tx.send(state.to_bytes().to_vec());
            }
            sleep(Duration::from_millis(16)).await
        }
    }

    pub async fn initialize_bluetooth(&self) -> bluer::Result<()> {
        let session = Session::new().await?;
        session.register_agent(Agent {
            request_default: true,
            request_pin_code: Some(Box::new(|_device| {
                Box::pin(async { Ok("0000".to_string()) }) // auto-accept PIN
            })),
            request_passkey: Some(Box::new(|_device| {
                Box::pin(async { Ok(123456) }) // auto-accept passkey
            })),
            request_confirmation: Some(Box::new(|_device| {
                Box::pin(async { Ok(()) }) // auto-confirm pairing
            })),
            request_authorization: Some(Box::new(|_device| {
                Box::pin(async { Ok(()) }) // auto-authorize device
            })),
            authorize_service: Some(Box::new(|_device| {
                Box::pin(async { Ok(()) }) // auto-authorize service
            })),
            display_pin_code: None,
            display_passkey: None,
            _non_exhaustive: (),
        });
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
                fun: Box::new(|_| Box::pin(async move { Ok(HID_REPORT_MAP.to_vec()) })),
                ..Default::default()
            }),
            ..Default::default()
        });

        // Input Report Characteristic (Notify)
        let report_rx = self.report_tx.lock().unwrap().subscribe();
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

        let app_handle = adapter.serve_gatt_application(app).await?;

        //TODO: product id: 0x0ce6
        // Configure Advertising
        let adv = Advertisement {
            service_uuids: vec![DUALSHOCK_SERVICE_UUID].into_iter().collect(),
            local_name: Some("Pad".into()),
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
        println!("🎮 PS5 DualSense Controller Advertising ");
        let mut device_events = adapter.discover_devices().await?;
        println!("Waiting for device connection...");

        while let Some(evt) = device_events.next().await {
            match evt {
                bluer::AdapterEvent::DeviceAdded(addr) => {
                    let device = adapter.device(addr)?;
                    device.set_trusted(true).await?;

                    println!("Device connected: {}", addr);
                    println!("Device name: {:?}", device.name().await?);

                    // Wait for GATT connection
                    while !device.is_connected().await? {
                        sleep(Duration::from_millis(100)).await;
                    }

                    println!("GATT connection established! Ready for input.");
                    return Ok(());
                }
                _ => {}
            }
        }
        panic!("Device not found");
        Ok(())
    }
}
