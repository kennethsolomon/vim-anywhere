# Architectural Change: Seamless Integration

**Date:** 2026-03-09
**Type:** Feature Addition
**Commits:** 0d57889..c600381 (11 commits)

## Summary

Added multi-window overlay system, smart escape mode, auto-insert on focus change, terminal exclusion, and first-launch onboarding to make vim-anywhere feel like a native macOS feature.

## What Changed

### New Windows (Tauri Multi-Window)
- **dim-overlay**: Fullscreen transparent window with CSS clip-path cutout around the active window. Click-through, visible on all Spaces.
- **focus-border**: Positioned window matching the active app window frame, renders a glowing border. Click-through, visible on all Spaces.
- **onboarding**: Modal setup wizard for Accessibility + Input Monitoring permissions. Shown on first launch or when permissions are missing.

### Core Engine Changes
- `ModeTransition::PassThrough` variant: Smart escape passes Escape through to the app when already in Normal mode.
- `ModeStateMachine::reset_to_insert()`: Resets mode + clears pending sequence state for auto-insert on focus change.
- `ModeEntryConfig.smart_escape`: New config field controlling PassThrough behavior.

### Focus Tracking
- `last_focused_element` state tracks identity via hash of (AX role + AXDescription + app bundle ID).
- On focus change while in Normal/Visual mode, automatically resets to Insert mode.

### Window Positioning
- `set_all_spaces()` helper applies NSWindowCollectionBehavior to make windows visible across all macOS Spaces/desktops.
- `get_window_frame(&AXElement)` and `get_focused_window_frame()` retrieve the active window's frame via AXWindow attribute traversal.

## Impact

- **Event tap callback** is now significantly larger (~150 lines) — handles mode notifications, overlay show/hide, focus highlight positioning, and focus tracking in addition to the core vim engine.
- **Config** has 7 new fields: `focus_highlight`, `dim_background`, `dim_intensity`, `excluded_apps`, `onboarding_complete`, `smart_escape` (in mode_entry), and `show_overlay` position options.
- **UI** has 4 new HTML pages (overlay redesign, dim-overlay, focus-border, onboarding) with corresponding CSS/JS.

## Migration / Compatibility

- Fully backwards compatible. New config fields have serde defaults.
- Existing users get smart escape enabled by default (single Escape works naturally).
- Terminal emulators are excluded by default via `default_excluded_apps()`.
