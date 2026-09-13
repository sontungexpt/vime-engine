#[cfg(test)]
use std::ffi::CStr;

use vime_engine::{ConfiguredRuleEngine, Engine, KeyEvent};

pub mod convert;
pub mod types;

pub use types::{
    VimeAction, VimeEngineHandle, VimeInputMethod, VimeKey, VimeKeyEvent, VimeOutput,
};

use convert::KeyEventConversionError;

#[no_mangle]
pub extern "C" fn vime_create() -> *mut VimeEngineHandle {
    Box::into_raw(Box::new(VimeEngineHandle {
        engine: Engine::default(),
        rendered: None,
        commit: None,
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
        return VimeOutput::default();
    };
    let result = engine.engine.reset();
    engine.output(result)
}

#[no_mangle]
pub unsafe extern "C" fn vime_process_key(
    engine: *mut VimeEngineHandle,
    event: VimeKeyEvent,
) -> VimeOutput {
    let Some(engine) = engine.as_mut() else {
        return VimeOutput::default();
    };

    let key_event: Result<KeyEvent, KeyEventConversionError> = event.try_into();
    let Ok(key_event) = key_event else {
        return VimeOutput::default();
    };

    let result = engine.engine.process_key(key_event);
    engine.output(result)
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

#[cfg(test)]
mod tests {
    use super::*;

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
                assert_eq!(out.action, VimeAction::UpdatePreedit);
                if !out.rendered.is_null() {
                    last_rendered = CStr::from_ptr(out.rendered).to_str().unwrap().to_string();
                }
            }

            assert_eq!(last_rendered, "việt");

            // Reset clears the buffer and updates the preedit (to empty).
            let reset_out = vime_reset(handle);
            assert_eq!(reset_out.action, VimeAction::UpdatePreedit);
            assert!(!reset_out.rendered.is_null());
            assert_eq!(CStr::from_ptr(reset_out.rendered).to_str().unwrap(), "");

            vime_destroy(handle);
        }
    }
}