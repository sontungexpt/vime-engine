use vime_engine::{Engine, Key, KeyEvent};
fn main() {
    let mut engine = Engine::default();
    for ch in "nguowif".chars() {
        engine.process_key(KeyEvent::key(Key::Character(ch)));
    }
    println!(
        "raw={} rendered={}",
        engine.keystrokes().to_string(),
        engine.rendered()
    );
    // replicate telex oo case too
    let mut e2 = Engine::default();
    for ch in "oo".chars() {
        e2.process_key(KeyEvent::key(Key::Character(ch)));
    }
    println!(
        "oo raw={} rendered={}",
        e2.keystrokes().to_string(),
        e2.rendered()
    );
}