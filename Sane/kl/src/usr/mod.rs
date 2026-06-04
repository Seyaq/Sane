pub mod drivers;
pub mod clowns;

pub fn boot() {
    drivers::nvme::start();
    drivers::usb::keyboard::start();
    drivers::usb::mouse::start();
    drivers::bus::pci::start();
}
