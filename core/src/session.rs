//! Sessions, and the shared settings they follow.
//!
//! An [`Engine`](crate::Engine) owns one [`SharedConfig`] and mints [`Session`]s
//! from it. A session is one typing buffer, several can exist at once, and by
//! default all of them run under the same settings. A session that needs its
//! own settings takes a private [`SessionConfig`] instead, and then stops
//! following the shared one.
//!
//! # How a change reaches every session
//!
//! The shared config is replaced in place behind an atomic generation counter
//! rather than pushed to each session, so a session cannot miss an update
//! because nobody remembered to notify it. Every session caches the settings it
//! resolved together with the generation it resolved them at;
//! [`Session::refresh`] compares that against the current generation and is
//! called for you by [`Session::process_key`]. In the steady state that costs
//! one atomic load, with no lock and no allocation.
//!
//! The counter is not only a cache check. Changing the shared settings can move
//! the rendered word even when no key was pressed, so a session has to be able
//! to notice that it happened: [`Session::refresh`] reports it, and
//! `process_key` turns it into a [`Result::Changed`] so the frontend re-reads
//! the word.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use crate::composition::Composition;
use crate::composition::syllable::{SyllableBuilder, SyllableContext};
use crate::config::Config;
use crate::event::{Key, KeyEvent};
use crate::keymap::Keymap;
use crate::phonology::TonePlacement;
use crate::result::Result;

const SUFFIX_SPACE: &str = " ";

/// The settings one session parses and renders under.
///
/// This is the engine [`Config`] together with the [`SyllableContext`] that
/// supplies the keymap and the tone-placement scheme. A session either shares
/// one of these with every other session or holds its own, which is why the
/// private form is a whole `SessionConfig` rather than a patch: a partial
/// override is the kind of thing that can end up half-applied.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SessionConfig<KM: Keymap> {
    /// Engine-level settings.
    pub config: Config,
    /// The parse and render context.
    pub context: SyllableContext<KM>,
}

impl<KM: Keymap> SessionConfig<KM> {
    /// Pairs an engine [`Config`] with a parse context.
    #[inline]
    pub fn new(config: Config, context: SyllableContext<KM>) -> Self {
        Self { config, context }
    }

    /// Pairs an engine [`Config`] with a keymap, using the modern
    /// tone-placement convention.
    #[inline]
    pub fn from_keymap(config: Config, keymap: KM) -> Self {
        Self::new(config, SyllableContext::new(keymap, TonePlacement::Modern))
    }
}

/// Settings shared by every session that has not taken a private config.
///
/// Cloning is a refcount bump, so each session can hold one and still observe a
/// later [`SharedConfig::replace`].
pub struct SharedConfig<KM: Keymap> {
    inner: Arc<Inner<KM>>,
}

struct Inner<KM: Keymap> {
    /// Bumped by every `replace`. Read on the keystroke path, so it is an
    /// atomic rather than a field behind the lock.
    generation: AtomicU64,
    current: RwLock<SessionConfig<KM>>,
}

impl<KM: Keymap> Clone for SharedConfig<KM> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<KM: Keymap> SharedConfig<KM> {
    /// Shares `config` with every session that does not override it.
    #[inline]
    pub fn new(config: SessionConfig<KM>) -> Self {
        Self {
            inner: Arc::new(Inner {
                generation: AtomicU64::new(0),
                current: RwLock::new(config),
            }),
        }
    }

    /// The generation of the current settings. Cheap; call it to find out
    /// whether anything moved since a previous read.
    #[inline]
    pub fn generation(&self) -> u64 {
        self.inner.generation.load(Ordering::Acquire)
    }

    /// Replaces the shared settings, so that every session following the
    /// shared config sees them from its next [`Session::refresh`].
    ///
    /// Returns the new generation.
    pub fn replace(&self, config: SessionConfig<KM>) -> u64 {
        // The value lands before the counter is published, so a session that
        // observes the new generation is guaranteed to read the new value
        // rather than the old one.
        *self
            .inner
            .current
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = config;
        self.inner.generation.fetch_add(1, Ordering::Release) + 1
    }
}

impl<KM: Keymap> SharedConfig<KM>
where
    KM: Clone,
{
    /// The settings in force right now.
    ///
    /// Takes the lock, so this is for a caller that is not on the keystroke
    /// path. A session does not use it: it reads [`SharedConfig::generation`]
    /// first and only looks at the value when something has actually moved.
    pub fn snapshot(&self) -> SessionConfig<KM> {
        // A poisoned lock means some other thread panicked mid-replace. The
        // value inside is still a whole `SessionConfig`, because the write is a
        // single assignment, so recovering beats pushing the panic out to every
        // keystroke of every session.
        self.inner
            .current
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }
}

impl<KM: Keymap> std::fmt::Debug for SharedConfig<KM> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SharedConfig")
            .field("generation", &self.generation())
            .finish_non_exhaustive()
    }
}

/// One typing buffer: the settings in force, and the composition they apply to.
///
/// By default a session follows a [`SharedConfig`], so a change to the shared
/// settings reaches it along with every other session. To pin a session to its
/// own settings use [`Session::with_config`], which opts it out until
/// [`Session::clear_private_config`] puts it back on the shared settings.
pub struct Session<KM: Keymap> {
    /// The settings in force, resolved from the private config or the shared
    /// one. Cached so the keystroke path never touches the lock.
    effective: SessionConfig<KM>,
    /// Set when this session does not follow the shared config.
    private: Option<SessionConfig<KM>>,
    shared: SharedConfig<KM>,
    /// The generation `effective` was resolved at, or `None` when it still has
    /// to be resolved.
    resolved: Option<u64>,
    /// The context the live composition is currently parsing under. Kept in
    /// step with `composition` by [`Session::adopt`].
    applied: SyllableContext<KM>,
    composition: Composition<KM>,
}

impl<KM: Keymap> Session<KM>
where
    KM: Clone + PartialEq,
{
    /// Creates an empty session that follows `shared`.
    pub fn new(shared: SharedConfig<KM>) -> Self {
        let effective = shared.snapshot();
        let mut session = Self {
            applied: effective.context.clone(),
            composition: Composition::new(SyllableBuilder::new(effective.context.clone())),
            effective,
            private: None,
            shared,
            // Force one resolution so a replacement racing with construction
            // cannot leave the session holding settings from a generation it
            // never named.
            resolved: None,
        };
        session.refresh();
        session
    }

    /// Creates an empty session with its own settings, which do not change when
    /// the shared config does.
    ///
    /// This session has no shared config of its own to return to, so
    /// [`Session::clear_private_config`] falls back to the settings it was
    /// created with. Use [`Session::with_config_on`] to come from an engine.
    pub fn with_config(private: SessionConfig<KM>) -> Self {
        Self::with_config_on(SharedConfig::new(private.clone()), private)
    }

    /// Creates an empty session with its own settings, which do not change when
    /// `shared` does.
    ///
    /// Unlike [`Session::with_config`] the session still remembers `shared`, so
    /// [`Session::clear_private_config`] puts it back on those settings — which
    /// is the pair of operations an application needs to give one buffer
    /// special treatment for a while and then take it back.
    pub fn with_config_on(shared: SharedConfig<KM>, private: SessionConfig<KM>) -> Self {
        Self {
            applied: private.context.clone(),
            composition: Composition::new(SyllableBuilder::new(private.context.clone())),
            effective: private.clone(),
            private: Some(private),
            shared,
            resolved: None,
        }
    }

    // ------------------------------------------------------------- settings

    /// The settings in force: the private config if this session has one,
    /// otherwise the shared config.
    #[inline]
    pub fn config(&self) -> &SessionConfig<KM> {
        &self.effective
    }

    /// The shared config this session follows.
    ///
    /// Replace it to move every session at once. A session with a private
    /// config keeps ignoring it until [`Session::clear_private_config`].
    #[inline]
    pub fn shared(&self) -> &SharedConfig<KM> {
        &self.shared
    }

    /// Whether this session is running under its own settings rather than
    /// following the shared config.
    #[inline]
    pub fn is_private(&self) -> bool {
        self.private.is_some()
    }

    /// Gives this session its own settings, which then stop following the
    /// shared config.
    ///
    /// A session that was following the shared config becomes a private one
    /// simply by being given a config; pass [`Session::config`]'s current value
    /// to opt out of following without changing anything.
    pub fn set_private_config(&mut self, private: SessionConfig<KM>) {
        self.adopt(private.clone());
        self.private = Some(private);
    }

    /// Drops the private config, so the session follows the shared one again
    /// and picks up whatever it says right now.
    pub fn clear_private_config(&mut self) {
        if self.private.take().is_none() {
            return;
        }
        self.resolved = None;
        self.refresh();
    }

    /// Re-resolves the settings if the shared config has moved since this
    /// session last looked.
    ///
    /// Returns `true` when the live composition was re-rendered as a result,
    /// meaning the word on screen changed without a key being pressed. A
    /// session with a private config is unaffected and returns `false`.
    pub fn refresh(&mut self) -> bool {
        let generation = self.shared.generation();
        if self.resolved == Some(generation) {
            return false;
        }
        self.resolved = Some(generation);

        if self.private.is_some() {
            return false;
        }

        self.adopt(self.shared.snapshot())
    }

    /// Makes `next` the settings in force, pushing whatever changed through to
    /// the live composition. Returns whether the word moved.
    fn adopt(&mut self, next: SessionConfig<KM>) -> bool {
        if next.context == self.applied {
            // Nothing the parser or the renderer reads has changed. The engine
            // `Config` may still be new, so record it without disturbing the
            // buffer.
            self.effective = next;
            return false;
        }

        if *next.context.keymap() == *self.applied.keymap() {
            // Same keymap, so the buffered keystrokes still mean what they
            // meant. Only the rendering moves, and it moves in place.
            self.composition.set_context(next.context.clone());
        } else {
            // A different keymap reinterprets the buffered keystrokes, so there
            // is nothing to keep. This is why `set_keymap` resets too.
            self.composition.set_keymap(next.context.keymap().clone());
            self.composition.reset();
        }

        self.applied = next.context.clone();
        self.effective = next;
        true
    }

    // ---------------------------------------------------------------- state

    /// The buffer as this session parses it, as a fresh [`String`].
    ///
    /// Parsing is what turns raw keystrokes into a word, so this is the
    /// spelled-out form: `aw` reads back as `ă`, not `aw`. Once the parse has
    /// failed there is nothing left to apply and the raw keystrokes come back
    /// verbatim.
    ///
    /// See [`Self::write_parsed_to`] for the version that does not allocate.
    #[inline]
    pub fn parsed(&self) -> String {
        let mut output = String::new();
        self.write_parsed_to(&mut output);
        output
    }

    /// Writes the parsed word into `output`, replacing its contents.
    ///
    /// The allocation-free counterpart to [`Self::parsed`]: a caller that writes
    /// on every keystroke can keep one `String` and reuse its capacity instead
    /// of building a new one each time.
    #[inline]
    pub fn write_parsed_to(&self, output: &mut String) {
        self.composition.write_parsed_to(output);
    }

    // ----------------------------------------------------------- key event

    /// Processes a full keyboard event and dispatches it to the session.
    ///
    /// Picks up a shared-config change first, so a session never renders a
    /// keystroke under settings that have already been replaced.
    pub fn process_key(&mut self, event: KeyEvent) -> Result {
        let reconfigured = self.refresh();
        let result = self.dispatch(event);

        // A settings change can move the rendered word even when the key itself
        // did nothing, and `Forward` would tell the frontend to leave the
        // screen alone.
        if reconfigured && matches!(result, Result::Forward | Result::CursorMoved) {
            return Result::Changed;
        }
        result
    }

    fn dispatch(&mut self, event: KeyEvent) -> Result {
        match event.key {
            Key::Character(character) => self.insert(character),
            Key::Backspace => self.backspace(),
            Key::Delete => self.delete(),
            Key::Left => self.move_left(),
            Key::Right => self.move_right(),
            Key::Space => self.commit_with_suffix(SUFFIX_SPACE),
            Key::Enter | Key::Tab | Key::Escape => self.commit(),
        }
    }

    /// Resets the session's composition to its initial empty state.
    pub fn reset(&mut self) -> Result {
        self.composition.reset();
        Result::Changed
    }

    /// Commits the current buffer and returns the resulting text.
    pub fn commit(&mut self) -> Result {
        self.commit_with_suffix("")
    }

    fn commit_with_suffix(&mut self, suffix: &str) -> Result {
        if self.composition.is_empty() {
            return Result::Forward;
        }

        let mut text = self.parsed();
        text.push_str(suffix);

        self.composition.reset();
        Result::Commit(text)
    }

    // ------------------------------------------------------------- editing

    #[inline]
    fn insert(&mut self, character: char) -> Result {
        self.composition.insert(character);
        Result::Changed
    }

    #[inline]
    fn backspace(&mut self) -> Result {
        if !self.composition.can_move_left() {
            return Result::Forward;
        }

        self.composition.backspace();
        Result::Changed
    }

    #[inline]
    fn delete(&mut self) -> Result {
        if !self.composition.can_move_right() {
            return Result::Forward;
        }

        self.composition.delete();
        Result::Changed
    }

    #[inline]
    fn move_left(&mut self) -> Result {
        if !self.composition.can_move_left() {
            return Result::Forward;
        }

        self.composition.move_left();
        Result::CursorMoved
    }

    #[inline]
    fn move_right(&mut self) -> Result {
        if !self.composition.can_move_right() {
            return Result::Forward;
        }

        self.composition.move_right();
        Result::CursorMoved
    }
}
