use cocoa::base::{id, nil};
use objc::{class, msg_send, sel, sel_impl};

#[derive(Debug, Clone)]
pub struct AppInfo {
    pub bundle_id: String,
    pub name: String,
    pub pid: i32,
}

#[allow(deprecated)]
pub fn get_frontmost_app() -> Option<AppInfo> {
    unsafe {
        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        let app: id = msg_send![workspace, frontmostApplication];

        if app == nil {
            return None;
        }

        let bundle_id: id = msg_send![app, bundleIdentifier];
        let name: id = msg_send![app, localizedName];
        let pid: i32 = msg_send![app, processIdentifier];

        let bundle_id_str = if bundle_id != nil {
            let bytes: *const std::ffi::c_char = msg_send![bundle_id, UTF8String];
            if bytes.is_null() {
                String::new()
            } else {
                std::ffi::CStr::from_ptr(bytes)
                    .to_string_lossy()
                    .into_owned()
            }
        } else {
            String::new()
        };

        let name_str = if name != nil {
            let bytes: *const std::ffi::c_char = msg_send![name, UTF8String];
            if bytes.is_null() {
                String::new()
            } else {
                std::ffi::CStr::from_ptr(bytes)
                    .to_string_lossy()
                    .into_owned()
            }
        } else {
            String::new()
        };

        Some(AppInfo {
            bundle_id: bundle_id_str,
            name: name_str,
            pid,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strategy {
    Accessibility,
    Keyboard,
    Disabled,
}

pub fn default_strategy_for_app(bundle_id: &str) -> Strategy {
    match bundle_id {
        // Terminal apps — disable vim-anywhere (they have their own vim)
        "com.apple.Terminal" | "com.googlecode.iterm2" | "io.alacritty" | "com.mitchellh.ghostty" => {
            Strategy::Disabled
        }
        // Known good accessibility support
        "com.apple.TextEdit"
        | "com.apple.Notes"
        | "com.apple.Safari"
        | "com.apple.Xcode" => Strategy::Accessibility,
        // Default: try accessibility, fall back to keyboard
        _ => Strategy::Accessibility,
    }
}
