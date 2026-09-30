//! Typing, editing, and cursor movement through the C ABI.

mod common;

/// The headline case: Telex turns raw ASCII into Vietnamese, and the ABI reports
/// the same render the core does.
#[test]
fn telex_turns_ascii_into_vietnamese() {
    let mut session = common::Factory::create().unwrap().open_session();
    assert_eq!(session.type_text("tieengs"), "tiếng");
    assert_eq!(session.render_text(), "tiếng");
}

/// Every tone, from the same vowel, to catch a regression in tone placement.
#[test]
fn every_tone_lands_on_the_right_vowel() {
    let mut session = common::Factory::create().unwrap().open_session();
    for (typed, expected) in [
        ("hoa", "hoa"),
        ("hoas", "hoá"),
        ("hoaf", "hoà"),
        ("hoar", "hoả"),
        ("hoax", "hoã"),
        ("hoaj", "hoạ"),
    ] {
        assert_eq!(session.type_text(typed), expected, "typing {typed:?}");
    }
}

/// The other two input methods the header exposes are selected purely by config.
#[test]
fn other_input_methods_come_from_config() {
    // Each method spells the same word with its own tone keys: Telex `s`, VNI
    // `1`, VIQR `'`. All three must render identically, which is what shows the
    // keymap came from the config rather than being hard-wired.
    let cases = [
        (vime::VimeInputMethod::Telex, "toans"),
        (vime::VimeInputMethod::Vni, "toan1"),
        (vime::VimeInputMethod::Viqr, "toan'"),
    ];
    for (input_method, typed) in cases {
        let config = vime::VimeConfig {
            input_method,
            ..Default::default()
        };
        let mut factory = common::Factory::create_with(&config).unwrap();
        let mut session = factory.open_session();
        assert_eq!(session.type_text(typed), "toán", "{input_method:?}");
        assert_eq!(session.render_raw_text(), typed);
    }
}

/// The shape and stroke keys differ per method too: Telex and VIQR use `o` for
/// the horn, VNI uses `7`, and Telex and VNI use `d`/`9` for the crossed `d`.
#[test]
fn shape_keys_come_from_the_config_too() {
    let cases = [
        (vime::VimeInputMethod::Telex, "ow", "ơ"),
        (vime::VimeInputMethod::Vni, "o7", "ơ"),
        (vime::VimeInputMethod::Viqr, "o+", "ơ"),
    ];
    for (input_method, typed, expected) in cases {
        let config = vime::VimeConfig {
            input_method,
            ..Default::default()
        };
        let mut factory = common::Factory::create_with(&config).unwrap();
        let mut session = factory.open_session();
        assert_eq!(session.type_text(typed), expected, "{input_method:?}");
    }
}

/// Tone placement is a config choice with a visible effect: `hoa` sắc'd goes to
/// the second vowel under Modern and the first under Old.
#[test]
fn tone_placement_is_configurable() {
    let config = vime::VimeConfig {
        tone_placement: vime::VimeTonePlacement::Old,
        ..Default::default()
    };
    let mut factory = common::Factory::create_with(&config).unwrap();
    let mut session = factory.open_session();
    assert_eq!(session.type_text("hoas"), "hóa");
}

/// `raw_text` is the keystrokes as typed, which is what a frontend needs to
/// decide whether it should re-interpret the word.
#[test]
fn raw_text_keeps_the_keystrokes() {
    let mut session = common::Factory::create().unwrap().open_session();
    session.type_text("tieengs");
    assert_eq!(session.render_raw_text(), "tieengs");
}

/// Backspace removes one grapheme cluster, not one code point and not one
/// keystroke. `tiếng` is five code points but the `ế` is an `e` plus a combining
/// tone mark, so a backspace takes the mark with its base letter — which is what
/// a caller has to mean by "one character".
#[test]
fn backspace_removes_a_whole_grapheme() {
    let mut session = common::Factory::create().unwrap().open_session();
    session.type_text("tieengs");
    assert_eq!(session.render_text(), "tiếng");
    assert_eq!(session.render_text().chars().count(), 5);

    assert!(session.backspace());
    assert_eq!(session.render_text(), "tiến");
    assert_eq!(session.render_raw_text(), "tieeng");

    // Keep going. The rendered word empties before the raw one does, because a
    // keystroke can collapse into a character that has already been removed —
    // which is why the return value is "did anything change", not a count.
    for expected in ["tiế", "ti", "t", ""] {
        session.backspace();
        assert_eq!(session.render_text(), expected);
    }
    assert_eq!(session.render_text(), "");

    // Drain the rest. The call reports false only once both buffers are empty.
    let mut guard = 0;
    while session.backspace() {
        guard += 1;
        assert!(guard < 10, "backspace should stop long before this");
    }
    assert_eq!(session.render_text(), "");
    assert_eq!(session.render_raw_text(), "");
}

/// `delete` removes forward, which only means anything once the caret is off the
/// end. At the end of the word there is nothing forward to remove.
#[test]
fn delete_removes_forward() {
    let mut session = common::Factory::create().unwrap().open_session();
    session.type_text("toan");
    assert_eq!(session.render_text(), "toan");
    assert!(!session.delete(), "the caret is at the end");

    assert!(session.move_cursor_left());
    assert_eq!(session.cursor_char_idx(), 3);
    assert!(session.delete());
    assert_eq!(session.render_text(), "toa");
}

/// Cursor indices are reported in both units, and the two only diverge for a word
/// with multi-byte characters. `dduongf` renders `đuong` with a tone — five code
/// points, but six bytes, because `đ` and the toned vowel are two bytes each.
#[test]
fn cursor_indices_are_in_both_bytes_and_characters() {
    let mut session = common::Factory::create().unwrap().open_session();
    session.type_text("dduongf");
    assert_eq!(session.render_text(), "đùong");

    // Walk the caret from the end back to the start, checking the two units
    // track each other.
    let text = "đùong";
    let expected_bytes: Vec<usize> = {
        let mut acc = 0;
        let mut v = vec![acc];
        for ch in text.chars() {
            acc += ch.len_utf8();
            v.push(acc);
        }
        v
    };
    assert_eq!(expected_bytes.len(), text.chars().count() + 1);

    // The two units really are different numbers, or this test proves nothing.
    assert!(text.len() > text.chars().count());

    for chars in (0..=text.chars().count()).rev() {
        assert_eq!(session.cursor_char_idx(), chars, "at {chars} characters");
        assert_eq!(
            session.cursor_byte_idx(),
            expected_bytes[chars],
            "at {chars} chars"
        );
        if chars > 0 {
            assert!(session.move_cursor_left());
        }
    }
    assert_eq!(session.cursor_char_idx(), 0);
    assert_eq!(session.cursor_byte_idx(), 0);

    // And back to the end again, to show the pair is not one-way.
    while session.move_cursor_right() {}
    assert_eq!(session.cursor_char_idx(), text.chars().count());
    assert_eq!(session.cursor_byte_idx(), text.len());
}

/// The raw cursor and the rendered cursor are independent: they are the same
/// caret over two different strings.
#[test]
fn the_raw_cursor_is_reported_separately() {
    let mut session = common::Factory::create().unwrap().open_session();
    session.type_text("tieengs");
    assert_eq!(session.render_text(), "tiếng");
    assert_eq!(session.render_raw_text(), "tieengs");

    // Both start at the end of their own string.
    assert_eq!(session.cursor_char_idx(), 5);
    assert_eq!(session.raw_cursor_char_idx(), 7);

    // Moving the rendered caret does not invent a raw position; the raw caret is
    // what the core tracks and the ABI only reports it.
    assert!(session.move_cursor_left());
    assert_eq!(session.cursor_char_idx(), 4);
}

/// Validity is the frontend's cue to stop treating the buffer as a draft.
#[test]
fn validity_tracks_whether_the_word_is_vietnamese() {
    let mut session = common::Factory::create().unwrap().open_session();
    session.type_text("tieengs");
    assert!(session.is_valid_vietnamese());
    assert!(session.render_state().is_valid_vietnamese);

    session.type_text("xyz");
    assert!(!session.is_valid_vietnamese());

    // A real word that is not valid Vietnamese either.
    session.type_text("hello");
    assert!(!session.is_valid_vietnamese());
}

/// Reset clears the buffer but keeps the session, and its config, alive.
#[test]
fn reset_clears_the_buffer() {
    let mut session = common::Factory::create().unwrap().open_session();
    session.type_text("tieengs");
    session.reset();
    assert_eq!(session.render_text(), "");
    assert_eq!(session.render_raw_text(), "");
    assert_eq!(session.cursor_char_idx(), 0);

    // Still usable.
    assert_eq!(session.type_text("toan"), "toan");
}

/// An empty session reports an empty render, not NULL — the header says the
/// return is a valid string, and a caller should not have to special-case it.
#[test]
fn an_empty_session_renders_an_empty_string() {
    let mut session = common::Factory::create().unwrap().open_session();
    assert_eq!(session.render_text(), "");
    assert_eq!(session.render_raw_text(), "");
    assert_eq!(session.render_text_ptr(), Some(String::new()));
    assert_eq!(common::Snapshot::empty(), session.render_state());
}

/// The `uint32_t` character argument is a Unicode scalar value, not a byte and
/// not a UTF-8 sequence. A precomposed Vietnamese character has to arrive intact.
#[test]
fn a_non_ascii_scalar_survives_the_boundary() {
    let mut session = common::Factory::create().unwrap().open_session();
    for ch in ['ạ', 'ế', 'ữ', '\u{1f600}'] {
        assert!(session.insert(ch), "insert({ch:?})");
    }
    assert_eq!(session.render_raw_text(), "ạếữ\u{1f600}");

    // A surrogate half and a value past U+10FFFF are refused, not truncated and
    // not wrapped, and a rejected insert leaves the buffer alone.
    assert!(!session.insert_raw(0xD800), "a lone high surrogate");
    assert!(!session.insert_raw(0xDFFF), "a lone low surrogate");
    assert!(!session.insert_raw(0x0011_0000), "past U+10FFFF");
    assert!(!session.insert_raw(u32::MAX));
    assert_eq!(session.render_raw_text(), "ạếữ\u{1f600}");

    // NUL is a valid scalar value, so the engine accepts it and keeps it. The C
    // string cannot show it, though: a `const char *` ends at the first NUL, so
    // the caller sees the text up to that point. The engine's own copy is
    // unaffected, which is what the header's contract implies by handing out a C
    // string at all.
    assert!(session.insert('\0'));
    assert_eq!(
        session.render_raw_text(),
        "ạếữ\u{1f600}",
        "the C view stops at the NUL"
    );
}
