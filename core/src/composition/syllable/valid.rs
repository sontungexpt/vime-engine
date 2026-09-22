use crate::{
    keymap::Keymap,
    phonology::{
        decode_vowel, BaseVowel, CasedBaseVowel, Coda, NucleusStatus, Onset, RootVowel, Shape,
        Tone, ToneScheme,
    },
};
use arrayvec::ArrayVec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputResult {
    Transformed,
    Inserted,
}

/// Effect of applying a transform key (shape/tone mark) to the syllable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformEffect {
    /// Applied a new mark/transform to the syllable (e.g. `a` + `w` -> `ă`).
    Applied,
    /// Removed/undid an existing mark, stripping back to base (e.g. `ă` + `w` -> `a`).
    Reverted,
    /// The key cannot transform the current state; pass through as a literal char.
    Ignored,
}

/// The parsing phase the syllable is currently in.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PushPhase {
    /// Reading the initial consonant(s).
    #[default]
    Onset,
    /// Reading the vowel nucleus.
    Vowel,
    /// Reading the final consonant(s).
    Coda,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyllableError {
    /// The consonant cluster is not a valid Vietnamese onset.
    InvalidOnset,
    /// The vowel nucleus violates the Vietnamese vowel-rule table.
    InvalidNucleus,
    /// The final consonant cluster is not a valid Vietnamese coda.
    InvalidCoda,
}

/// A single Vietnamese syllable under construction.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ValidSyllableBuilder {
    onset_kind: Onset,
    onset: ArrayVec<char, { Onset::MAX_CHARS }>,

    vowels: ArrayVec<CasedBaseVowel, 3>,

    coda_kind: Coda,
    coda: ArrayVec<char, { Coda::MAX_CHARS }>,

    tone: Tone,

    push_phase: PushPhase,
}

impl ValidSyllableBuilder {
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.onset.len() + self.vowels.len() + self.coda.len()
    }

    #[inline(always)]
    pub fn onset(&self) -> &[char] {
        &self.onset
    }

    #[inline(always)]
    pub fn onset_kind(&self) -> Onset {
        self.onset_kind
    }

    #[inline(always)]
    pub fn coda(&self) -> &[char] {
        &self.coda
    }

    #[inline(always)]
    pub fn coda_kind(&self) -> Coda {
        self.coda_kind
    }

    #[inline(always)]
    pub fn vowels(&self) -> &[CasedBaseVowel] {
        &self.vowels
    }

    #[inline(always)]
    pub const fn tone(&self) -> Tone {
        self.tone
    }

    #[inline]
    pub fn reset(&mut self) {
        self.onset_kind = Onset::None;
        self.onset.clear();
        self.vowels.clear();
        self.coda_kind = Coda::None;
        self.coda.clear();
        self.tone = Tone::Flat;
        self.push_phase = PushPhase::Onset;
    }

    #[inline(always)]
    pub fn tone_index(&self, tone_scheme: ToneScheme) -> Option<usize> {
        tone_scheme.tone_index(&self.vowels, self.coda.is_empty())
    }

    #[inline(always)]
    pub fn chars(&self, tone_scheme: ToneScheme) -> Vec<char> {
        let mut output = Vec::with_capacity(self.len());

        output.extend(self.onset.iter().copied());

        let tone_position = self.tone_index(tone_scheme);
        for (index, vowel) in self.vowels.iter().enumerate() {
            let tone = if Some(index) == tone_position {
                self.tone()
            } else {
                Tone::Flat
            };

            output.push(vowel.to_char_tone(tone));
        }

        output.extend(self.coda.iter().copied());

        output
    }
}

impl ValidSyllableBuilder {
    /// Thao tác trên `coda` một cách an toàn bằng cách truyền vào closure `mutate` và `revert`.
    ///
    /// Nếu kiểm tra `Coda::from_chars` thành công, trạng thái `coda_kind` sẽ được cập nhật.
    /// Nếu thất bại, closure `revert` sẽ được gọi để hoàn tác mảng `coda`.
    pub fn try_update_coda<F, R>(&mut self, update: F, revert: R) -> bool
    where
        F: FnOnce(&mut ArrayVec<char, { Coda::MAX_CHARS }>),
        R: FnOnce(&mut ArrayVec<char, { Coda::MAX_CHARS }>),
    {
        if self.coda.len() >= Coda::MAX_CHARS {
            return false;
        }
        // 1. Thực hiện thao tác sửa đổi mảng coda
        update(&mut self.coda);

        // 2. Validation ngữ âm coda
        match Coda::from_chars(&self.coda) {
            Ok(kind) => {
                self.coda_kind = kind;
                true
            }
            Err(_) => {
                // 3. Revert nếu coda không hợp lệ
                revert(&mut self.coda);
                false
            }
        }
    }

    pub fn try_update_onset<F, R>(&mut self, update: F, revert: R) -> bool
    where
        F: FnOnce(&mut ArrayVec<char, { Onset::MAX_CHARS }>),
        R: FnOnce(&mut ArrayVec<char, { Onset::MAX_CHARS }>),
    {
        if self.onset.len() >= Onset::MAX_CHARS {
            return false;
        }

        // 1. Thực hiện thao tác sửa đổi mảng onset
        update(&mut self.onset);

        // 2. Validation ngữ âm onset
        match Onset::from_chars(&self.onset) {
            Ok(kind) => {
                self.onset_kind = kind;
                true
            }
            Err(_) => {
                // 3. Revert nếu onset không hợp lệ
                revert(&mut self.onset);
                false
            }
        }
    }

    /// Applies or toggles a tone on the syllable.
    ///
    /// Tapping the same tone again toggles the syllable back to `Flat`;
    /// a different tone replaces the current one.
    #[inline]
    fn transform_tone(&mut self, tone: Tone) -> TransformEffect {
        if self.vowels.is_empty() {
            return TransformEffect::Ignored;
        }
        // Same tone -> toggle back to Flat.
        else if self.tone == tone {
            self.tone = Tone::Flat;
            return TransformEffect::Reverted;
        }

        // Different tone -> replace the current tone.
        self.tone = tone;
        TransformEffect::Applied
    }

    /// Toggles the D-stroke on the onset cluster (`d` ↔ `đ`, `D` ↔ `Đ`).
    ///
    /// Only applies when the onset is a lone `D`/`Đ`; otherwise the stroke key
    /// cannot act here.
    #[inline]
    fn toggle_d_stroke(&mut self) -> TransformEffect {
        let onset_chars = &mut self.onset;

        match self.onset_kind {
            Onset::D => {
                onset_chars[0] = if onset_chars[0] == 'd' { 'đ' } else { 'Đ' };
                self.onset_kind = Onset::DStroke;
                TransformEffect::Applied
            }
            Onset::DStroke => {
                onset_chars[0] = if onset_chars[0] == 'đ' { 'd' } else { 'D' };
                self.onset_kind = Onset::D;
                TransformEffect::Reverted
            }
            _ => TransformEffect::Ignored,
        }
    }

    /// Whether the nucleus starts with an unmarked `u o` pair.
    #[inline(always)]
    fn vowels_starts_with_uo(&self) -> bool {
        let vowels = &self.vowels;
        vowels.len() > 1
            && vowels[0].value.root() == RootVowel::U
            && vowels[1].value.root() == RootVowel::O
    }

    /// Normalizes an unmarked `u o` prefix that arrived without a shape key.
    ///
    /// `uơ → ươ` and `ưo → ươ` are folded once at least two vowels are present.
    #[inline]
    fn finalize_uo_prefix(&mut self) {
        let vowels = &mut self.vowels;
        if vowels.len() < 2 {
            return;
        }

        match (vowels[0].value, vowels[1].value) {
            (BaseVowel::U, BaseVowel::OHorn) => {
                vowels[0].value = BaseVowel::UHorn;
            }
            (BaseVowel::UHorn, BaseVowel::O) => {
                vowels[1].value = BaseVowel::OHorn;
            }
            _ => {}
        }
    }

    /// Validates the vowel nucleus against the rule table, keeping only the
    /// first `N` vowels.
    #[inline(always)]
    fn validate_vowels(&self) -> NucleusStatus {
        let mut buf = [BaseVowel::A; 3];
        let len = self.vowels.len().min(3);

        for i in 0..len {
            buf[i] = self.vowels[i].value;
        }

        NucleusStatus::from_vowels(&buf[..len])
    }

    /// Applies a Horn shape to the `u o` prefix (requires at least 2 vowels).
    fn transform_uo_horn(&mut self) -> TransformEffect {
        let vowels = &self.vowels;

        match (vowels[0].value, vowels[1].value) {
            // ươ -> uow
            //
            // This always reverts, even with a third vowel.
            (BaseVowel::UHorn, BaseVowel::OHorn) => {
                self.vowels[0].value = BaseVowel::U;
                self.vowels[1].value = BaseVowel::O;
                TransformEffect::Reverted
            }

            // ưô -> ươ
            (BaseVowel::UHorn, BaseVowel::OCircumflex) => {
                self.try_transform_vowel_shape(1, Shape::Horn)
            }

            // ưo -> ươ
            (BaseVowel::UHorn, BaseVowel::O) => self.try_transform_vowel_shape(1, Shape::Horn),

            // uo -> uơ
            (BaseVowel::U, BaseVowel::O) => self.try_transform_vowel_shape(1, Shape::Horn),

            // uơ -> ươ
            (BaseVowel::U, BaseVowel::OHorn) => self.try_transform_vowel_shape(0, Shape::Horn),

            // uô -> uơ
            (BaseVowel::U, BaseVowel::OCircumflex) => {
                self.try_transform_vowel_shape(1, Shape::Horn)
            }

            _ => TransformEffect::Ignored,
        }
    }

    /// Applies a Circumflex shape to the `u o` prefix (requires at least 2 vowels).
    fn transform_uo_circumflex(&mut self) -> TransformEffect {
        let vowels = &self.vowels;
        let first = vowels[0].value;

        match (first, vowels[1].value) {
            // ươ -> uô
            (BaseVowel::UHorn, BaseVowel::OHorn) => {
                self.vowels[0].value = BaseVowel::U;

                match self.try_transform_vowel_shape(1, Shape::Circumflex) {
                    effect @ (TransformEffect::Applied | TransformEffect::Reverted) => effect,
                    TransformEffect::Ignored => {
                        self.vowels[0].value = first;
                        TransformEffect::Ignored
                    }
                }
            }

            // ưô -> not applicable
            (BaseVowel::UHorn, BaseVowel::OCircumflex) => TransformEffect::Ignored,

            // ưo -> uô
            (BaseVowel::UHorn, BaseVowel::O) => {
                self.vowels[0].value = BaseVowel::U;

                match self.try_transform_vowel_shape(1, Shape::Circumflex) {
                    effect @ (TransformEffect::Applied | TransformEffect::Reverted) => effect,
                    TransformEffect::Ignored => {
                        self.vowels[0].value = first;
                        TransformEffect::Ignored
                    }
                }
            }

            // uo -> uô
            (BaseVowel::U, BaseVowel::O) => self.try_transform_vowel_shape(1, Shape::Circumflex),

            // uơ -> uô
            (BaseVowel::U, BaseVowel::OHorn) => {
                self.try_transform_vowel_shape(1, Shape::Circumflex)
            }

            // uô -> undo (revert to "uo")
            (BaseVowel::U, BaseVowel::OCircumflex) => {
                self.vowels[1].value = BaseVowel::O;
                TransformEffect::Reverted
            }
            _ => TransformEffect::Ignored,
        }
    }

    /// Tries to apply `key` as a shape transform on the vowel sequence,
    /// scanning from the last vowel backwards.
    fn try_transform_shape<KM: Keymap>(
        &mut self,
        keymap: &KM,
        key: char,
        upper_bound_idx: Option<usize>,
    ) -> TransformEffect {
        // 1. Xác định giới hạn con trỏ (upper bound)
        let max_len = match upper_bound_idx {
            Some(idx) => idx.min(self.vowels.len()),
            None => self.vowels.len(),
        };

        // Nếu con trỏ ở ngay vị trí 0, không có nguyên âm nào bên trái -> Bỏ qua ngay lập tức!
        if max_len == 0 {
            return TransformEffect::Ignored;
        }
        // 2. Xử lý trường hợp đặc biệt "uo" (uow -> ươ, uoo -> uô)
        // ĐIỀU KIỆN MỚI: Chỉ check "uo" khi con trỏ nằm SAU chữ 'u' (tức max_len >= 1)
        else if self.vowels_starts_with_uo() {
            if let Some(shape) = keymap
                .decode_shape(key, RootVowel::O)
                .or_else(|| keymap.decode_shape(key, RootVowel::U))
            {
                match shape {
                    Shape::Horn => return self.transform_uo_horn(),
                    Shape::Circumflex => return self.transform_uo_circumflex(),
                    _ => {}
                }
            }
        }

        // 3. Duyệt ngược từ (max_len - 1) lùi về 0 (CHỈ xét các nguyên âm đứng TRƯỚC con trỏ)
        for index in (0..max_len).rev() {
            let base = self.vowels[index].value;

            if let Some(shape) = keymap.decode_shape(key, base.root()) {
                let effect = self.try_transform_vowel_shape(index, shape);
                if effect != TransformEffect::Ignored {
                    return effect;
                }
            }
        }

        TransformEffect::Ignored
    }

    /// Applies `shape` to the vowel at `index`, re-validating the nucleus.
    ///
    /// Applying the shape it already has reverts it; an invalid result rolls
    /// the vowel back.
    fn try_transform_vowel_shape(&mut self, index: usize, shape: Shape) -> TransformEffect {
        let old = self.vowels[index].value;

        if old.has_shape(shape) && shape.is_some() {
            self.vowels[index].value = old.remove_shape();
            return TransformEffect::Reverted;
        }

        let Ok(new) = old.replace_shape(shape) else {
            return TransformEffect::Ignored;
        };

        self.vowels[index].value = new;

        if self.vowels.len() < 2 {
            return TransformEffect::Applied;
        }

        match self.validate_vowels() {
            NucleusStatus::Valid => TransformEffect::Applied,
            NucleusStatus::InComplete => TransformEffect::Applied,
            NucleusStatus::Dead => {
                self.vowels[index].value = old;
                TransformEffect::Ignored
            }
        }
    }

    #[inline]
    fn try_toggle_d_stroke<KM: Keymap>(&mut self, keymap: &KM, key: char) -> TransformEffect {
        if self.onset.len() > 0 && keymap.is_stroke_key(key) {
            return self.toggle_d_stroke();
        }
        TransformEffect::Ignored
    }

    #[inline]
    fn try_transform<KM: Keymap>(
        &mut self,
        keymap: &KM,
        key: char,
        upper_bound_idx: Option<usize>,
    ) -> TransformEffect {
        // 1. Tone
        if let Some(tone) = keymap.decode_tone(key) {
            return self.transform_tone(tone);
        }
        // 2. Shape
        else if keymap.is_shape_key(key) {
            return self.try_transform_shape(keymap, key, upper_bound_idx);
        }
        // 3. D-stroke
        self.try_toggle_d_stroke(keymap, key)
    }
}

impl ValidSyllableBuilder {
    /// Chèn một ký tự `input` vào vị trí `index` bất kỳ trong âm tiết.
    #[inline]
    pub fn insert<KM: Keymap>(
        &mut self,
        keymap: &KM,
        index: usize,
        key: char,
    ) -> Result<InputResult, SyllableError> {
        let total_len = self.len();

        // 1. Chèn vào cuối -> chính là push()
        if index >= total_len {
            return self.push(keymap, key);
        }

        let onset_len = self.onset.len();
        let vowels_len = self.vowels.len();

        // 2. Định tuyến theo vùng index
        if index <= onset_len {
            // 1. Phím gạch ngang (d -> đ)
            if self.try_toggle_d_stroke(keymap, key) == TransformEffect::Applied {
                return Ok(InputResult::Transformed);
            }

            self.insert_onset(keymap, index, key)
        } else if index <= onset_len + vowels_len {
            let vowel_idx = index - onset_len;
            self.insert_vowel(keymap, vowel_idx, key)
        } else {
            // 1. Áp dụng dấu thanh nếu gõ phím biến đổi khi con trỏ ở vùng Coda
            if self.try_transform(keymap, key, Some(index)) == TransformEffect::Applied {
                return Ok(InputResult::Transformed);
            }
            let coda_idx = index - onset_len - vowels_len;
            self.insert_coda(coda_idx, key)
        }
    }

    // ─────────────────────────── Insert Onset ───────────────────────────

    pub fn insert_onset<KM: Keymap>(
        &mut self,
        keymap: &KM,
        index: usize,
        input: char,
    ) -> Result<InputResult, SyllableError> {
        let at_end_onset = self.onset.len() == index;

        // 2. Thử mở rộng Onset (thường xảy ra khi index == onset.len() hoặc chèn phụ âm vào giữa)
        if self.try_update_onset(
            |onset| onset.insert(index, input),
            |onset| {
                onset.remove(index);
            },
        ) {
            return Ok(InputResult::Inserted);
        }
        // Nếu chèn ở giữa Onset (index < onset.len()) mà Onset không nhận -> Báo lỗi InvalidOnset
        else if !at_end_onset {
            return Err(SyllableError::InvalidOnset);
        }

        // 4. Khi index == onset.len() nhưng input là nguyên âm -> Đẩy sang đầu mảng Vowel (index 0)
        self.insert_vowel(keymap, 0, input)
    }

    // ─────────────────────────── Insert Vowel ───────────────────────────

    pub fn insert_vowel<KM: Keymap>(
        &mut self,
        keymap: &KM,
        vowel_index: usize,
        input: char,
    ) -> Result<InputResult, SyllableError> {
        // 1. Áp dụng dấu thanh nếu gõ phím biến đổi khi con trỏ ở vùng Vowel
        if self.try_transform(keymap, input, Some(vowel_index)) == TransformEffect::Applied {
            return Ok(InputResult::Transformed);
        }

        // ─── Chèn nguyên âm thô (Literal Vowel Insertion) ────────────────────
        // Nếu không phải là phím modifier (hoặc modifier bị Ignored), giải mã
        // và chèn input như một nguyên âm mới.
        let Some((base_cased, tone)) = decode_vowel(input) else {
            if vowel_index == self.vowels.len() {
                // Chuẩn hóa tiền tố uo / ươ nếu có coda đi kèm
                self.finalize_uo_prefix();
                return self.insert_coda(0, input);
            }

            return Err(SyllableError::InvalidNucleus);
        };

        // Tiếng Việt tối đa 3 nguyên âm
        if self.vowels.len() >= 3 {
            return Err(SyllableError::InvalidNucleus);
        }

        // Kiểm tra xung đột dấu thanh đối với nguyên âm mới chèn vào
        let old_tone = self.tone;
        if self.tone != Tone::Flat && tone != Tone::Flat && self.tone != tone {
            return Err(SyllableError::InvalidNucleus);
        }

        // Chèn nguyên âm mới vào đúng vị trí vowel_index
        self.vowels.insert(vowel_index, base_cased);
        if self.tone == Tone::Flat {
            self.tone = tone;
        }

        // Validate lại toàn bộ Nucleus
        if self.validate_vowels() == NucleusStatus::Dead {
            // Rollback nếu chuỗi nguyên âm mới tạo ra không hợp lệ
            self.vowels.remove(vowel_index);
            self.tone = old_tone;
            return Err(SyllableError::InvalidNucleus);
        }

        // Chuẩn hóa tiền tố uo / ươ nếu chạm mốc >= 2 nguyên âm
        if self.vowels.len() == 3 {
            self.finalize_uo_prefix();
        }

        Ok(InputResult::Inserted)
    }

    // ─────────────────────────── Insert Coda ───────────────────────────

    pub fn insert_coda(
        &mut self,
        coda_index: usize,
        input: char,
    ) -> Result<InputResult, SyllableError> {
        // Chèn phụ âm cuối vào mảng Coda
        if self.try_update_coda(
            |coda| coda.insert(coda_index, input),
            |coda| {
                coda.remove(coda_index);
            },
        ) {
            // always in coda phase if has coda
            self.push_phase = PushPhase::Coda;
            return Ok(InputResult::Inserted);
        }

        Err(SyllableError::InvalidCoda)
    }
}

impl ValidSyllableBuilder {
    /// Feeds one character into the syllable, routed by the current parse phase.
    ///
    /// A transform key (tone / shape / stroke) is given the first chance to
    /// modify what is already typed. Only an `Applied` effect consumes the key;
    /// `Reverted` and `Ignored` fall through so the same key can still be parsed
    /// as a literal character.
    #[inline(always)]
    pub fn push<KM: Keymap>(
        &mut self,
        keymap: &KM,
        key: char,
    ) -> Result<InputResult, SyllableError> {
        let curr_phase = self.push_phase;
        match curr_phase {
            PushPhase::Onset => {
                // A stroke key strokes an existing onset letter (`d` -> `đ`), so
                // it only acts once the onset already holds a character.
                if self.try_toggle_d_stroke(keymap, key) == TransformEffect::Applied {
                    // Applied: the onset became `đ` / `Đ`; the key is consumed.
                    //   `d` + `d` -> `đ`, `D` + `D` -> `Đ`
                    return Ok(InputResult::Transformed);
                }

                // Reverted: the key removed an existing stroke, then falls
                // through and is re-parsed as a literal onset character.
                //   `đ` + `d` -> `d` (stroke removed) then literal `d`
                // Ignored: the onset cannot be stroked, so the key is an
                // ordinary literal (rejected here if it breaks the onset).
                //   `b` + `d` -> `b` then literal `d` -> invalid onset
                self.push_onset(key)
            }
            PushPhase::Vowel | PushPhase::Coda => {
                // A transform key (tone / shape / stroke) is offered to the
                // current syllable first. Only `Applied` consumes it; every
                // other effect falls through to be parsed as a literal.
                if self.try_transform(keymap, key, None) == TransformEffect::Applied {
                    // Applied: the mark changed the syllable; the key is consumed.
                    //   `a` + `s` -> `á` (tone), `a` + `w` -> `ă` (shape)
                    return Ok(InputResult::Transformed);
                }

                // Reverted: an equal mark was toggled off; the key then falls
                // through and is parsed as a literal (it may join the nucleus or
                // land in the coda).
                //   `ô` + `o` -> `oo` (circumflex removed, literal `o` added)
                // Ignored: nothing to transform, so the key is a plain literal.
                //   `i` + `e` -> `ie` (`e` cannot shape `i`, so it is inserted)
                match curr_phase {
                    PushPhase::Vowel => self.push_vowel(key),
                    PushPhase::Coda => self.push_coda(key),
                    PushPhase::Onset => unreachable!(),
                }
            }
        }
    }

    // ─────────────────────────── Onset ───────────────────────────
    /// Handles a literal (non-transform) character while in the `Onset` phase.
    ///
    /// `q` is kept as a transitional prefix waiting for `u` (to form `qu`);
    /// a vowel ends the onset and starts the nucleus; a consonant is appended
    /// if it still forms a valid cluster — otherwise the parse is killed.
    fn push_onset(&mut self, input: char) -> Result<InputResult, SyllableError> {
        let onset_len = self.onset.len();

        // `q` is a transitional onset prefix; wait for `u`.
        if onset_len == 0 && (input as u32 | 0x20) == ('q' as u32) {
            self.onset.push(input);
            return Ok(InputResult::Inserted);
        } else if let Some((base_cased, tone)) = decode_vowel(input) {
            // qu
            // A `u` following a lone `q` is kept as the onset of `qu`.
            // Only the plain `u` counts here: a precomposed `ư` is an actual vowel.
            if onset_len == 1 && (self.onset[0] as u32 | 0x20) == ('q' as u32) {
                if base_cased.value == BaseVowel::U {
                    self.onset.push(input);
                    self.onset_kind = Onset::Qu;
                    return Ok(InputResult::Inserted);
                }

                // `q` cannot start a vowel nucleus by itself.
                return Err(SyllableError::InvalidOnset);
            }

            if self.try_push_vowel(base_cased, tone) {
                self.push_phase = PushPhase::Vowel;
                return Ok(InputResult::Inserted);
            }

            return Err(SyllableError::InvalidNucleus);
        } else if self.try_update_onset(
            |onset| onset.push(input),
            |onset| {
                onset.pop();
            },
        ) {
            return Ok(InputResult::Inserted);
        }

        Err(SyllableError::InvalidOnset)
    }

    // ─────────────────────────── Vowel ───────────────────────────

    /// Appends an already-decoded vowel to the nucleus.
    ///
    /// Returns `true` when the vowel joined the nucleus (possibly resolving
    /// `gi` as a phoneme boundary). Returns `false` when the nucleus cannot
    /// accept it (too many vowels, tone conflict, or an invalid sequence);
    /// the caller is responsible for reporting the failure.
    fn try_push_vowel(&mut self, cased_base: CasedBaseVowel, tone: Tone) -> bool {
        let vowels_len = self.vowels.len();

        if vowels_len == 0 {
            // First vowel; add it to the nucleus.
            self.vowels.push(cased_base);
            self.tone = tone;
            return true;
        }
        // Vietnamese vowel sequence supports at most 3 vowels.
        else if vowels_len >= 3 {
            return false;
        }

        // Append the new vowel to the sequence.

        let old_tone = self.tone;

        // A pretoned vowel carrying a tone must not conflict with the tone
        // already on the syllable. e.g. "á" is already typed and a raw "ắ" is
        // pushed straight into the buffer - the two tones would collide (áắ)
        if self.tone == Tone::Flat {
            // If the tone is currently flat, a pretoned vowel can be used.
            // For example `i` can be contiued typed with an pretoned vowel `ế`. to `iế`
            self.tone = tone;
        } else if tone != Tone::Flat {
            // Tone conflict
            return false;
        }

        // ---------------------------------------------------------
        // Special case: gi
        // A syllable prefix of "gi" is ambiguous: if another vowel
        // follows the 'i', "gi" becomes the onset; otherwise 'g'
        // stays the onset and the 'i' is the nucleus.
        // ---------------------------------------------------------
        if vowels_len == 1 && self.vowels[0].value == BaseVowel::I && self.onset_kind == Onset::G {
            let i = self.vowels.pop().unwrap();
            self.onset.push(if i.is_upper() { 'I' } else { 'i' });
            self.onset_kind = Onset::Gi;
        }

        self.vowels.push(cased_base);

        if NucleusStatus::Dead == self.validate_vowels() {
            // rollback state
            self.vowels.pop();
            self.tone = old_tone;
            return false;
        }

        true
    }

    /// Handles a literal character while in the `Vowel` phase.
    ///
    /// A vowel joins the nucleus (or falls through to kill); a possible coda
    /// starter moves the phase into `Coda` and appends the character; anything
    /// else (invalid coda, non-letter) kills the parse.
    fn push_vowel(&mut self, input: char) -> Result<InputResult, SyllableError> {
        if let Some((base_cased, tone)) = decode_vowel(input) {
            if !self.try_push_vowel(base_cased, tone) {
                return Err(SyllableError::InvalidNucleus);
            }

            if self.vowels.len() == 3 {
                self.finalize_uo_prefix();
            }

            return Ok(InputResult::Inserted);
        }
        // Not a vowel then may be a coda
        else if self.try_push_coda(input) {
            // `normalize_uo` validates the length itself and only acts once there
            // are at least two vowels - which is exactly when a coda character is
            // being pushed here.
            self.finalize_uo_prefix();
            self.push_phase = PushPhase::Coda;
            return Ok(InputResult::Inserted);
        }

        Err(SyllableError::InvalidCoda)
    }

    // ─────────────────────────── Coda ───────────────────────────

    /// Handles a literal character while in the `Coda` phase; kills the parse
    /// unless the consonant extends the coda to a valid cluster.
    #[inline(always)]
    fn push_coda(&mut self, input: char) -> Result<InputResult, SyllableError> {
        if self.try_push_coda(input) {
            return Ok(InputResult::Inserted);
        }

        Err(SyllableError::InvalidCoda)
    }

    /// Appends a consonant to the coda, returning `false` when the extended coda
    /// is full or no longer a valid Vietnamese coda (the character is rolled
    /// back in that case).
    #[inline(always)]
    fn try_push_coda(&mut self, input: char) -> bool {
        self.try_update_coda(
            |coda| coda.push(input),
            |coda| {
                coda.pop();
            },
        )
    }
}
