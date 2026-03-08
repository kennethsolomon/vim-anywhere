use core_foundation::base::{CFRelease, TCFType};
use core_foundation::string::CFString;
use std::ffi::c_void;
use std::ptr;

// AXUIElement FFI bindings
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXUIElementCreateSystemWide() -> *mut c_void;
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
    fn AXUIElementIsAttributeSettable(
        element: *mut c_void,
        attribute: *const c_void,
        settable: *mut bool,
    ) -> i32;
    fn AXIsProcessTrusted() -> bool;
}

// CFRange-based AXValue
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXValueCreate(value_type: u32, value_ptr: *const c_void) -> *const c_void;
    fn AXValueGetValue(value: *const c_void, value_type: u32, value_ptr: *mut c_void) -> bool;
}

const K_AX_VALUE_TYPE_CG_POINT: u32 = 1;
const K_AX_VALUE_TYPE_CG_SIZE: u32 = 2;
const K_AX_VALUE_TYPE_CF_RANGE: u32 = 4;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct CGPoint {
    x: f64,
    y: f64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct CGSize {
    width: f64,
    height: f64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CFRange {
    pub location: i64,
    pub length: i64,
}

const K_AX_ERROR_SUCCESS: i32 = 0;

/// RAII wrapper for AXUIElement pointers. Calls CFRelease on drop.
pub struct AXElement {
    ptr: *mut c_void,
}

impl AXElement {
    /// Takes ownership of a CF object pointer (assumes +1 retain count).
    pub fn from_owned(ptr: *mut c_void) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Self { ptr })
        }
    }

    pub fn as_ptr(&self) -> *mut c_void {
        self.ptr
    }
}

impl Drop for AXElement {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { CFRelease(self.ptr) };
        }
    }
}

pub fn is_accessibility_trusted() -> bool {
    unsafe { AXIsProcessTrusted() }
}

pub fn get_focused_element() -> Option<AXElement> {
    unsafe {
        let system_wide = AXUIElementCreateSystemWide();
        let system_guard = AXElement::from_owned(system_wide)?;

        let attr = CFString::new("AXFocusedApplication");
        let mut app_ref: *mut c_void = ptr::null_mut();
        let result = AXUIElementCopyAttributeValue(
            system_guard.as_ptr(),
            attr.as_concrete_TypeRef() as _,
            &mut app_ref,
        );

        if result != K_AX_ERROR_SUCCESS || app_ref.is_null() {
            return None;
        }
        let app_guard = AXElement::from_owned(app_ref)?;

        let attr_focused = CFString::new("AXFocusedUIElement");
        let mut element_ref: *mut c_void = ptr::null_mut();
        let result = AXUIElementCopyAttributeValue(
            app_guard.as_ptr(),
            attr_focused.as_concrete_TypeRef() as _,
            &mut element_ref,
        );

        if result != K_AX_ERROR_SUCCESS || element_ref.is_null() {
            return None;
        }

        AXElement::from_owned(element_ref)
    }
}

pub fn get_ax_attribute_string(element: &AXElement, attribute: &str) -> Option<String> {
    unsafe {
        let attr = CFString::new(attribute);
        let mut value: *mut c_void = ptr::null_mut();
        let result = AXUIElementCopyAttributeValue(
            element.as_ptr(),
            attr.as_concrete_TypeRef() as _,
            &mut value,
        );

        if result != K_AX_ERROR_SUCCESS || value.is_null() {
            return None;
        }

        // AXUIElementCopyAttributeValue returns an owned reference (+1 retain),
        // so use wrap_under_create_rule to take ownership correctly.
        let cf_str = CFString::wrap_under_create_rule(value as _);
        Some(cf_str.to_string())
    }
}

pub fn get_ax_value(element: &AXElement) -> Option<String> {
    get_ax_attribute_string(element, "AXValue")
}

pub fn get_ax_role(element: &AXElement) -> Option<String> {
    get_ax_attribute_string(element, "AXRole")
}

pub fn check_ax_support(element: &AXElement) -> AxSupport {
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

/// Check if the focused element is an editable text field.
/// Known text input roles (AXTextArea, AXTextField, AXComboBox) are treated as
/// editable directly — AXUIElementIsAttributeSettable can report false for these
/// in some apps even though they accept input.  For other roles (e.g. AXWebArea)
/// we fall back to the settable check.
pub fn is_editable_text(element: &AXElement) -> bool {
    if let Some(role) = get_ax_role(element) {
        match role.as_str() {
            "AXTextArea" | "AXTextField" | "AXComboBox" => return true,
            _ => {}
        }
    }

    unsafe {
        let attr = CFString::new("AXValue");
        let mut settable = false;
        let result = AXUIElementIsAttributeSettable(
            element.as_ptr(),
            attr.as_concrete_TypeRef() as _,
            &mut settable,
        );
        result == K_AX_ERROR_SUCCESS && settable
    }
}

pub fn set_ax_value(element: &AXElement, text: &str) -> bool {
    unsafe {
        let attr = CFString::new("AXValue");
        let cf_value = CFString::new(text);
        let result = AXUIElementSetAttributeValue(
            element.as_ptr(),
            attr.as_concrete_TypeRef() as _,
            cf_value.as_concrete_TypeRef() as _,
        );
        result == K_AX_ERROR_SUCCESS
    }
}

pub fn get_ax_selected_range(element: &AXElement) -> Option<(usize, usize)> {
    unsafe {
        let attr = CFString::new("AXSelectedTextRange");
        let mut value: *mut c_void = ptr::null_mut();
        let result = AXUIElementCopyAttributeValue(
            element.as_ptr(),
            attr.as_concrete_TypeRef() as _,
            &mut value,
        );

        if result != K_AX_ERROR_SUCCESS || value.is_null() {
            return None;
        }

        let mut range = CFRange {
            location: 0,
            length: 0,
        };
        let ok = AXValueGetValue(
            value as _,
            K_AX_VALUE_TYPE_CF_RANGE,
            &mut range as *mut CFRange as *mut c_void,
        );
        CFRelease(value);

        if ok {
            Some((range.location as usize, range.length as usize))
        } else {
            None
        }
    }
}

pub fn set_ax_selected_range(element: &AXElement, location: usize, length: usize) -> bool {
    unsafe {
        let range = CFRange {
            location: location as i64,
            length: length as i64,
        };
        let ax_value = AXValueCreate(
            K_AX_VALUE_TYPE_CF_RANGE,
            &range as *const CFRange as *const c_void,
        );
        if ax_value.is_null() {
            return false;
        }

        let attr = CFString::new("AXSelectedTextRange");
        let result = AXUIElementSetAttributeValue(
            element.as_ptr(),
            attr.as_concrete_TypeRef() as _,
            ax_value,
        );
        CFRelease(ax_value);
        result == K_AX_ERROR_SUCCESS
    }
}

/// Get the frame (x, y, width, height) of the window containing the focused element.
/// Traverses up via AXWindow attribute.
pub fn get_focused_window_frame() -> Option<(f64, f64, f64, f64)> {
    let element = get_focused_element()?;
    unsafe {
        // Get the AXWindow from the focused element
        let attr = CFString::new("AXWindow");
        let mut window_ref: *mut c_void = ptr::null_mut();
        let result = AXUIElementCopyAttributeValue(
            element.as_ptr(),
            attr.as_concrete_TypeRef() as _,
            &mut window_ref,
        );
        if result != K_AX_ERROR_SUCCESS || window_ref.is_null() {
            return None;
        }
        let window = AXElement::from_owned(window_ref)?;

        // Get AXPosition (CGPoint)
        let pos_attr = CFString::new("AXPosition");
        let mut pos_value: *mut c_void = ptr::null_mut();
        let result = AXUIElementCopyAttributeValue(
            window.as_ptr(),
            pos_attr.as_concrete_TypeRef() as _,
            &mut pos_value,
        );
        if result != K_AX_ERROR_SUCCESS || pos_value.is_null() {
            return None;
        }
        let mut point = CGPoint { x: 0.0, y: 0.0 };
        let ok = AXValueGetValue(
            pos_value as _,
            K_AX_VALUE_TYPE_CG_POINT,
            &mut point as *mut CGPoint as *mut c_void,
        );
        CFRelease(pos_value);
        if !ok {
            return None;
        }

        // Get AXSize (CGSize)
        let size_attr = CFString::new("AXSize");
        let mut size_value: *mut c_void = ptr::null_mut();
        let result = AXUIElementCopyAttributeValue(
            window.as_ptr(),
            size_attr.as_concrete_TypeRef() as _,
            &mut size_value,
        );
        if result != K_AX_ERROR_SUCCESS || size_value.is_null() {
            return None;
        }
        let mut size = CGSize { width: 0.0, height: 0.0 };
        let ok = AXValueGetValue(
            size_value as _,
            K_AX_VALUE_TYPE_CG_SIZE,
            &mut size as *mut CGSize as *mut c_void,
        );
        CFRelease(size_value);
        if !ok {
            return None;
        }

        Some((point.x, point.y, size.width, size.height))
    }
}

/// Convert a (line, col) cursor position to a character offset in text.
pub fn cursor_to_offset(text: &str, line: usize, col: usize) -> usize {
    if text.is_empty() {
        return 0;
    }
    let mut offset = 0;
    let mut line_idx = 0;
    for chunk in text.split('\n') {
        if line_idx == line {
            return offset + col.min(chunk.len());
        }
        offset += chunk.len() + 1; // +1 for the \n
        line_idx += 1;
    }
    // If line is past the end, return end of text
    text.len()
}

/// Convert a character offset back to (line, col).
pub fn offset_to_cursor(text: &str, offset: usize) -> (usize, usize) {
    if text.is_empty() {
        return (0, 0);
    }
    let clamped = offset.min(text.len());
    let mut remaining = clamped;
    let mut line_idx = 0;
    for chunk in text.split('\n') {
        if remaining <= chunk.len() {
            return (line_idx, remaining);
        }
        remaining -= chunk.len() + 1; // +1 for \n
        line_idx += 1;
    }
    // Fallback: last line, last col
    let last_line_len = text.split('\n').last().map(|l| l.len()).unwrap_or(0);
    (line_idx.saturating_sub(1), last_line_len)
}
