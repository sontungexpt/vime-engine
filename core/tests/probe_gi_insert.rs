use vime_engine::DefaultKeymap;
use vime_engine::{composition::syllable::BuildingSyllable, phonology::TonePlacement};

fn chars(s: &BuildingSyllable) -> String {
    s.to_chars(TonePlacement::Modern).iter().collect()
}

#[test]
fn insert_vowel_after_gi_appends_the_vowel() {
    let km = DefaultKeymap::telex();
    let mut s = BuildingSyllable::default();

    // "gin": `g` onset, `i` nucleus, `n` coda.
    for c in ['g', 'i', 'n'] {
        assert!(s.push(&km, c).is_ok());
    }
    assert_eq!(chars(&s), "gin");

    // Insert `a` at the vowel boundary (between `i` and `n`): "gian".
    assert!(s.insert(&km, 2, 'a').is_ok());
    assert_eq!(chars(&s), "gian");
}

#[test]
fn push_and_insert_build_gian_the_same_way() {
    let km = DefaultKeymap::telex();

    let mut via_push = BuildingSyllable::default();
    for c in ['g', 'i', 'a', 'n'] {
        assert!(via_push.push(&km, c).is_ok());
    }

    let mut via_insert = BuildingSyllable::default();
    for c in ['g', 'i', 'n'] {
        assert!(via_insert.push(&km, c).is_ok());
    }
    assert!(via_insert.insert(&km, 2, 'a').is_ok());

    assert_eq!(chars(&via_push), chars(&via_insert));
}
