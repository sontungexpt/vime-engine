use std::io::{self, Read};
use vime_engine::{Config, Engine, Key, KeyEvent, Result};

fn main() -> io::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let mut engine = Engine::new(Config::default());

    for character in input.chars() {
        let key = match character {
            ' ' => Key::Space,
            '\n' => Key::Enter,
            '\t' => Key::Tab,
            character => Key::Character(character),
        };
        match engine.process_key(KeyEvent::key(key)) {
            Result::Changed => eprint!("\r\x1b[2K{}", engine.rendered()),
            Result::Commit(text) => print!("{text}"),
            Result::Noop | Result::Forward => {}
        }
    }
    if let Result::Commit(text) = engine.commit() {
        print!("{text}");
    }
    Ok(())
}