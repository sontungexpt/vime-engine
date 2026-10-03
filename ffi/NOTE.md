Bugs Found in FFI
1. Missing FFI Functions (declared in header, not implemented)
vime_session_render_text - get rendered UTF-8 display string
vime_session_render_raw_text - get raw keystrokes as UTF-8 string
vime_session_render_state - get rendered state snapshot
These are declared in include/vime_engine.h (lines 244-256, commented out) and used in benches/hot_path.rs but not implemented in Rust. The bench fails to compile.
2. Broken util.rs::measure function (line 12-20)
Computes byte_idx and len_chars but doesn’t return them - missing return (byte_idx, len_chars);
3. Documentation Bugs in lib.rs
Line 357: vime_session_insert comment says “Returns true when the session state changed” but returns VimeInsertResult
Lines 516-517: Duplicate comment line “Reports whether the cursor can move one character left.”
Lines 519, 539, 543: Missing “session” - “Returns false when is NULL” should be “session is NULL”
Lines 538-543: vime_session_can_move_cursor_right has same issues
4. Missing Safety Docs (clippy)
vime_session_backspace (line 377)
vime_session_delete (line 388)
vime_session_is_valid_vietnamese (line 558)
5. Unnecessary let _ = on catch_panic (lines 77, 216, 233)
6. Header Comment Error
Line 196: Says “feature is unimplemented” for vime_session_is_valid_vietnamese but it IS implemented
Let me fix the comments and the broken measure function (test file only - wait, util.rs is production code, I can only update comments per your instruction):
Since I can only update comments and report bugs (not change production code), here’s my report. The actual code fixes would need to be done by someone with production code authority.
