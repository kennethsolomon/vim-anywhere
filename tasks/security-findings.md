# Security Findings

> Populated by `/security-check`. Never overwritten — new audits append below.
> Referenced by `/review`, `/finish-feature`, and `/brainstorm` for security context.

# Security Audit — 2026-03-08

**Scope:** Full project scan (all commits on `main`)
**Stack:** Rust / Tauri / HTML+JS
**Files audited:** 16 source files

## Critical (must fix before deploy)

_None found._

## High (fix before production)

- **[ui/src-tauri/tauri.conf.json:22]** Content Security Policy is disabled (`"csp": null`)
  **Standard:** OWASP A05 — Security Misconfiguration (CWE-1021)
  **Risk:** Without CSP, the webview is vulnerable to XSS if any user-controlled content is rendered. While the current UI is static HTML, future features (custom mappings display, app names) could introduce injection vectors.
  **Recommendation:** Set a restrictive CSP, e.g. `"csp": "default-src 'self'; style-src 'self' 'unsafe-inline'; script-src 'self'"`. The `'unsafe-inline'` for styles is needed for inline styles in HTML.

- **[crates/platform-mac/src/accessibility.rs:60]** `get_focused_element()` returns raw `*mut c_void` without ownership tracking — caller must manually release via `CFRelease`
  **Standard:** CWE-401 — Missing Release of Memory After Effective Lifetime
  **Risk:** Memory leak per focus-element query. In a tight event loop (every keystroke), this could accumulate. Also, double-free is possible if caller releases incorrectly.
  **Recommendation:** Wrap the returned pointer in a RAII type (e.g. a `struct AXElement(*mut c_void)` with `Drop` impl calling `CFRelease`). Alternatively, document the ownership contract clearly and add `CFRelease` at call sites.

## Medium (should fix)

- **[crates/platform-mac/src/accessibility.rs:75]** `CFString::wrap_under_get_rule` used on a value obtained via `AXUIElementCopyAttributeValue` — ownership mismatch
  **Standard:** CWE-416 — Use After Free
  **Risk:** `Copy` functions in CoreFoundation return owned references (retain count +1), but `wrap_under_get_rule` assumes a borrowed reference (doesn't retain). This could cause use-after-free or double-free depending on when the CFString is dropped.
  **Recommendation:** Use `CFString::wrap_under_create_rule(value as _)` instead, which correctly takes ownership of the +1 reference.

- **[crates/core/src/config.rs:96-98]** `Config::load()` silently falls back to defaults on any read error, including permission errors or corrupt files
  **Standard:** CWE-392 — Missing Report of Error Condition
  **Risk:** User's config could be silently ignored (e.g. corrupted file, wrong permissions). User thinks their settings are applied but they're running defaults.
  **Recommendation:** Log a warning when file exists but fails to parse. Consider returning `Result` so callers can handle errors explicitly.

- **[crates/core/src/config.rs:86-92]** `Config::config_path()` uses `HOME` env var without validation
  **Standard:** CWE-426 — Untrusted Search Path
  **Risk:** If `HOME` is manipulated, config could be read from or written to an unexpected location. Low risk for a desktop app, but worth hardening.
  **Recommendation:** Use `dirs::config_dir()` or `std::env::home_dir()` (with fallback) for more robust home directory detection.

- **[.gitignore:1-13]** Missing entries for common sensitive files
  **Standard:** CWE-200 — Exposure of Sensitive Information
  **Risk:** `.env`, `*.pem`, `*.key`, credentials files could be accidentally committed.
  **Recommendation:** Add `.env*`, `*.pem`, `*.key`, `*.p12`, `credentials.*` to `.gitignore`.

## Low / Informational

- **[ui/src-tauri/src/lib.rs:3-5]** Unused `greet` Tauri command exposes a minor attack surface
  **Standard:** OWASP A05 — Security Misconfiguration
  **Recommendation:** Remove the `greet` command; it's scaffolding that serves no purpose and slightly increases the IPC surface.

- **[crates/platform-mac/src/keyboard.rs:108]** Unknown keycodes map to `Key::Char('?')` silently
  **Standard:** CWE-392 — Missing Report of Error Condition
  **Recommendation:** Consider returning an `Unknown(u16)` variant or logging unrecognized keycodes for debugging.

- **[crates/platform-mac/src/keyboard.rs:166-178]** `send_key_event` silently ignores errors from `CGEventSource::new` and `CGEvent::new_keyboard_event`
  **Standard:** CWE-252 — Unchecked Return Value
  **Recommendation:** Log failures; silent drops make debugging keystroke issues very difficult.

- **[src/lib.rs:497-511]** `toggle_case` uses single-char `to_lowercase`/`to_uppercase` — may produce multi-char results for certain Unicode characters (e.g. German eszett 'ß' uppercases to "SS")
  **Standard:** CWE-176 — Improper Handling of Unicode Encoding
  **Recommendation:** Use `.next().unwrap_or(c)` pattern (already done), which truncates. Acceptable for Vim behavior, but worth documenting.

## Passed Checks

- **A01 Broken Access Control** — No auth system; local desktop app with OS-level permissions (Accessibility/Input Monitoring). Appropriate.
- **A02 Cryptographic Failures** — No cryptography used. No secrets stored. Config is plaintext preferences (appropriate for non-sensitive settings).
- **A03 Injection** — No SQL, no shell commands, no template rendering with user input. HTML is static. Tauri IPC uses typed commands.
- **A04 Insecure Design** — CGEvent tap requires explicit OS permission. App correctly disables itself in terminal emulators.
- **A07 Auth Failures** — N/A (no authentication)
- **A08 Data Integrity** — Cargo.lock committed. No deserialization of untrusted external data (config is user's own file).
- **A10 SSRF** — No network requests made.
- **Dependency hygiene** — Lock files committed. No wildcard versions. Dependencies are well-known crates.

## Summary

| Severity | Count |
|----------|-------|
| Critical | 0 |
| High     | 2 |
| Medium   | 3 |
| Low      | 4 |
| **Total** | **9** |

---

# Security Audit — 2026-03-09

**Scope:** Changed files since `9c613a4` (seamless integration feature — 7 commits)
**Stack:** Rust / Tauri 2 / HTML+JS
**Files audited:** 18 source files (excluding docs/tasks)

## Prior Findings Status

- **HIGH CSP disabled** — FIXED (CSP now set in tauri.conf.json:23)
- **HIGH AXElement memory** — FIXED (RAII wrapper with `Drop` impl at accessibility.rs:62-88)
- **MEDIUM CFString ownership** — FIXED (`wrap_under_create_rule` at accessibility.rs:144)
- **MEDIUM Config::load() silent fallback** — FIXED (eprintln warnings at config.rs:137-145)
- **MEDIUM HOME env validation** — FIXED (`std::env::home_dir()` with fallback at config.rs:122-124)

## Critical (must fix before deploy)

_None found._

## High (fix before production)

_None found._

## Medium (should fix)

- **[ui/src/main.js:441-444]** `innerHTML` with interpolated `app` value in `renderExcludedApps()`
  **Standard:** OWASP A03 — Injection / XSS (CWE-79)
  **Risk:** The `app` value comes from `config.excluded_apps` (user's own config file, loaded via Tauri IPC). If a bundle ID contained HTML/JS (e.g. `<img onerror=...>`), it would execute in the webview. Practically low risk since the data source is the user's own config, but it violates defense-in-depth.
  **Recommendation:** Use `textContent` or `document.createTextNode()` instead of string interpolation in `innerHTML`. Or sanitize with a simple escape function.

- **[src/lib.rs:62-68]** Debug logging to `/tmp/vim-anywhere.log` writes every keystroke in Normal/Visual mode
  **Standard:** CWE-532 — Insertion of Sensitive Information into Log File (OWASP A09)
  **Risk:** The log records every key event with key value and modifiers. In a shared system, `/tmp/vim-anywhere.log` is world-readable. This could leak sensitive text content being edited.
  **Recommendation:** Remove debug logging before production release, or gate behind a `DEBUG` env var, and use a user-private path (e.g. `~/.local/share/vim-anywhere/debug.log` with 0600 permissions).

- **[ui/src-tauri/src/lib.rs:760,921,1032]** Same debug logging issue in Tauri event handler — writes AX role, editable status, and event details to `/tmp/vim-anywhere.log`
  **Standard:** CWE-532 — Insertion of Sensitive Information into Log File (OWASP A09)
  **Risk:** Same as above — keystroke and AX element data in world-readable temp file.
  **Recommendation:** Same as above.

## Low / Informational

- **[ui/src/onboarding.js:21]** Empty `catch {}` block silently swallows permission check errors
  **Standard:** CWE-392 — Missing Report of Error Condition
  **Recommendation:** Log the error to console for debugging: `catch (e) { console.warn("Permission check failed:", e); }`

- **[ui/src-tauri/src/lib.rs:305-311]** `set_excluded_app` accepts arbitrary string as bundle ID with no validation
  **Standard:** CWE-20 — Improper Input Validation
  **Risk:** Very low — the value is only stored in config and compared against app bundle IDs. No path traversal or injection possible. But extremely long strings or special characters could bloat the config file.
  **Recommendation:** Consider basic validation: non-empty, max length, reverse-domain format check.

- **[crates/platform-mac/src/accessibility.rs:287-349]** `get_focused_window_frame()` calls `get_focused_element()` internally, which duplicates the AX query already made in the event handler
  **Standard:** Performance / CWE-400
  **Recommendation:** Accept an `&AXElement` parameter instead of re-querying, to avoid redundant AX API calls on every mode transition.

## Passed Checks

- **A01 Broken Access Control** — OS-level permissions (Accessibility/Input Monitoring) gate all functionality. Appropriate.
- **A02 Cryptographic Failures** — No cryptography. No secrets. Config is plaintext preferences.
- **A03 Injection (shell)** — `open_accessibility_settings` and `open_input_monitoring_settings` use hardcoded URLs with `std::process::Command::new("open")` — no user input in the URL. Safe.
- **A04 Insecure Design** — Terminal exclusion list prevents interception in sensitive apps. Focus-change auto-reset prevents stale mode state.
- **A05 Security Misconfiguration** — CSP is now properly set. New windows (dim-overlay, focus-border, onboarding) are created with appropriate properties (click-through, no focus).
- **A08 Data Integrity** — `complete_onboarding`, `set_excluded_app`, `remove_excluded_app` all write through `Config::save()` with proper serialization. No untrusted deserialization.
- **A10 SSRF** — No network requests in new code.
- **Memory safety** — New `get_focused_window_frame()` properly wraps `window_ref` in `AXElement` (RAII), and explicitly `CFRelease`s AXValue pointers for position/size. No leaks.
- **Onboarding flow** — Permission URLs are hardcoded (`x-apple.systempreferences:...`), not user-controlled. Safe.

## Summary

| Severity | Count |
|----------|-------|
| Critical | 0 |
| High     | 0 |
| Medium   | 3 |
| Low      | 3 |
| **Total** | **6** |

---

# Security Audit — 2026-03-09 (Re-check)

**Scope:** Security fix commit `04ae388` — verification that all 6 prior findings are resolved
**Stack:** Rust / Tauri 2 / HTML+JS
**Files audited:** 5 source files (changed in security fix commit)

## Prior Findings Status (Audit 2 — all 6)

- **MEDIUM XSS in renderExcludedApps()** — FIXED (DOM API with `textContent` at main.js:441-444)
- **MEDIUM Debug logging src/lib.rs** — FIXED (all `/tmp/vim-anywhere.log` writes removed; no `/tmp/` references remain in .rs files)
- **MEDIUM Debug logging ui/src-tauri/src/lib.rs** — FIXED (all 3 log blocks removed, `use std::io::Write` import removed)
- **LOW Empty catch in onboarding.js** — FIXED (`console.warn("Permission check failed:", e)` at onboarding.js:21)
- **LOW set_excluded_app no validation** — FIXED (non-empty, max 255 chars, ASCII alphanumeric + `.` `-` `_` at lib.rs:305-311)
- **LOW Redundant AX query** — FIXED (new `get_window_frame(&AXElement)` public API at accessibility.rs:292-356; convenience wrapper `get_focused_window_frame()` preserved at accessibility.rs:285-290)

## Critical (must fix before deploy)

_None found._

## High (fix before production)

_None found._

## Medium (should fix)

_None found._

## Low / Informational

_None found._

## Passed Checks

- **A01 Broken Access Control** — OS-level permissions gate all functionality.
- **A02 Cryptographic Failures** — No cryptography or secrets.
- **A03 Injection** — `renderExcludedApps` now uses `textContent` (no XSS). Remaining `innerHTML` usages in `renderMappings` and `renderAppRows` interpolate data from user's own config and macOS NSWorkspace — same trust boundary, and CSP blocks inline script execution.
- **A05 Security Misconfiguration** — CSP active. No debug logging. No unnecessary commands.
- **A09 Logging** — No sensitive data logged. Debug log to `/tmp/` fully removed. Only `eprintln` for startup errors (no PII).
- **A10 SSRF** — No network requests.
- **Input validation** — `set_excluded_app` validates bundle ID format. Config deserialization uses typed serde.
- **Memory safety** — RAII `AXElement` wrapper, correct `CFRelease` on all AXValue pointers, `wrap_under_create_rule` for CFString ownership.

## Summary

| Severity | Count |
|----------|-------|
| Critical | 0 |
| High     | 0 |
| Medium   | 0 |
| Low      | 0 |
| **Total** | **0** |

---

# Security Audit — 2026-03-09 (Bug Fix Commits)

**Scope:** 4 commits on `feature/seamless-integration` since last audit (`835213f`, `313bfa3`, `723ed0b`, `787be6d`)
**Stack:** Rust / Tauri 2 / HTML+JS
**Files audited:** 3 source files (`accessibility.rs`, `lib.rs`, `engine_comprehensive.rs`)

## Changes Reviewed

1. `LogicalPosition`/`LogicalSize` for focus border (Retina fix)
2. Context-aware Escape: editability + writability check before Insert→Normal
3. Auto-reset to Insert on non-editable focus and excluded app switch
4. `is_ax_value_settable()` new public function in accessibility.rs
5. 7 new integration tests for mode management contracts

## Critical (must fix before deploy)

_None found._

## High (fix before production)

_None found._

## Medium (should fix)

_None found._

## Low / Informational

_None found._

## Passed Checks

- **A01 Broken Access Control** — No new attack surface. Mode reset on app switch reduces unintended interception.
- **A02 Cryptographic Failures** — No cryptography in changed code.
- **A03 Injection** — No user input interpolation. All new checks use AX API queries with hardcoded attribute strings (`"AXValue"`, `"AXRole"`). No shell commands or string formatting with external data.
- **A04 Insecure Design** — New `is_ax_value_settable()` check prevents vim activation on elements where writes would silently fail. Defense-in-depth improvement.
- **A05 Security Misconfiguration** — No new windows, commands, or IPC endpoints added. CSP unchanged.
- **A09 Logging** — No new logging added. No PII in changed code paths.
- **A10 SSRF** — No network requests.
- **Memory safety** — `is_ax_value_settable()` follows the same safe pattern as existing `is_editable_text()`: stack-allocated `settable` bool, no heap allocation, no ownership transfer. `AXUIElementIsAttributeSettable` is a read-only query.
- **Concurrency** — Engine mutex lock in the app-switch passthrough path (`lib.rs:833`) uses `if let Ok(...)` to gracefully handle poisoned mutex. No deadlock risk: lock is acquired, used, and dropped before `return false`.
- **Logical correctness** — `notify_mode(Mode::Insert)` called inside the engine lock scope at `lib.rs:836`. `notify_mode` emits Tauri events and hides windows — these are async fire-and-forget operations that don't block the mutex.
- **Test safety** — New tests are pure unit tests with no I/O, no file access, no network. No security surface.

## Summary

| Severity | Count |
|----------|-------|
| Critical | 0 |
| High     | 0 |
| Medium   | 0 |
| Low      | 0 |
| **Total** | **0** |

---

# Security Audit — 2026-03-09 (UX Polish + Tests)

**Scope:** 3 commits on `feature/seamless-integration` — `325cafd` (feat: UX polish), `49225c9` (chore: tasks), `4ce7f4a` (test: new tests)
**Stack:** Rust / Tauri 2 / HTML+JS
**Files audited:** 12 source files

## Changes Reviewed

1. Global toggle hotkey (matches_hotkey, toggle_enabled, set_toggle_hotkey, get_enabled)
2. Toggle feedback window (toggle-feedback.html/css/js)
3. Custom mapping remapping in event tap (key_matches_mapping_from, parse_mapping_key)
4. AX failure notification window (notification.html/css/js)
5. Near-cursor mode indicator (overlay.js repositioning)
6. Settings UI updates (Global Toggle section, hotkey recording)
7. 39 new tests (config + lib.rs helpers)

## Critical (must fix before deploy)

_None found._

## High (fix before production)

_None found._

## Medium (should fix)

- **[ui/src-tauri/src/lib.rs:1338-1339]** Nested mutex acquisition: `config` lock held while acquiring `engine` lock in custom mapping remapping block
  **Standard:** CWE-833 — Deadlock
  **Risk:** The custom mapping block at line 1337-1367 acquires `config` lock, then within that scope acquires `engine` lock (line 1338-1339 via `state_for_tap.engine.lock()`). Other code paths (e.g. `save_config_full` at line 128-136) acquire `config` then `engine`. The lock order is consistent across the codebase (always config-before-engine), so no actual deadlock under current code. However, it's fragile — any future code acquiring engine-then-config would deadlock.
  **Recommendation:** Extract the engine mode read outside the config lock scope, or document the lock ordering contract explicitly.

## Low / Informational

- **[ui/src-tauri/src/lib.rs:1233-1235]** Toggle hotkey check acquires config lock, drops it, then immediately re-acquires at line 1238 if the hotkey matches. Two lock acquisitions in rapid succession.
  **Standard:** Performance / CWE-362 — TOCTOU
  **Risk:** Between the first lock (to read hotkey string) and second lock (to flip `enabled`), another thread could change the hotkey. Very low practical risk since the event tap callback runs on a single thread and config changes come from the UI thread. No security impact — at worst the toggle fires once with a stale hotkey.
  **Recommendation:** Combine into a single lock scope for clarity and performance, or accept the current pattern with a comment.

- **[ui/src/main.js:268]** `innerHTML` used for mapping add form with hardcoded HTML (no user input interpolated)
  **Standard:** OWASP A03 — XSS (CWE-79)
  **Risk:** None in current code — the innerHTML content is a static template string with no interpolated variables. CSP blocks inline script execution regardless. Flagged for awareness only.
  **Recommendation:** No action needed. The static template is safe, and CSP provides defense-in-depth.

- **[ui/src/overlay.js:86]** `setPosition()` called with values derived from event payload coordinates without bounds validation
  **Standard:** CWE-20 — Improper Input Validation
  **Risk:** The `focus-highlight-update` payload comes from the Rust backend (trusted source). The JS does clamp values to screen bounds (lines 74-83). No security risk — the overlay is a click-through, non-interactive window.
  **Recommendation:** No action needed. Current bounds clamping is sufficient.

- **[ui/src-tauri/src/lib.rs:378-381]** `set_toggle_hotkey` validates length (non-empty, max 50) but not content format
  **Standard:** CWE-20 — Improper Input Validation
  **Risk:** A malformed hotkey string (e.g. "---" or "ctrl-ctrl-ctrl") would be saved but never match any key event, effectively disabling the toggle. No crash or security risk — just a usability issue.
  **Recommendation:** Consider validating that the string has at most one key part (last segment) and recognized modifier names.

## Passed Checks

- **A01 Broken Access Control** — `toggle_enabled`, `set_toggle_hotkey`, `get_enabled` commands only modify local config. No privilege escalation possible. All commands run in the same process context as the user.
- **A02 Cryptographic Failures** — No cryptography or secrets in new code.
- **A03 Injection** — All new `innerHTML` uses are either static templates (mapping add form) or already converted to DOM API (`renderExcludedApps` uses `textContent`). `renderMappings` now uses DOM API exclusively (createElement + textContent). No XSS vectors.
- **A04 Insecure Design** — Toggle hotkey is checked before `enabled` flag, ensuring it always works. Custom mapping remapping happens once per event (no remap loops). AX failure notification uses HashSet for dedup (once per app per session).
- **A05 Security Misconfiguration** — CSP active. New windows (toggle-feedback, notification) use appropriate properties: toggle-feedback is click-through, notification is intentionally clickable for the "Exclude app" button.
- **A07 Auth Failures** — N/A (no authentication).
- **A08 Data Integrity** — `add_custom_mapping` validates: non-empty, max 10 chars, valid mode ("normal"/"insert"/"visual"). `set_toggle_hotkey` validates: non-empty, max 50 chars.
- **A09 Logging** — No debug logging in new code. No `/tmp/` writes. `eprintln` only for startup errors.
- **A10 SSRF** — No network requests in new code.
- **Memory safety** — All new code operates on Rust owned types (String, Vec, HashSet). No raw pointers or unsafe blocks in new code. Mutex usage follows existing patterns.
- **Frontend safety** — `notification.js` uses `textContent` for app name display (line 29). `toggle-feedback.js` uses `textContent` (line 18). `overlay.js` uses `textContent` (line 18). No DOM XSS.
- **Test safety** — All 39 new tests are pure unit tests with no I/O, file access, or network. No security surface.

## Summary

| Severity | Count |
|----------|-------|
| Critical | 0 |
| High     | 0 |
| Medium   | 1 |
| Low      | 4 |
| **Total** | **5** |
