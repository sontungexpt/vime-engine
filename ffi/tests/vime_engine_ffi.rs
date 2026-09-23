use std::ffi::CStr;
use std::ptr;

use vime::{VimeAction, VimeInputMethod, VimeKey, VimeKeyEvent, VimeTonePlacement};

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
        assert_eq!(CStr::from_ptr(commit_out.commit).to_str().unwrap(), "việt");

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
        assert_eq!(CStr::from_ptr(switch_out.rendered).to_str().unwrap(), "");

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
        let old = vime::vime_create_with(VimeInputMethod::Telex, VimeTonePlacement::Old);
        assert!(!old.is_null());
        for ch in ['h', 'o', 'a', 's'] {
            vime::vime_process_key(
                old,
                VimeKeyEvent {
                    key: VimeKey::None,
                    character: ch as u32,
                    states: 0,
                },
            );
        }
        let out = vime::vime_commit(old);
        assert_eq!(out.action, VimeAction::Commit);
        assert_eq!(CStr::from_ptr(out.commit).to_str().unwrap(), "hóa");

        // Switch tone placement mid-buffer and re-render.
        let dynamic = vime::vime_create_with(VimeInputMethod::Telex, VimeTonePlacement::Modern);
        for ch in ['h', 'o', 'a', 's'] {
            vime::vime_process_key(
                dynamic,
                VimeKeyEvent {
                    key: VimeKey::None,
                    character: ch as u32,
                    states: 0,
                },
            );
        }
        let re_render = vime::vime_set_tone_placement(dynamic, VimeTonePlacement::Old);
        assert_eq!(re_render.action, VimeAction::UpdatePreedit);
        assert!(!re_render.rendered.is_null());
        assert_eq!(CStr::from_ptr(re_render.rendered).to_str().unwrap(), "hóa");
        let out = vime::vime_commit(dynamic);
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

        // VIQR engine round-trip.
        let viqr = vime::vime_create_with(VimeInputMethod::Viqr, VimeTonePlacement::Modern);
        assert!(!viqr.is_null());
        for ch in ['t', 'o', 'a', '\'', 'n'] {
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
        assert_eq!(CStr::from_ptr(out.commit).to_str().unwrap(), "toán");

        vime::vime_destroy(modern);
        vime::vime_destroy(old);
        vime::vime_destroy(dynamic);
        vime::vime_destroy(vni);
        vime::vime_destroy(viqr);
    }
}

#[test]
fn special_keys_backspace_and_navigation() {
    unsafe {
        let handle = vime::vime_create();
        assert!(!handle.is_null());

        // Type 'v', 'i', 'e', 't'
        for ch in ['v', 'i', 'e', 't'] {
            vime::vime_process_key(
                handle,
                VimeKeyEvent {
                    key: VimeKey::None,
                    character: ch as u32,
                    states: 0,
                },
            );
        }

        // Backspace removes 't' -> "vie"
        let out = vime::vime_process_key(
            handle,
            VimeKeyEvent {
                key: VimeKey::Backspace,
                character: 0,
                states: 0,
            },
        );
        assert_eq!(out.action, VimeAction::UpdatePreedit);
        assert_eq!(CStr::from_ptr(out.rendered).to_str().unwrap(), "vie");

        vime::vime_destroy(handle);
    }
}

#[test]
fn null_handle_safety() {
    unsafe {
        vime::vime_destroy(ptr::null_mut());

        let reset_out = vime::vime_reset(ptr::null_mut());
        assert_eq!(reset_out.action, VimeAction::Forward);
        assert!(reset_out.rendered.is_null());
        assert!(reset_out.commit.is_null());

        let commit_out = vime::vime_commit(ptr::null_mut());
        assert_eq!(commit_out.action, VimeAction::Forward);

        let key_out = vime::vime_process_key(
            ptr::null_mut(),
            VimeKeyEvent {
                key: VimeKey::None,
                character: 'a' as u32,
                states: 0,
            },
        );
        assert_eq!(key_out.action, VimeAction::Forward);

        let im_out = vime::vime_set_input_method(ptr::null_mut(), VimeInputMethod::Telex);
        assert_eq!(im_out.action, VimeAction::Forward);

        let tone_out = vime::vime_set_tone_placement(ptr::null_mut(), VimeTonePlacement::Modern);
        assert_eq!(tone_out.action, VimeAction::Forward);
    }
}
