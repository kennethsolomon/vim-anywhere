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
