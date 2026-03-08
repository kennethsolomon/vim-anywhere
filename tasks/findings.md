# Findings — 2026-03-09 — Bug Fixes: Seamless Mode Management

## Problem Statement

vim-anywhere has friction caused by 3 bugs and 1 UX gap:

1. **Green focus border is wrong size/position** — Uses `PhysicalPosition`/`PhysicalSize` for AX-returned logical coordinates on Retina displays, causing the border to render at half-size and wrong position
2. **Escape transitions to Normal in non-text contexts** — Insert mode Escape handler doesn't check if focused element is a text field, causing Normal mode to activate on web page body, buttons, etc.
3. **Block cursor fails on non-text elements** — After Escape→Normal, tries to set 1-char AX selection on non-text elements, fails silently → thin cursor with no visual feedback
4. **Mode stays stale when clicking away from text fields** — Auto-reset to Insert only triggers on next keypress, not on focus change. User sees "NORMAL" overlay when not in any text field.

## Chosen Approach: B — Context-Aware Mode Management

### Fix 1: Retina Border Coordinates
- **`lib.rs:781-782`** — Change `PhysicalPosition::new(x as i32, y as i32)` → `LogicalPosition::new(x, y)`
- Change `PhysicalSize::new(w as u32, h as u32)` → `LogicalSize::new(w, h)`
- AX API returns logical coordinates (points); overlay at line 184 already uses `LogicalPosition` correctly

### Fix 2: Context-Aware Escape in Insert Mode
- **`lib.rs:856-887`** — Before processing Escape in Insert mode, check if focused element is an editable text field
- If NOT editable: pass Escape through (return false), do NOT change mode
- This prevents Normal mode from activating on non-text elements (web page body, buttons, etc.)

### Fix 3: Guard Block Cursor + Overlay/Border
- **`lib.rs:872-878`** — Skip block cursor setup if element is not editable
- Skip overlay/border display if not in an editable text field
- This is defense-in-depth; Fix 2 should prevent this path entirely

### Fix 4: Auto-Reset Mode on Focus Loss
- When in Normal/Visual mode and the focused element is NOT editable → immediately reset to Insert and hide overlay/border
- Currently the editability check at lines 899-907 returns `false` (pass through) but does NOT reset the engine mode — it should
- After the `!is_editable` check: call `eng.reset_to_insert()` + `notify_mode(Insert)` before returning false
- This ensures: clicking away from text field = vim-anywhere becomes invisible

### Fix 5: Auto-Reset on App Switch
- When app switches (detected via bundle_id change), reset to Insert mode
- Currently auto-reset only checks element change within same app
- Extend the focus-change detection to also reset when the app changes and we're not in a text field

## Key Decisions

| Decision | Rationale |
|----------|-----------|
| LogicalPosition for border | AX API returns points, not pixels. Matches overlay behavior. |
| Check editability before Escape→Normal | Normal mode should only activate in text fields |
| Auto-reset on focus loss | Eliminates stale mode state entirely |
| Keep 1-char selection block cursor | Works in most apps; unreliable cases handled by overlay indicator |
| Don't add cursor-following overlay | Too complex for this iteration; screen-corner overlay is sufficient |

## Open Questions

- None — all fixes are straightforward with clear locations in the code.

## Files to Modify

| File | Changes |
|------|---------|
| `ui/src-tauri/src/lib.rs` | Fixes 1-5: border coords, escape context check, auto-reset on focus loss |

## Constraints from Lessons/Security

- No lessons.md entries active
- Security audit clean (0 findings) — maintain CSP, RAII AXElement, textContent for DOM
