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

/// Détermine si une date donnée tombe pendant l'heure d'été (Europe)
fn is_summer_time(day: u8, month: u8, year: u8) -> bool {
    // Janvier, Février, Novembre, Décembre -> Hiver
    if month < 3 || month > 10 { return false; }
    // Avril à Septembre -> Été
    if month > 3 && month < 10 { return true; }

    // Pour Mars et Octobre, le changement se fait le dernier dimanche
    // Calcul simplifié du dernier dimanche du mois
    let last_sunday = 31 - ((5 * year as u32 / 4 + 4) % 7) as u8;

    if month == 3 {
        day >= last_sunday // Après le dernier dimanche de Mars -> Été
    } else {
        day < last_sunday  // Avant le dernier dimanche d'Octobre -> Été
    }
}

pub fn get_time() -> RtcTime {
    // 1. Lecture des registres bruts (BCD)
    let seconds = read_rtc_register(0x00);
    let minutes = read_rtc_register(0x02);
    let hours   = read_rtc_register(0x04);
    let day     = read_rtc_register(0x07);
    let month   = read_rtc_register(0x08);
    let year    = read_rtc_register(0x09);

    // 2. Conversion BCD vers Décimal
    let s  = (seconds & 0x0F) + ((seconds / 16) * 10);
    let m  = (minutes & 0x0F) + ((minutes / 16) * 10);
    let mut h = (hours & 0x0F) + ((hours / 16) * 10);
    let d  = (day & 0x0F) + ((day / 16) * 10);
    let mo = (month & 0x0F) + ((month / 16) * 10);
    let y  = (year & 0x0F) + ((year / 16) * 10);

    // 3. Correction automatique du fuseau horaire (France)
    if is_summer_time(d, mo, y) {
        h = (h + 2) % 24; // UTC+2 en été
    } else {
        h = (h + 1) % 24; // UTC+1 en hiver
    }

    RtcTime {
        seconds: s,
        minutes: m,
        hours: h,
    }
}