use std::{env, sync::Arc};

#[cfg(target_os = "linux")]
use bluetooth::DualSenseController;
#[cfg(not(target_os = "linux"))]
use bluetooth_faker::DualSenseController;
use hidapi::HidApi;
use interfaces::{bluetooth::ControllerState, internal::ControllerStateInternal, usb::ParsedInput};

#[cfg(target_os = "linux")]
mod bluetooth;
#[cfg(not(target_os = "linux"))]
mod bluetooth_faker;
pub mod interfaces;

async fn init_bluetooth() -> Arc<DualSenseController> {
    let controller = Arc::new(DualSenseController::new());
    controller.initialize_bluetooth().await.unwrap();

    let report_controller = controller.clone();
    tokio::spawn(async move {
        report_controller.run_report_loop().await;
    });
    controller
}

fn parse_vid_pid(args: &[String], default_vendor_id: u16, default_product_id: u16) -> (u16, u16) {
    args.iter()
        .position(|v| v.contains(":"))
        .map(|id_arg_pos| {
            let id_arg = &args[id_arg_pos];
            let parts: Vec<&str> = id_arg.split(':').collect();
            if parts.len() == 2 {
                let v_id = u16::from_str_radix(parts[0], 16).unwrap_or(default_vendor_id);
                let p_id = u16::from_str_radix(parts[1], 16).unwrap_or(default_product_id);
                (v_id, p_id)
            } else {
                (default_vendor_id, default_product_id)
            }
        })
        .unwrap_or((default_vendor_id, default_product_id))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().collect::<Vec<String>>();
    if args.iter().find(|v| v.as_str() == "list-devices").is_some() {
        let api = HidApi::new()?;
        for device in api.device_list() {
            println!(
                "{:04x}:{:04x} - {}",
                device.vendor_id(),
                device.product_id(),
                device.product_string().unwrap_or("Unknown")
            );
        }
    } else if args.iter().find(|v| v.as_str() == "ps5").is_some() {
        let controller = init_bluetooth().await;
        let (vendor_id, product_id) = parse_vid_pid(&args, 0x054C, 0x0CE6);

        let api = HidApi::new()?;
        let device = api.open(vendor_id, product_id)?;
        let mut buf = [0u8; 64];
        println!(
            "Reading from USB device {:04x}:{:04x}...",
            vendor_id, product_id
        );
        loop {
            match device.read(&mut buf) {
                Ok(_) => {
                    let parsed = ControllerStateInternal::from(ParsedInput::from_ps5_buf(&buf));
                    // dbg!(&parsed);
                    controller.update_state(move |state| {
                        *state = ControllerState::from(parsed);
                    });
                }
                Err(e) => {
                    eprintln!("Read error: {:?}", e);
                    break;
                }
            }
        }
    } else if args.iter().find(|v| v.as_str() == "ps4").is_some() {
        let controller = init_bluetooth().await;
        let (vendor_id, product_id) = parse_vid_pid(&args, 0x054C, 0x09CC);

        let api = HidApi::new()?;
        let device = api.open(vendor_id, product_id)?;
        let mut buf = [0u8; 64];
        println!(
            "Reading from USB device {:04x}:{:04x}...",
            vendor_id, product_id
        );
        loop {
            match device.read(&mut buf) {
                Ok(_) => {
                    let parsed = ControllerStateInternal::from(ParsedInput::from_ps5_buf(&buf));
                    controller.update_state(move |state| {
                        *state = ControllerState::from(parsed);
                    });
                }
                Err(e) => {
                    eprintln!("Read error: {:?}", e);
                    break;
                }
            }
        }
    } else {
        panic!("invalid mode")
    }

    tokio::signal::ctrl_c().await?;
    Ok(())
}
