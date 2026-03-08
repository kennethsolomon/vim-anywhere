use core_foundation::base::{CFRelease, TCFType};
use core_foundation::string::CFString;
use std::ffi::c_void;
use std::ptr;

// AXUIElement FFI bindings
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXUIElementCreateSystemWide() -> *mut c_void;
    fn AXUIElementCreateApplication(pid: i32) -> *mut c_void;
    fn AXUIElementCopyAttributeValue(
        element: *mut c_void,
        attribute: *const c_void,
        value: *mut *mut c_void,
    ) -> i32;
    fn AXUIElementSetAttributeValue(
        element: *mut c_void,
        attribute: *const c_void,
        value: *const c_void,
    ) -> i32;
    fn AXIsProcessTrusted() -> bool;
}

const K_AX_ERROR_SUCCESS: i32 = 0;

pub fn is_accessibility_trusted() -> bool {
    unsafe { AXIsProcessTrusted() }
}

pub fn get_focused_element() -> Option<*mut c_void> {
    unsafe {
        let system_wide = AXUIElementCreateSystemWide();
        if system_wide.is_null() {
            return None;
        }

        let attr = CFString::new("AXFocusedApplication");
        let mut app_ref: *mut c_void = ptr::null_mut();
        let result =
            AXUIElementCopyAttributeValue(system_wide, attr.as_concrete_TypeRef() as _, &mut app_ref);
        CFRelease(system_wide);

        if result != K_AX_ERROR_SUCCESS || app_ref.is_null() {
            return None;
        }

        let attr_focused = CFString::new("AXFocusedUIElement");
        let mut element_ref: *mut c_void = ptr::null_mut();
        let result = AXUIElementCopyAttributeValue(
            app_ref,
            attr_focused.as_concrete_TypeRef() as _,
            &mut element_ref,
        );
        CFRelease(app_ref);

        if result != K_AX_ERROR_SUCCESS || element_ref.is_null() {
            return None;
        }

        Some(element_ref)
    }
}

pub fn get_ax_attribute_string(element: *mut c_void, attribute: &str) -> Option<String> {
    unsafe {
        let attr = CFString::new(attribute);
        let mut value: *mut c_void = ptr::null_mut();
        let result =
            AXUIElementCopyAttributeValue(element, attr.as_concrete_TypeRef() as _, &mut value);

        if result != K_AX_ERROR_SUCCESS || value.is_null() {
            return None;
        }

        let cf_str = CFString::wrap_under_get_rule(value as _);
        Some(cf_str.to_string())
    }
}

pub fn get_ax_value(element: *mut c_void) -> Option<String> {
    get_ax_attribute_string(element, "AXValue")
}

pub fn get_ax_role(element: *mut c_void) -> Option<String> {
    get_ax_attribute_string(element, "AXRole")
}

pub fn check_ax_support(element: *mut c_void) -> AxSupport {
    let role = get_ax_role(element);
    let value = get_ax_value(element);

    AxSupport {
        has_role: role.is_some(),
        role: role.unwrap_or_default(),
        has_value: value.is_some(),
        has_selected_range: get_ax_attribute_string(element, "AXSelectedTextRange").is_some(),
        has_number_of_chars: get_ax_attribute_string(element, "AXNumberOfCharacters").is_some(),
    }
}

#[derive(Debug, Clone)]
pub struct AxSupport {
    pub has_role: bool,
    pub role: String,
    pub has_value: bool,
    pub has_selected_range: bool,
    pub has_number_of_chars: bool,
}

impl AxSupport {
    pub fn is_fully_supported(&self) -> bool {
        self.has_role && self.has_value && self.has_selected_range
    }

    pub fn is_text_element(&self) -> bool {
        self.role == "AXTextArea" || self.role == "AXTextField"
    }
}
