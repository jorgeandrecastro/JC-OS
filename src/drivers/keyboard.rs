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
            HandleControl::Ignore
        ));
}

/// Initialise le driver (appelé par main.rs)
pub fn init() {
    // On peut logger l'init via le port série si besoin
    crate::serial_println!("[DRIVERS] Keyboard driver initialized (AZERTY)");
}

/// Ajoute un scancode brut à la machine à états
pub fn add_scancode(scancode: u8) {
    let mut keyboard = KEYBOARD.lock();

    // ÉTAPE 1 : On donne TOUJOURS le scancode au décodeur.
    // C'est ici que le driver voit passer le "KeyUp" du Shift et met à jour son état interne.
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        
        // ÉTAPE 2 : On ne s'intéresse qu'aux touches que l'on vient d'enfoncer pour le Shell.
        if key_event.state == KeyState::Down {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                // On pousse dans la file
                let _ = KEY_QUEUE.push(key);
            }
        } else {
            // ÉTAPE 3 : Pour les touches relâchées (Up), on appelle quand même process_keyevent.
            // Cela permet au driver de finaliser le changement d'état interne (Majuscules, Alt, etc.)
            // sans forcément envoyer de caractère au Shell.
            let _ = keyboard.process_keyevent(key_event);
        }
    }
}