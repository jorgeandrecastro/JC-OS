use pc_keyboard::{layouts, HandleControl, Keyboard, ScancodeSet2, DecodedKey, KeyState};
use spin::Mutex;
use lazy_static::lazy_static;
use crossbeam_queue::ArrayQueue;

lazy_static! {
    /// File d'attente pour les touches décodées
    pub static ref KEY_QUEUE: ArrayQueue<DecodedKey> = ArrayQueue::new(100);

    /// État interne du clavier (AZERTY)
    static ref KEYBOARD: Mutex<Keyboard<layouts::Azerty, ScancodeSet2>> =
        Mutex::new(Keyboard::new(
            ScancodeSet2::new(),
            layouts::Azerty,
           HandleControl::MapLettersToUnicode
        ));
}

/// Initialise le driver (appelé par main.rs)
pub fn init() {
    // On peut logger l'init via le port série si besoin
    crate::serial_println!("[DRIVERS] Keyboard driver initialized (AZERTY)");
}
/// Ajoute un scancode brut à la machine à états (Version corrigée pour Ctrl)
pub fn add_scancode(scancode: u8) {
    let mut keyboard = KEYBOARD.lock();

    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        // ÉTAPE 1 : On mémorise l'état AVANT que l'événement ne soit déplacé
        let is_down = key_event.state == KeyState::Down;

        // ÉTAPE 2 : On donne l'événement au décodeur (le 'move' se produit ici)
        let key = keyboard.process_keyevent(key_event);

        // ÉTAPE 3 : On utilise notre variable mémorisée
        if is_down {
            if let Some(key) = key {
                let _ = KEY_QUEUE.push(key);
            }
        }
    }
}