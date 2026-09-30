//! The edges: NULL handles, bad input, and the C string's limits.
//!
//! Every case here is a promise the header makes, or a way a caller can get it
//! wrong. They are grouped in one file because the argument is the same in all of
//! them: a NULL handle must not be a crash, and a rejected call must leave the
//! session exactly as it was.

mod common;

use std::ptr;

use vime::*;

// ---------------------------------------------------------------- null handles

/// Every function that takes a session must tolerate NULL. NULL is what a
/// frontend has when its handle was never created, so the crash it would like to
/// cause is exactly the one it can least afford.
#[test]
fn every_session_call_tolerates_a_null_handle() {
    let null = common::null_session();
    // SAFETY: passing NULL is the point; each function must check before use.
    unsafe {
        assert!(!vime_session_insert(null, b'a' as u32));
        assert!(!vime_session_backspace(null));
        assert!(!vime_session_delete(null));
        assert!(!vime_session_move_cursor_left(null));
        assert!(!vime_session_move_cursor_right(null));
        vime_session_reset(null);
        assert!(vime_session_render_text(null).is_null());
        assert!(vime_session_render_raw_text(null).is_null());
        assert!(vime_session_render_state(null).is_null());
        assert!(vime_session_get_cursor_char_idx(null) == 0);
        assert!(vime_session_get_cursor_byte_idx(null) == 0);
        assert!(vime_session_get_raw_cursor_char_idx(null) == 0);
        assert!(vime_session_get_raw_cursor_byte_idx(null) == 0);
        assert!(!vime_session_is_valid_vietnamese(null));
        assert!(!vime_session_set_config(null, ptr::null()));
        assert!(!vime_session_clear_config(null));
        vime_session_destroy(null);
    }
}

/// The same for the factory.
#[test]
fn every_factory_call_tolerates_a_null_handle() {
    let null = common::null_factory();
    // SAFETY: as above.
    unsafe {
        assert!(!vime_session_factory_set_config(null, ptr::null()));
        assert!(vime_session_create(null).is_null());
        assert!(vime_session_create_with_config(null, ptr::null()).is_null());
        vime_session_factory_destroy(null);
    }
}

/// Destroying NULL is a no-op rather than an error, so a `goto cleanup` path
/// works even when creation failed.
#[test]
fn destroying_a_null_handle_twice_is_harmless() {
    // SAFETY: both calls are documented no-ops.
    unsafe {
        vime_session_destroy(ptr::null_mut());
        vime_session_destroy(ptr::null_mut());
        vime_session_factory_destroy(ptr::null_mut());
        vime_session_factory_destroy(ptr::null_mut());
    }
}

// ----------------------------------------------------------------- bad configs

/// An invalid enum is rejected rather than trusted. A C caller can pass anything,
/// and reading an out-of-range discriminant as a Rust enum is undefined behaviour
/// — so the adapter checks the integer before it ever forms the enum.
#[test]
fn an_out_of_range_enum_is_rejected() {
    for bad in [0u32, 4, 5, 100, u32::MAX, 0xFFFF_FFFE] {
        let bytes = common::config_with_raw_input_method(bad);
        // SAFETY: `bytes` is 12 correctly aligned bytes holding a plausible
        // `VimeConfig` with an out-of-range discriminant, which is what a C caller
        // can pass. The adapter must validate the integer before it forms the
        // enum, so this is exactly the input it is meant to survive.
        unsafe {
            let factory = vime_session_factory_create_with_config(bytes.as_ptr().cast());
            assert!(factory.is_null(), "input_method = {bad} should be rejected");
            if !factory.is_null() {
                vime_session_factory_destroy(factory);
            }
        }
    }
}

/// The same for tone placement, which has its own range.
#[test]
fn an_out_of_range_tone_placement_is_rejected() {
    for bad in [0u32, 3, 99, u32::MAX] {
        let bytes = common::config_with_raw_tone_placement(bad);
        // SAFETY: see above.
        unsafe {
            let factory = vime_session_factory_create_with_config(bytes.as_ptr().cast());
            assert!(
                factory.is_null(),
                "tone_placement = {bad} should be rejected"
            );
            if !factory.is_null() {
                vime_session_factory_destroy(factory);
            }
        }
    }
}

/// A bad config is also rejected by the two setters, and — importantly — a
/// rejected set leaves the previous config in force.
#[test]
fn a_rejected_config_leaves_the_old_one_in_place() {
    let mut factory = common::Factory::create().unwrap();
    let mut session = factory.open_session();

    let good = VimeConfig {
        tone_placement: VimeTonePlacement::Old,
        ..Default::default()
    };
    assert!(factory.set_config(&good));

    let bad = common::config_with_raw_tone_placement(0);
    let bad = bad.as_slice();
    // SAFETY: an out-of-range tone placement, which the setters must reject.
    unsafe {
        assert!(
            !vime_session_factory_set_config(factory.handle(), bad.as_ptr().cast()),
            "0 is not a tone placement"
        );
        assert!(
            !vime_session_set_config(session.handle(), bad.as_ptr().cast()),
            "0 is not a tone placement"
        );
    }

    assert_eq!(
        session.type_text("hoas"),
        "hóa",
        "the Old placement set earlier still holds"
    );
}

/// NULL where a config is required is refused by the setters, which have no
/// default to fall back on.
#[test]
fn a_null_config_is_refused_by_the_setters() {
    let mut factory = common::Factory::create().unwrap();
    let mut session = factory.open_session();
    // SAFETY: NULL is what is being tested; both calls must check first.
    unsafe {
        assert!(!vime_session_factory_set_config(
            factory.handle(),
            ptr::null()
        ));
        assert!(!vime_session_set_config(session.handle(), ptr::null()));
    }
    // Still usable afterwards.
    assert_eq!(session.type_text("toan"), "toan");
}

/// A NULL config on a *constructor* means "use the default", because that is what
/// the separate no-config constructor already does and the header says so.
#[test]
fn a_null_config_on_a_constructor_means_default() {
    let mut plain_factory = common::Factory::create().unwrap();
    // SAFETY: NULL config is the documented way to ask for the default.
    let from_null = unsafe {
        let f = vime_session_factory_create_with_config(ptr::null());
        assert!(!f.is_null());
        let s = vime_session_create_with_config(f, ptr::null());
        assert!(!s.is_null());
        let text = vime_session_render_text(s);
        let owned = common::read_str(text);
        vime_session_destroy(s);
        vime_session_factory_destroy(f);
        owned
    };
    assert_eq!(from_null, "");

    let mut plain_session = plain_factory.open_session();
    assert_eq!(from_null, plain_session.render_text());
}

// ------------------------------------------------------------------ bad input

/// A `uint32_t` that is not a Unicode scalar value is refused. The engine takes
/// `char`, and `char::from_u32` returning `None` is the check; the alternative —
/// truncating to a byte or wrapping — would silently corrupt the buffer.
#[test]
fn a_value_that_is_not_a_character_is_refused() {
    let mut session = common::Factory::create().unwrap().open_session();
    assert_eq!(session.type_text("toan"), "toan");

    for bad in [
        0xD800u32,   // high surrogate
        0xDBFF,      // high surrogate
        0xDC00,      // low surrogate
        0xDFFF,      // low surrogate
        0x0011_0000, // past the last scalar value
        u32::MAX,
    ] {
        assert!(!session.insert_raw(bad), "{bad:#x} is not a character");
    }
    assert_eq!(session.render_text(), "toan", "the buffer is untouched");
    assert_eq!(session.render_raw_text(), "toan");
}

/// Control characters are valid scalars and are passed through. The engine
/// decides what to do with them; the adapter's only job is to not lose them.
#[test]
fn control_characters_are_accepted() {
    let mut session = common::Factory::create().unwrap().open_session();
    for ch in ['\u{7}', '\u{1b}', '\t'] {
        assert!(session.insert(ch), "{ch:?}");
    }
    assert_eq!(session.render_raw_text(), "\u{7}\u{1b}\t");
}

// -------------------------------------------------------------- the c contract

/// The returned pointer stays valid until the next call that can change the text.
/// Reading it twice in a row, with no edit between, must be safe — which is why
/// the handle keeps the buffer rather than building a temporary.
#[test]
fn the_returned_string_outlives_the_next_query() {
    let mut session = common::Factory::create().unwrap().open_session();
    session.type_text("tieengs");

    // SAFETY: the pointer is owned by the live session and is not invalidated by
    // another query.
    unsafe {
        let first = vime_session_render_text(session.handle());
        let first = std::ffi::CStr::from_ptr(first).to_owned();
        let second = vime_session_render_text(session.handle());
        let second = std::ffi::CStr::from_ptr(second).to_owned();
        assert_eq!(first, second);
        assert_eq!(common::read_str(first.as_ptr()), "tiếng");

        // Interleaving the other query must not disturb the text pointer.
        let raw = vime_session_render_raw_text(session.handle());
        assert_eq!(std::ffi::CStr::from_ptr(raw).to_str().unwrap(), "tieengs");
        let again = vime_session_render_text(session.handle());
        assert_eq!(std::ffi::CStr::from_ptr(again).to_str().unwrap(), "tiếng");
    }
}

/// The snapshot's strings are the same buffers the single getters return, so one
/// query filling both is not two renders' worth of work, and they cannot disagree.
#[test]
fn the_snapshot_matches_the_individual_getters() {
    let mut session = common::Factory::create().unwrap().open_session();
    session.type_text("tieengs");
    session.move_cursor_left();

    let state = session.render_state();
    assert_eq!(state.text, session.render_text());
    assert_eq!(state.raw_text, session.render_raw_text());
    assert_eq!(state.cursor_char_idx, session.cursor_char_idx());
    assert_eq!(state.cursor_byte_idx, session.cursor_byte_idx());
    assert_eq!(state.raw_cursor_char_idx, session.raw_cursor_char_idx());
    assert_eq!(state.raw_cursor_byte_idx, session.raw_cursor_byte_idx());
    assert_eq!(state.is_valid_vietnamese, session.is_valid_vietnamese());
}

/// `bytes_to_delete` is what the host must erase before it writes the new word,
/// so it is the length of the *previous* snapshot's text, not the current one.
/// This is the one field with a dependency on call history, so it gets its own
/// test.
#[test]
fn bytes_to_delete_is_the_previously_delivered_length() {
    let mut session = common::Factory::create().unwrap().open_session();
    assert_eq!(session.render_state(), common::Snapshot::empty());

    // Type a word, then read the snapshot: nothing has been delivered yet, so
    // there is nothing to erase.
    session.type_text("toan");
    let first = session.render_state();
    assert_eq!(first.text, "toan");
    assert_eq!(
        first.bytes_to_delete, 0,
        "nothing was delivered before this"
    );
    assert_eq!(first.chars_to_delete, 0);

    // Edit, then read again: now the host is being told to erase the 4 bytes it
    // was last handed.
    session.insert('g');
    let second = session.render_state();
    assert_eq!(second.text, "toang");
    assert_eq!(second.bytes_to_delete, 4, "the 'toan' from last time");
    assert_eq!(second.chars_to_delete, 4);

    // Reading again moves the request forward: the previous call reported
    // "toang", so that is now what the host has on screen and what it is being
    // told to erase. The header says "bytes to remove from previous render", and
    // the previous render is this call's predecessor, not the text before the
    // edit.
    let third = session.render_state();
    assert_eq!(third.text, "toang");
    assert_eq!(
        third.bytes_to_delete, 5,
        "the 'toang' the last call reported"
    );
    assert_eq!(third.chars_to_delete, 5);

    // The length is in bytes even when the text is not ASCII.
    session.type_text("dduongf");
    let _ = session.render_state();
    let state = session.render_state();
    assert_eq!(state.bytes_to_delete, "đùong".len());
    assert_eq!(state.chars_to_delete, "đùong".chars().count());
    assert!(state.bytes_to_delete > state.chars_to_delete);
}

/// `render_text` on its own does not move the erase request: the host may be
/// reading the text for a reason other than painting it, and an unacknowledged
/// snapshot must not be forgotten.
#[test]
fn reading_the_text_does_not_consume_the_erase_request() {
    let mut session = common::Factory::create().unwrap().open_session();
    session.type_text("toan");
    let _ = session.render_state();

    session.insert('g');
    assert_eq!(session.render_text(), "toang");
    assert_eq!(session.render_text(), "toang", "and again");

    let state = session.render_state();
    assert_eq!(
        state.bytes_to_delete, 4,
        "still the 'toan' that was delivered"
    );
}

/// An empty word reports zero lengths rather than something a caller has to
/// special-case.
#[test]
fn an_empty_word_has_no_bytes_to_delete() {
    let mut session = common::Factory::create().unwrap().open_session();
    session.type_text("toan");
    let _ = session.render_state();
    session.reset();

    let state = session.render_state();
    assert_eq!(state.text, "");
    assert_eq!(
        state.bytes_to_delete, 4,
        "the host still has 'toan' on screen"
    );
    assert_eq!(state.chars_to_delete, 4);
    assert_eq!(state.cursor_byte_idx, 0);
    assert_eq!(state.cursor_char_idx, 0);
    assert!(!state.is_valid_vietnamese, "an empty buffer is not a word");
}
