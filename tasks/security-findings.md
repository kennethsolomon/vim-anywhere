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
