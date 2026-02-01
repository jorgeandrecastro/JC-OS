use x86_64::instructions::port::Port;

pub struct RtcTime {
    pub seconds: u8,
    pub minutes: u8,
    pub hours: u8,
}

fn read_rtc_register(reg: u8) -> u8 {
    unsafe {
        let mut addr_port = Port::new(0x70);
        let mut data_port = Port::new(0x71);
        addr_port.write(reg);
        data_port.read()
    }
}

pub fn get_time() -> RtcTime {
    // Note: Ce code lit le format BCD (Binary Coded Decimal) standard du RTC
    let seconds = read_rtc_register(0x00);
    let minutes = read_rtc_register(0x02);
    let hours = read_rtc_register(0x04);

    // Conversion BCD vers Décimal
    RtcTime {
        seconds: (seconds & 0x0F) + ((seconds / 16) * 10),
        minutes: (minutes & 0x0F) + ((minutes / 16) * 10),
        hours: (hours & 0x0F) + ((hours / 16) * 10),
    }
}