use pc_keyboard::{layouts, HandleControl, Keyboard, ScancodeSet1, DecodedKey, KeyState};
use spin::Mutex;
use lazy_static::lazy_static;
use crossbeam_queue::ArrayQueue;

lazy_static! {
    pub static ref KEY_QUEUE: ArrayQueue<DecodedKey> = ArrayQueue::new(100);

    static ref KEYBOARD: Mutex<Keyboard<layouts::Azerty, ScancodeSet1>> =
        Mutex::new(Keyboard::new(
            ScancodeSet1::new(),
            layouts::Azerty,
            HandleControl::MapLettersToUnicode
        ));
}

pub fn init() {
    crate::serial_println!("[DRIVERS] Keyboard driver initialized (AZERTY - Set 1)");
}

pub fn add_scancode(scancode: u8) {
    let mut keyboard = KEYBOARD.lock();

    // CRUCIAL : On laisse keyboard traiter TOUS les octets (appuis ET relâchements)
    // pour que l'état interne du SHIFT et du CAPS_LOCK reste synchrone.
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        let state = key_event.state; // On mémorise si c'est Down ou Up
        
        // On traite l'événement pour mettre à jour l'état interne (Shift, etc.)
        if let Some(key) = keyboard.process_keyevent(key_event) {
            // MAIS on ne pousse dans la file du Shell QUE si la touche est pressée.
            // Si on ne faisait pas ça, relâcher 'A' renverrait un deuxième 'A'.
            if state == KeyState::Down {
                let _ = KEY_QUEUE.push(key);
            }
        }
    }
}

pub fn force_reset() {
    let mut keyboard = KEYBOARD.lock();
    *keyboard = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Azerty,
        HandleControl::MapLettersToUnicode
    );
    while KEY_QUEUE.pop().is_some() {}
}