#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum DescType { Device = 0x01, Config = 0x02, String = 0x03, Interface = 0x04, Endpoint = 0x05, DeviceQual = 0x06, HidClass = 0x21, HidReport = 0x22 }

#[repr(C, packed)] pub struct DescHeader  { pub length: u8, pub desc_type: u8 }
#[repr(C, packed)] pub struct DeviceDesc  {
    pub length: u8, pub desc_type: u8, pub bcd_usb: u16,
    pub device_class: u8, pub device_subclass: u8, pub device_protocol: u8,
    pub max_packet_size: u8, pub vendor_id: u16, pub product_id: u16,
    pub bcd_device: u16, pub manufacturer: u8, pub product: u8,
    pub serial_number: u8, pub num_configs: u8,
}

pub const HID_CLASS:          u8 = 0x03;
pub const HID_SUBCLASS_BOOT:  u8 = 0x01;
pub const HID_PROTO_KEYBOARD: u8 = 0x01;
pub const HID_PROTO_MOUSE:    u8 = 0x02;

pub fn parse_header(buf: &[u8]) -> Option<DescType> {
    if buf.len() < 2 { return None; }
    match buf[1] {
        0x01 => Some(DescType::Device),    0x02 => Some(DescType::Config),
        0x03 => Some(DescType::String),    0x04 => Some(DescType::Interface),
        0x05 => Some(DescType::Endpoint),  0x21 => Some(DescType::HidClass),
        _ => None,
    }
}
