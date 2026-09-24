use std::ptr;

use vime_engine::phonology::TonePlacement;
use vime_engine::{Config, DefaultKeymap, Engine, KeyEvent};

pub mod convert;
pub mod types;

pub use types::{
    VimeAction, VimeEngineHandle, VimeInputMethod, VimeKey, VimeKeyEvent, VimeOutput,
    VimeTonePlacement,
};

use convert::KeyEventConversionError;

#[no_mangle]
pub extern "C" fn vime_create() -> *mut VimeEngineHandle {
    VimeEngineHandle::new(Engine::telex(Config::default())).into_raw()
}

/// Creates an engine for any built-in input method with the given
/// tone-placement scheme. Returns NULL for an unknown input method.
#[no_mangle]
pub extern "C" fn vime_create_with(
    method: VimeInputMethod,
    tone_placement: VimeTonePlacement,
) -> *mut VimeEngineHandle {
    let engine = match method {
        VimeInputMethod::Telex => Engine::with_tone_placement(
            Config::default(),
            DefaultKeymap::telex(),
            tone_placement.into(),
        ),
        VimeInputMethod::Vni => Engine::with_tone_placement(
            Config::default(),
            DefaultKeymap::vni(),
            tone_placement.into(),
        ),
        VimeInputMethod::Viqr => Engine::with_tone_placement(
            Config::default(),
            DefaultKeymap::viqr(),
            tone_placement.into(),
        ),
        #[allow(unreachable_patterns)]
        _ => return ptr::null_mut(),
    };
    VimeEngineHandle::new(engine).into_raw()
}

impl VimeEngineHandle {
    fn into_raw(self) -> *mut VimeEngineHandle {
        Box::into_raw(Box::new(self))
    }
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
pub unsafe extern "C" fn vime_commit(engine: *mut VimeEngineHandle) -> VimeOutput {
    let Some(engine) = engine.as_mut() else {
        return VimeOutput::default();
    };
    let result = engine.engine.commit();
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
) -> VimeOutput {
    let Some(engine) = engine.as_mut() else {
        return VimeOutput::default();
    };

    #[allow(unreachable_patterns)]
    match method {
        VimeInputMethod::Telex => engine.engine.set_keymap(DefaultKeymap::telex()),
        VimeInputMethod::Vni => engine.engine.set_keymap(DefaultKeymap::vni()),
        VimeInputMethod::Viqr => engine.engine.set_keymap(DefaultKeymap::viqr()),
        _ => return VimeOutput::default(),
    }

    // `set_keymap` resets the buffer; surface the resulting (empty) preedit.
    let result = engine.engine.reset();
    engine.output(result)
}

/// Switches the tone-placement scheme, re-rendering the current preedit.
#[no_mangle]
pub unsafe extern "C" fn vime_set_tone_placement(
    engine: *mut VimeEngineHandle,
    tone_placement: VimeTonePlacement,
) -> VimeOutput {
    let Some(engine) = engine.as_mut() else {
        return VimeOutput::default();
    };

    #[allow(unreachable_patterns)]
    let tone_placement = match tone_placement {
        VimeTonePlacement::Modern => TonePlacement::Modern,
        VimeTonePlacement::Old => TonePlacement::Old,
        _ => return VimeOutput::default(),
    };

    engine.engine.set_tone_placement(tone_placement);
    engine.output(vime_engine::Result::Changed)
}
