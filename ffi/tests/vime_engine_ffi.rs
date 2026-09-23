use std::ffi::CStr;

use vime::{
    VimeAction, VimeInputMethod, VimeKey, VimeKeyEvent, VimeTonePlacement,
};

#[test]
fn lifecycle_and_telex_typing() {
    unsafe {
        let handle = vime::vime_create();
        assert!(!handle.is_null());
        vime::vime_set_input_method(handle, VimeInputMethod::Telex);

        // Type 'v', 'i', 'e', 'e', 't', 'j' -> "việt"
        let keys = ['v', 'i', 'e', 'e', 't', 'j'];
        let mut last_rendered = String::new();

        for ch in keys {
            let out = vime::vime_process_key(
                handle,
                VimeKeyEvent {
                    key: VimeKey::None,
                    character: ch as u32,
                    states: 0,
                },
            );
            assert_eq!(out.action, VimeAction::UpdatePreedit);
            if !out.rendered.is_null() {
                last_rendered = CStr::from_ptr(out.rendered).to_str().unwrap().to_string();
            }
        }

        assert_eq!(last_rendered, "việt");

        // Explicit commit returns the text and clears the preedit.
        let commit_out = vime::vime_commit(handle);
        assert_eq!(commit_out.action, VimeAction::Commit);
        assert_eq!(
            CStr::from_ptr(commit_out.commit).to_str().unwrap(),
            "việt"
        );

        // Switching input method clears a pending preedit.
        for ch in ['t', 'o', 'o'] {
            let out = vime::vime_process_key(
                handle,
                VimeKeyEvent {
                    key: VimeKey::None,
                    character: ch as u32,
                    states: 0,
                },
            );
            assert_eq!(out.action, VimeAction::UpdatePreedit);
        }
        let switch_out = vime::vime_set_input_method(handle, VimeInputMethod::Vni);
        assert_eq!(switch_out.action, VimeAction::UpdatePreedit);
        assert!(!switch_out.rendered.is_null());
        assert_eq!(
            CStr::from_ptr(switch_out.rendered).to_str().unwrap(),
            ""
        );

        // Reset clears the buffer and updates the preedit (to empty).
        let reset_out = vime::vime_reset(handle);
        assert_eq!(reset_out.action, VimeAction::UpdatePreedit);
        assert!(!reset_out.rendered.is_null());
        assert_eq!(CStr::from_ptr(reset_out.rendered).to_str().unwrap(), "");

        vime::vime_destroy(handle);
    }
}

#[test]
fn create_with_method_and_tone_placement() {
    unsafe {
        // Modern Telex ("hoá").
        let modern = vime::vime_create_with(VimeInputMethod::Telex, VimeTonePlacement::Modern);
        assert!(!modern.is_null());
        for ch in ['h', 'o', 'a', 's'] {
            vime::vime_process_key(
                modern,
                VimeKeyEvent {
                    key: VimeKey::None,
                    character: ch as u32,
                    states: 0,
                },
            );
        }
        let out = vime::vime_commit(modern);
        assert_eq!(out.action, VimeAction::Commit);
        assert_eq!(CStr::from_ptr(out.commit).to_str().unwrap(), "hoá");

        // Old Telex ("hóa") — switch mid-buffer and re-render.
        vime::vime_set_tone_placement(modern, VimeTonePlacement::Old);
        for ch in ['h', 'o', 'a', 's'] {
            vime::vime_process_key(
                modern,
                VimeKeyEvent {
                    key: VimeKey::None,
                    character: ch as u32,
                    states: 0,
                },
            );
        }
        let re_render = vime::vime_set_tone_placement(modern, VimeTonePlacement::Old);
        assert_eq!(re_render.action, VimeAction::UpdatePreedit);
        assert!(!re_render.rendered.is_null());
        assert_eq!(CStr::from_ptr(re_render.rendered).to_str().unwrap(), "hóa");
        let out = vime::vime_commit(modern);
        assert_eq!(CStr::from_ptr(out.commit).to_str().unwrap(), "hóa");

        // VNI engine round-trip.
        let vni = vime::vime_create_with(VimeInputMethod::Vni, VimeTonePlacement::Modern);
        assert!(!vni.is_null());
        for ch in ['h', 'o', 'a', '1'] {
            vime::vime_process_key(
                vni,
                VimeKeyEvent {
                    key: VimeKey::None,
                    character: ch as u32,
                    states: 0,
                },
            );
        }
        let out = vime::vime_commit(vni);
        assert_eq!(CStr::from_ptr(out.commit).to_str().unwrap(), "hoá");

        // VIQR engine round-trip (tone placed after the vowel).
        let viqr = vime::vime_create_with(VimeInputMethod::Viqr, VimeTonePlacement::Modern);
        assert!(!viqr.is_null());
        for ch in ['t', 'o', 'a', '\'', 'i', 'n', 'f'] {
            vime::vime_process_key(
                viqr,
                VimeKeyEvent {
                    key: VimeKey::None,
                    character: ch as u32,
                    states: 0,
                },
            );
        }
        let out = vime::vime_commit(viqr);
        assert_eq!(CStr::from_ptr(out.commit).to_str().unwrap(), "toáinf");

        vime::vime_destroy(modern);
        vime::vime_destroy(vni);
        vime::vime_destroy(viqr);
    }
}