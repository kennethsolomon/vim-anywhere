# Plan — Seamless Mode Management Bug Fixes

## Goal

Fix 3 bugs and 1 UX gap in `ui/src-tauri/src/lib.rs` so vim-anywhere is invisible outside text fields, the focus border sizes correctly on Retina, and mode never gets stale.

## Plan

### Phase 1: Fix Retina Focus Border
- [x] **1.1** At ~line 781, change `PhysicalPosition::new(x as i32, y as i32)` → `LogicalPosition::new(x, y)`
- [x] **1.2** At ~line 782, change `PhysicalSize::new(w as u32, h as u32)` → `LogicalSize::new(w, h)`

### Phase 2: Context-Aware Escape in Insert Mode
- [x] **2.1** In Insert mode handler (~line 856), after detecting Escape, query `get_focused_element()` and check editability (same logic as Normal mode ~line 899-907)
- [x] **2.2** If NOT editable: `return false` (pass Escape through, stay in Insert)
- [x] **2.3** If editable: proceed with existing Escape→Normal logic unchanged

### Phase 3: Auto-Reset on Focus Loss
- [x] **3.1** At the `!is_editable` check (~line 905-907), before returning false: call `eng.reset_to_insert()` + `notify_mode(Mode::Insert)` if current mode is not Insert
- [x] **3.2** This hides the overlay + border immediately when clicking away from a text field

### Phase 4: Guard Block Cursor (defense-in-depth)
- [x] **4.1** The block cursor code (~line 872-878) already has the element from step 2.1 — only set 1-char selection if the element is confirmed editable

### Phase 5: Build & Verify
- [x] **5.1** `cargo build` — must compile
- [x] **5.2** `cargo test` — all existing tests pass
- [x] **5.3** `cargo clippy` — no new warnings

## Verification

```bash
cargo build
cargo test
cargo clippy
```

## Acceptance Criteria

1. Focus border matches the active window size/position on Retina displays
2. Pressing Escape when NOT in a text field passes through and stays in Insert mode
3. Clicking away from a text field while in Normal mode auto-resets to Insert, hides overlay + border
4. Block cursor only attempted on editable text fields
5. All existing tests pass, no new clippy warnings

## Risks / Unknowns

- None. All changes are in a single file (`ui/src-tauri/src/lib.rs`) with clear locations and minimal blast radius.
