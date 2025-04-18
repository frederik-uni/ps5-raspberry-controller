import time
from hidtools.device import HIDDevice
from hidtools.profile import HIDProfile
from hidtools.report import HIDReportMap, HIDInputReport

# Minimal PS5-like HID Descriptor
# Supports 8 buttons (Cross, Circle, Square, Triangle, etc.)
report_descriptor = [
    0x05, 0x01,       # Usage Page (Generic Desktop)
    0x09, 0x05,       # Usage (Game Pad)
    0xA1, 0x01,       # Collection (Application)
    0x85, 0x01,       # Report ID (1)

    0x05, 0x09,       # Usage Page (Button)
    0x19, 0x01,       # Usage Minimum (Button 1)
    0x29, 0x08,       # Usage Maximum (Button 8)
    0x15, 0x00,       # Logical Minimum (0)
    0x25, 0x01,       # Logical Maximum (1)
    0x95, 0x08,       # Report Count (8)
    0x75, 0x01,       # Report Size (1)
    0x81, 0x02,       # Input (Data, Var, Abs)

    0xC0              # End Collection
]

# Define the HID report map and input report
report_map = HIDReportMap(report_descriptor)
input_report = HIDInputReport(1, 1)  # Report ID 1, length 1 byte

# Create the HID profile and device
hid_profile = HIDProfile(name="DualSense Emulator", report_map=report_map)
device = HIDDevice(name="PS5 Controller", profile=hid_profile)

print("🔵 Advertising as PS5 Controller...")
device.start()

try:
    while True:
        # Emulate pressing Circle (typically Button 2 => 0b00000010)
        input_report.set_bytes([0x02])
        device.send_input_report(input_report)
        time.sleep(1)

        # Release the button
        input_report.set_bytes([0x00])
        device.send_input_report(input_report)
        time.sleep(1)
except KeyboardInterrupt:
    print("\n🛑 Stopping...")
    device.stop()
