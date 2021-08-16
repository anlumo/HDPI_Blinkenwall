use spidev::{Spidev, SpidevOptions, SpiModeFlags};
use gpio_cdev::LineHandle;
use std::{io, io::Write, time::Duration};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Led {
    Relay = 0x00,
    Hd1 = 0x01,
    Hd2 = 0x02,
    Lan1 = 0x03,
    Lan2 = 0x04,
    Eth1 = 0x05,
    Eth2 = 0x06,
    Earth = 0x07,
    Wrench = 0xff,
}

pub struct LedControl {
    state: u8,
    spi: Spidev,
    rck: LineHandle,
    wrench: LineHandle,
}

impl LedControl {
    pub fn new(rck: LineHandle, wrench: LineHandle) -> io::Result<Self> {
        let mut spi = Spidev::open("/dev/spidev2.0")?;
        let options = SpidevOptions::new()
                .bits_per_word(8)
                .max_speed_hz(10_000_000)
                .mode(SpiModeFlags::SPI_MODE_0)
                .build();
        spi.configure(&options)?;
        Ok(Self {
            state: 0,
            spi,
            rck,
            wrench,
        })
    }

    fn write_leds(&mut self) -> Result<(), gpio_cdev::Error> {
        log::info!("Setting leds to {:08b}", self.state);
        self.spi.write(&[self.state])?;
        std::thread::sleep(Duration::from_millis(1));
        self.rck.set_value(1)?;
        std::thread::sleep(Duration::from_millis(1));
        self.rck.set_value(0)?;
        Ok(())
    }

    pub fn set(&mut self, led: Led, value: bool) -> Result<(), gpio_cdev::Error> {
        if led == Led::Wrench {
            self.wrench.set_value(if value { 1 } else { 0 })?;
            Ok(())
        } else {
            if value {
                self.state |= 1 << (led as u8);
            } else {
                self.state &= !(1 << (led as u8));
            }
            self.write_leds()
        }
    }
}
