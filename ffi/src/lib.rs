use std::ffi::{c_char, CString};

use vime_engine::{ConfiguredRuleEngine, Engine, KeyEvent};

pub mod convert;
pub mod types;

pub use types::{VimeEngineHandle, VimeInputMethod, VimeKey, VimeKeyEvent, VimeOutput};

use convert::to_vime_output;

#[no_mangle]
pub extern "C" fn vime_create() -> *mut VimeEngineHandle {
    Box::into_raw(Box::new(VimeEngineHandle {
        engine: Engine::default(),
    }))
}

#[no_mangle]
pub unsafe extern "C" fn vime_destroy(engine: *mut VimeEngineHandle) {
    if !engine.is_null() {
        drop(Box::from_raw(engine));
    }
}

#[no_mangle]
pub unsafe extern "C" fn vime_reset(engine: *mut VimeEngineHandle) -> VimeOutput {
    let Some(engine) = engine.as_mut() else {
        return VimeOutput::empty();
    };
    let result = engine.engine.reset();
    to_vime_output(&engine.engine, result)
}

#[no_mangle]
pub unsafe extern "C" fn vime_process_key(
    engine: *mut VimeEngineHandle,
    event: VimeKeyEvent,
) -> VimeOutput {
    let Some(engine) = engine.as_mut() else {
        return VimeOutput::empty();
    };

    let Ok(key_event) = KeyEvent::try_from(event) else {
        return VimeOutput::empty();
    };

    let result = engine.engine.process_key(key_event);
    to_vime_output(&engine.engine, result)
}

#[no_mangle]
pub unsafe extern "C" fn vime_set_input_method(
    engine: *mut VimeEngineHandle,
    method: VimeInputMethod,
) {
    if let Some(engine) = engine.as_mut() {
        match method {
            VimeInputMethod::Telex => {
                engine.engine.set_layout(ConfiguredRuleEngine::telex());
            }
            VimeInputMethod::Vni => {
                engine.engine.set_layout(ConfiguredRuleEngine::vni());
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn vime_free_string(value: *mut c_char) {
    if !value.is_null() {
        drop(CString::from_raw(value));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;

    #[test]
    fn test_lifecycle_and_telex_typing() {
        unsafe {
            let handle = vime_create();
            assert!(!handle.is_null());
            vime_set_input_method(handle, VimeInputMethod::Telex);

            // Type 'v', 'i', 'e', 'e', 't', 'j' -> "việt"
            let keys = ['v', 'i', 'e', 'e', 't', 'j'];
            let mut last_rendered = String::new();

            for ch in keys {
                let out = vime_process_key(
                    handle,
                    VimeKeyEvent {
                        key: VimeKey::None,
                        character: ch as u32,
                        states: 0,
                    },
                );
                assert!(out.consumed);
                if !out.rendered.is_null() {
                    last_rendered = CStr::from_ptr(out.rendered).to_str().unwrap().to_string();
                    vime_free_string(out.rendered);
                }
            }

            assert_eq!(last_rendered, "việt");

            // Reset
            let reset_out = vime_reset(handle);
            if !reset_out.rendered.is_null() {
                vime_free_string(reset_out.rendered);
            }
            if !reset_out.commit.is_null() {
                vime_free_string(reset_out.commit);
            }

            vime_destroy(handle);
        }
    }
}
