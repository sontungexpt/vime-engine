use vime_engine::composition::syllable::SyllableContext;
use vime_engine::{Config, DefaultKeymap, Keymap, Session};

type KM = DefaultKeymap<'static>;

/// Proposal A: both variants hold the same type.
enum TwoVariant<KM: Keymap + Clone> {
    Following(Config<KM>),
    Pinned(Config<KM>),
}

/// Proposal B: hoist `resolved` into the Following variant, so Pinned
/// genuinely has no resolved field to carry.
enum Hoisted<KM: Keymap + Clone> {
    Following {
        effective: Config<KM>,
        resolved: u64,
    },
    Pinned(Config<KM>),
}

/// What the struct pays today: a config plus a bool, packed by alignment.
struct Today<KM: Keymap + Clone> {
    effective: Config<KM>,
    has_private_config: bool,
}

#[test]
fn probe() {
    println!("Config<KM>        {}", size_of::<Config<KM>>());
    println!("  Config                  {}", size_of::<Config<KM>>());
    println!("  SyllableContext<KM>     {}", size_of::<SyllableContext<KM>>());
    println!("TwoVariant<KM>           {}", size_of::<TwoVariant<KM>>());
    println!("Hoisted<KM>              {}", size_of::<Hoisted<KM>>());
    println!("cfg + bool (today)       {}", size_of::<Today<KM>>());
    println!("Session<KM> (today)      {}", size_of::<Session<KM>>());
}
