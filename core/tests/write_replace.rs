//! The `write_*_to` family replaces `output` rather than appending to it, so a
//! frontend may reuse one buffer across keystrokes without clearing it itself.
//! These tests pin that contract down and check that reuse stays allocation-free.

use vime_engine::{DefaultKeymap, Session, SessionFactory, Settings};

fn session() -> Session<DefaultKeymap<'static>> {
    SessionFactory::telex(Settings::default()).new_session()
}

#[test]
fn write_rendered_replaces_rather_than_accumulates() {
    let mut session = session();
    for ch in "tan".chars() {
        session.insert(ch);
    }

    let mut out = String::new();
    for _ in 0..5 {
        session.write_rendered_to(&mut out);
        assert_eq!(out, "tan", "must replace, not accumulate");
    }
}

#[test]
fn write_raw_replaces_rather_than_accumulates() {
    let mut session = session();
    for ch in "hoas".chars() {
        session.insert(ch);
    }

    let mut out = String::new();
    for _ in 0..5 {
        session.write_raw_to(&mut out);
        assert_eq!(out, "hoas", "must replace, not accumulate");
    }
}

#[test]
fn overwriting_a_shorter_word_leaves_no_stale_tail() {
    let mut session = session();
    for ch in "nguyen".chars() {
        session.insert(ch);
    }

    let mut out = String::new();
    session.write_rendered_to(&mut out);
    assert!(!out.is_empty());

    for _ in 0..5 {
        session.backspace();
    }
    session.write_rendered_to(&mut out);
    assert_eq!(out, "n", "the longer word's tail must be gone");
}

#[test]
fn reusing_one_buffer_never_reallocates() {
    let mut session = session();
    for ch in "hoas".chars() {
        session.insert(ch);
    }

    let mut out = String::new();
    session.write_rendered_to(&mut out);
    let capacity = out.capacity();
    let ptr = out.as_ptr();

    for _ in 0..10_000 {
        session.write_rendered_to(&mut out);
    }

    assert_eq!(out.capacity(), capacity, "capacity must be retained");
    assert_eq!(out.as_ptr(), ptr, "stable address means no reallocation");
    assert_eq!(out, "hoá");
}
