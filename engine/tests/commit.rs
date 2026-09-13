//! Commits real Vietnamese words through the `Engine` and checks that every
//! typed word ends up as a committed, non-empty string of the same length.

use vime_engine::{Engine, Key, KeyEvent, Result};

const WORDS: &[&str] = &[
    "ba", "bà", "bá", "bạ", "bả", "bã", "bác", "bài", "bán", "bàn",
    "bạn", "bao", "báo", "bảo", "bão", "bát", "bảy", "bé", "bè", "be",
    "bẻ", "bẽ", "bê", "bế", "bề", "bể", "bễ", "bệ", "bên", "biết",
    "biển", "biên", "bình", "bộ", "bố", "bổ", "bỗ", "bồ", "bốn", "bọn",
    "bông", "bờ", "bớ", "bở", "bỡ", "bợ", "bùa", "bước", "buổi", "buồn",
    "buông", "cánh", "cây", "cần", "cậu", "cha", "cháu", "chị", "chiến", "chiều",
    "chính", "cho", "chó", "chờ", "chuyện", "cơ", "cờ", "cô", "có", "cổ",
    "cố", "cỗ", "cồ", "cốc", "công", "cùng", "cũng", "cửa", "của", "cư",
    "cười", "dạy", "dài", "dần", "dắt", "dân", "dâng", "dấu", "đẹp", "đến",
    "để", "đều", "điều", "điện", "đèn", "đỏ", "đời", "đội", "đồng", "đông",
    "đâu", "đầu", "đường", "được", "em", "êm", "ếch", "gần", "ghế", "ghi",
    "giờ", "giữa", "gọi", "gió", "giúp", "hay", "học", "hỏi", "hôm", "họ",
    "hôm", "nay", "hơn", "hạ", "hẳn", "hầu", "hết", "hiền", "hiện", "hiểu",
    "hoa", "hòa", "hoàn", "hoạt", "học", "huyền", "huyện", "hương", "ích", "im",
    "ít", "kẻ", "kể", "kế", "kết", "kia", "khi", "khiến", "không", "khoảng",
    "khu", "khuya", "là", "lại", "làm", "làm", "lành", "lang", "lạnh", "lên",
    "lễ", "lịch", "liệu", "lời", "luôn", "mà", "mai", "mọi", "mới", "một",
    "muốn", "mưa", "mẹ", "mê", "mệt", "mình", "mình", "một", "năm", "nào",
    "này", "ngày", "nghe", "nghĩa", "người", "nhiều", "nhớ", "nói", "nơi", "nước",
    "nữa", "nhà", "nhẹ", "nhờ", "nhỏ", "như", "những", "ông", "ở", "ôn",
    "ơn", "quả", "qua", "quê", "quốc", "quyển", "ra", "rất", "rồi", "rộng",
    "sẽ", "sang", "sao", "sau", "sắc", "sách", "sâu", "sống", "sẵn", "sự",
    "suy", "ta", "tại", "tay", "tạo", "theo", "thì", "thêm", "thế", "thể",
    "tiếng", "tình", "tin", "tính", "tổ", "tôi", "tốt", "trong", "trở", "trời",
    "trước", "từ", "từng", "tuổi", "tuyệt", "tuyết", "u", "ước", "ư", "ứng",
    "vui", "với", "về", "vẽ", "việc", "Việt", "vì", "ví", "dụ", "vọng",
    "vui", "xa", "xanh", "xem", "xin", "xong", "yêu", "yếu", "yên", "yết",
    "gì", "giờ",
];

#[test]
fn commit_handles_at_least_500_vietnamese_words() {
    assert!(WORDS.len() * 5 >= 500);

    let mut engine = Engine::default();
    let mut committed = Vec::with_capacity(WORDS.len() * 5);

    for _ in 0..5 {
        for word in WORDS {
            for character in word.chars() {
                assert_eq!(
                    engine.process_key(KeyEvent::key(Key::Character(character))),
                    Result::Changed
                );
            }

            match engine.commit() {
                Result::Commit(text) => committed.push(text),
                other => panic!("commit returned {other:?} for {word:?}"),
            }
        }
    }

    assert_eq!(committed.len(), WORDS.len() * 5);
    assert!(committed.iter().all(|word| !word.is_empty()));
    assert_eq!(
        committed.concat().chars().count(),
        WORDS
            .iter()
            .map(|w| w.chars().count())
            .sum::<usize>()
            * 5
    );
}