//! Chính tả (spelling) validation: kiểm tra sự kết hợp hợp lệ của các thành phần
//! trong vần tiếng Việt — phụ âm đầu (onset), nguyên âm (nucleus), phụ âm cuối
//! (coda) và thanh điệu (tone).
//!
//! # Cơ chế Tối ưu bằng Bitmask:
//! Thay vì giải mã các enum `RootVowel`, `Shape` hoặc gọi hàm so sánh runtime,
//! toàn bộ thuộc tính âm vị học được mã hóa thành các cờ bit trong `PhonotacticFlags`.
//!
//! Khi kiểm tra quy tắc chính tả, validator chỉ thực hiện các phép toán bitwise (`&`, `|`)
//! giúp CPU thực thi không rẽ nhánh (branchless execution) và hoàn toàn tương thích `const fn`.

use crate::phonology::{BaseVowel, CasedBaseVowel, Coda, Onset, Tone};

/// Bitmask mã hóa các thuộc tính ÂM VỊ HỌC (phonotactic attributes) của vần.
///
/// Định dạng: 16-bit integer (u16)
/// - Bits 0..5  : Dành cho Nguyên âm (Vowel/Nucleus properties)
/// - Bits 6..8  : Dành cho Phụ âm đầu (Onset constraints)
/// - Bits 9..10 : Dành cho Phụ âm cuối (Coda constraints)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct PhonotacticFlags(u16);

impl PhonotacticFlags {
    // ─────────────── Bitmask Nguyên âm (Nucleus / Vowel Flags: Bits 0..5) ───────────────

    /// Nguyên âm hàng trước (Front Vowels): `i`, `y`, `e`, `ê`.
    /// Chi phối luật kết hợp với các phụ âm đầu như `k/gh/ngh` và `c/g/ng`.
    pub const VOWEL_FRONT: Self = Self(1 << 0);

    /// Nguyên âm ngắn (Short Vowels): `ă`, `â`.
    /// Bắt buộc phải có âm đóng (phụ âm cuối) đi kèm, không được đứng mở ở cuối vần.
    pub const VOWEL_SHORT: Self = Self(1 << 1);

    /// Nguyên âm cho phép phụ âm ngạc đi kèm: `i`, `y`, `e`, `ê`, `a`.
    /// Dùng để kiểm tra luật phụ âm cuối ngạc (`ch`, `nh`).
    pub const VOWEL_ALLOWS_PALATAL: Self = Self(1 << 2);

    /// Nguyên âm `u`.
    /// Dùng để cấm tổ hợp viết thừa âm đệm như `quu` (khi đã có `Qu`).
    pub const VOWEL_U: Self = Self(1 << 3);

    /// Nguyên âm tròn môi (Round Vowels): `u`, `o`, `ô`.
    pub const VOWEL_ROUND: Self = Self(1 << 4);

    /// Dòng nguyên âm `a` (bao gồm cả `a` thường, `ă`, `â`).
    pub const VOWEL_A: Self = Self(1 << 5);

    // ─────────────── Bitmask Phụ âm đầu (Onset Flags: Bits 6..8) ───────────────

    /// Phụ âm đầu BẮT BUỘC đi với nguyên âm hàng trước `i, e, ê, y` (gồm: `k`, `gh`, `ngh`).
    pub const ONSET_REQUIRES_FRONT: Self = Self(1 << 6);

    /// Phụ âm đầu CẤM đi với nguyên âm hàng trước `i, e, ê, y` (gồm: `c`, `g`, `ng`).
    pub const ONSET_FORBIDS_FRONT: Self = Self(1 << 7);

    /// Phụ âm đầu có âm đệm môi `Qu`: cấm nguyên âm theo sau là `u`.
    pub const ONSET_LABIOVELAR: Self = Self(1 << 8);

    // ─────────────── Bitmask Phụ âm cuối (Coda Flags: Bits 9..10) ───────────────

    /// Phụ âm cuối TẮC (Stop Codas): `p`, `t`, `c`, `ch`.
    /// Buộc vần phải mang thanh Sắc (`Acute`) hoặc Nặng (`Dot`) (thanh nhập).
    pub const CODA_STOP: Self = Self(1 << 9);

    /// Phụ âm cuối NGẠC (Palatal Codas): `ch`, `nh`.
    /// Chỉ được đứng sau các nguyên âm thuộc tập `VOWEL_ALLOWS_PALATAL`.
    pub const CODA_PALATAL: Self = Self(1 << 10);

    // ─────────────── Helper Methods cho Bitwise Operations ───────────────

    /// Tạo cờ rỗng (tất cả các bit đều bằng 0).
    #[inline(always)]
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Lấy giá trị nguyên `u16` thô đại diện cho cờ.
    #[inline(always)]
    pub const fn bits(self) -> u16 {
        self.0
    }

    /// Khởi tạo cờ từ giá trị `u16` thô.
    #[inline(always)]
    pub const fn from_bits_truncate(bits: u16) -> Self {
        Self(bits)
    }

    /// Kiểm tra xem `self` có chứa TOÀN BỘ các bit của `other` hay không.
    #[inline(always)]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Kiểm tra xem `self` và `other` có CHUNG ÍT NHẤT MỘT bit nào không.
    #[inline(always)]
    pub const fn intersects(self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }

    /// Kiểm tra cờ có bằng 0 hay không.
    #[inline(always)]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Hợp (OR) hai tập cờ lại với nhau.
    #[inline(always)]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl std::ops::BitOr for PhonotacticFlags {
    type Output = Self;

    #[inline(always)]
    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl std::ops::BitOrAssign for PhonotacticFlags {
    #[inline(always)]
    fn bitor_assign(&mut self, other: Self) {
        self.0 |= other.0;
    }
}

impl Onset {
    /// Ánh xạ Phụ âm đầu sang Bitmask ràng buộc chính tả.
    #[inline(always)]
    pub const fn phonotactic_flags(self) -> PhonotacticFlags {
        match self {
            // k, gh, ngh -> Bắt buộc đi với i, e, ê, y
            Self::K | Self::Gh | Self::Ngh => PhonotacticFlags::ONSET_REQUIRES_FRONT,
            // c, g, ng -> Tuyệt đối cấm đi với i, e, ê, y
            Self::C | Self::G | Self::Ng => PhonotacticFlags::ONSET_FORBIDS_FRONT,
            // qu -> Cấm nguyên âm u đi ngay sau
            Self::Qu => PhonotacticFlags::ONSET_LABIOVELAR,
            // Các phụ âm đầu khác không có quy tắc ràng buộc đặc biệt
            _ => PhonotacticFlags::empty(),
        }
    }
}

impl Coda {
    /// Ánh xạ Phụ âm cuối sang Bitmask ràng buộc chính tả.
    #[inline(always)]
    pub const fn phonotactic_flags(self) -> PhonotacticFlags {
        match self {
            // p, t, c -> Âm tắc thuần túy (yêu cầu thanh Sắc / Nặng)
            Self::P | Self::T | Self::C => PhonotacticFlags::CODA_STOP,
            // ch -> Vừa là âm tắc, vừa là âm ngạc
            Self::Ch => PhonotacticFlags::CODA_STOP.union(PhonotacticFlags::CODA_PALATAL),
            // nh -> Âm ngạc thuần túy
            Self::Nh => PhonotacticFlags::CODA_PALATAL,
            // Không có phụ âm cuối hoặc các phụ âm khác (m, n, ng...)
            _ => PhonotacticFlags::empty(),
        }
    }
}

impl Tone {
    /// Kiểm tra thanh điệu có phải là thanh Nhập (Sắc/Nặng) hay không.
    ///
    /// Trong tiếng Việt, các từ có phụ âm cuối tắc (p, t, c, ch) chỉ chấp nhận thanh Sắc hoặc Nặng.
    #[inline(always)]
    pub const fn allows_stop_coda(self) -> bool {
        matches!(self, Self::Acute | Self::Dot)
    }
}

impl BaseVowel {
    /// Ánh xạ Nguyên âm đơn sang Bitmask thuộc tính tổng hợp.
    ///
    /// Dùng `union` (const fn) để gộp cờ trực tiếp mà không cần truy cập trường `.0`.
    #[inline(always)]
    pub const fn phonotactic_flags(self) -> PhonotacticFlags {
        match self {
            // i, e, ê, y -> Nguyên âm hàng trước & Cho phép đứng trước ch/nh
            Self::I | Self::E | Self::ECircumflex | Self::Y => {
                PhonotacticFlags::VOWEL_FRONT.union(PhonotacticFlags::VOWEL_ALLOWS_PALATAL)
            }
            // a -> Thuộc dòng 'a' & Cho phép đứng trước ch/nh
            Self::A => PhonotacticFlags::VOWEL_A.union(PhonotacticFlags::VOWEL_ALLOWS_PALATAL),
            // ă, â -> Thuộc dòng 'a' & Nguyên âm ngắn (cần âm đóng)
            Self::ABreve | Self::ACircumflex => {
                PhonotacticFlags::VOWEL_A.union(PhonotacticFlags::VOWEL_SHORT)
            }
            // u -> Tròn môi & Cấm đi ngay sau Qu
            Self::U => PhonotacticFlags::VOWEL_ROUND.union(PhonotacticFlags::VOWEL_U),
            // o, ô -> Tròn môi
            Self::O | Self::OCircumflex => PhonotacticFlags::VOWEL_ROUND,
            // ư, ơ -> Không có cờ đặc biệt
            Self::UHorn | Self::OHorn => PhonotacticFlags::empty(),
        }
    }
}

/// Gộp (OR) cờ của toàn bộ nguyên âm cấu thành nucleus (tối đa 3 nguyên âm: ví dụ `oai`, `uyê`).
///
/// Dùng để tra cứu tổng thể thuộc tính của cả phần vần.
#[inline(always)]
pub const fn nucleus_flags(vowels: &[BaseVowel]) -> PhonotacticFlags {
    let mut bits = 0u16;
    let len = if vowels.len() > 3 { 3 } else { vowels.len() };
    let mut i = 0;
    while i < len {
        bits |= vowels[i].phonotactic_flags().bits();
        i += 1;
    }
    PhonotacticFlags(bits)
}

/// Tương tự [`nucleus_flags`], nhưng làm việc trực tiếp trên dữ liệu `CasedBaseVowel`
/// trong quá trình người dùng gõ phím.
#[inline(always)]
pub const fn cased_nucleus_flags(vowels: &[CasedBaseVowel]) -> PhonotacticFlags {
    let mut bits = 0u16;
    let len = if vowels.len() > 3 { 3 } else { vowels.len() };
    let mut i = 0;
    while i < len {
        bits |= vowels[i].value().phonotactic_flags().bits();
        i += 1;
    }
    PhonotacticFlags(bits)
}

/// Các loại lỗi chính tả âm vị học có thể xảy ra.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ValidationError {
    /// Phụ âm `k/gh/ngh` đứng trước nguyên âm không phải hàng trước (thiếu `i, e, ê, y`).
    MissingFrontVowel,
    /// Phụ âm `c/g/ng` đứng trước nguyên âm hàng trước (`i, e, ê, y`).
    ForbiddenFrontVowel,
    /// Phụ âm `Qu` đi ngay trước nguyên âm `u` (viết thừa `quu`).
    GlideAfterQu,
    /// Phụ âm cuối tắc (`p, t, c, ch`) nhưng thiếu thanh Sắc hoặc Nặng.
    EnteringToneRequired,
    /// Phụ âm cuối ngạc (`ch, nh`) ghép sai nguyên âm (không phải `i, e, ê, y, a`).
    PalatalCodaVowelMismatch,
    /// Nguyên âm ngắn (`ă, â`) đứng ở vị trí mở cuối nucleus nhưng không có phụ âm cuối.
    CodaRequiredForShortVowel,
}

/// Kiểm tra tính hợp lệ chính tả của vần.
pub trait PhonotacticValidator {
    /// Kiểm tra tính hợp lệ chính tả của vần bằng các phép toán bitmask.
    ///
    /// # Luồng xử lý:
    /// 1. Trích xuất cờ bitmask của nguyên âm ĐẦU (`first`) và nguyên âm CUỐI (`last`) trong nucleus.
    /// 2. Áp dụng các quy tắc chính tả dựa trên phép giao bitwise (`intersects`).
    fn validate(
        &self,
        onset: Onset,
        vowels: &[BaseVowel],
        coda: Coda,
        tone: Tone,
    ) -> Result<(), ValidationError>;
}

/// Triển khai mặc định của [`PhonotacticValidator`] theo quy tắc chính tả tiếng Việt chuẩn.
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultPhonotacticValidator;

impl PhonotacticValidator for DefaultPhonotacticValidator {
    #[inline(always)]
    fn validate(
        &self,
        onset: Onset,
        vowels: &[BaseVowel],
        coda: Coda,
        tone: Tone,
    ) -> Result<(), ValidationError> {
        // Trích xuất cờ bitmask của nguyên âm đầu và cuối trong nucleus mà không gây ra bounds check overhead.
        let (first_flags, last_flags) = match vowels {
            [] => return Ok(()),
            [single] => {
                let flags = single.phonotactic_flags();
                (flags, flags)
            }
            [head, .., tail] => (head.phonotactic_flags(), tail.phonotactic_flags()),
        };

        let o_flags = onset.phonotactic_flags();

        // ---------------------------------------------------------------------
        // Luật 1: Ràng buộc nguyên âm hàng trước đối với phụ âm đầu (xét nguyên âm ĐẦU)
        // ---------------------------------------------------------------------
        // k, gh, ngh -> Bắt buộc nguyên âm ngay sau phải thuộc tập FRONT (i, e, ê, y)
        if o_flags.intersects(PhonotacticFlags::ONSET_REQUIRES_FRONT)
            && !first_flags.intersects(PhonotacticFlags::VOWEL_FRONT)
        {
            return Err(ValidationError::MissingFrontVowel);
        }
        // c, g, ng -> Cấm nguyên âm ngay sau thuộc tập FRONT
        if o_flags.intersects(PhonotacticFlags::ONSET_FORBIDS_FRONT)
            && first_flags.intersects(PhonotacticFlags::VOWEL_FRONT)
        {
            return Err(ValidationError::ForbiddenFrontVowel);
        }

        // ---------------------------------------------------------------------
        // Luật 2: Ràng buộc phụ âm Qu (xét nguyên âm ĐẦU)
        // ---------------------------------------------------------------------
        // Qu đã chứa sẵn âm đệm /w/ (u), cấm kết hợp với nguyên âm 'u' tiếp theo (ví dụ: "quu")
        if o_flags.intersects(PhonotacticFlags::ONSET_LABIOVELAR)
            && first_flags.intersects(PhonotacticFlags::VOWEL_U)
        {
            return Err(ValidationError::GlideAfterQu);
        }

        let c_flags = coda.phonotactic_flags();

        // ---------------------------------------------------------------------
        // Luật 3: Thanh Nhập bắt buộc cho phụ âm cuối tắc
        // ---------------------------------------------------------------------
        // Phụ âm cuối p, t, c, ch chặn hoàn toàn dòng khí -> Bắt buộc mang thanh Sắc hoặc Nặng
        if c_flags.intersects(PhonotacticFlags::CODA_STOP) && !tone.allows_stop_coda() {
            return Err(ValidationError::EnteringToneRequired);
        }

        // ---------------------------------------------------------------------
        // Luật 4: Ràng buộc phụ âm cuối ngạc (xét nguyên âm CUỐI)
        // ---------------------------------------------------------------------
        // ch, nh chỉ đứng ngay sau các nguyên âm i, e, ê, y hoặc a thường
        if c_flags.intersects(PhonotacticFlags::CODA_PALATAL)
            && !last_flags.intersects(PhonotacticFlags::VOWEL_ALLOWS_PALATAL)
        {
            return Err(ValidationError::PalatalCodaVowelMismatch);
        }

        // ---------------------------------------------------------------------
        // Luật 5: Nguyên âm ngắn đứng ở vị trí mở (xét nguyên âm CUỐI)
        // ---------------------------------------------------------------------
        // ă, â có thời lượng phát âm cực ngắn -> Phải có phụ âm cuối đóng vần (ví dụ: "ăn", "ân")
        // Nếu đứng ở cuối nucleus mà không có coda -> Báo lỗi
        if last_flags.intersects(PhonotacticFlags::VOWEL_SHORT) && coda.is_none() {
            return Err(ValidationError::CodaRequiredForShortVowel);
        }

        Ok(())
    }
}
