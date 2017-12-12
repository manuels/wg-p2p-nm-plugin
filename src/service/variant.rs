use std::ffi::CString;

use glib_sys;
use glib::Variant;
use glib::translate::ToGlibPtr;

pub struct VariantBuilder(*mut glib_sys::GVariantBuilder);

impl VariantBuilder {
    pub fn new(typ: &str) -> Result<VariantBuilder,()> {
        let typ = CString::new(typ).unwrap();

        let ptr = unsafe {
            glib_sys::g_variant_builder_new(typ.as_ptr() as *const _)
        };

        if ptr.is_null() {
            Err(())
        } else {
            Ok(VariantBuilder(ptr))
        }
    }

    pub fn add_dict_entry(&mut self, key: &str, value: Variant) {
        let typ = CString::new("{sv}").unwrap();
        let key = CString::new(key).unwrap();

        unsafe {
            glib_sys::g_variant_builder_add(self.0,
                typ.as_ptr(),
                key.as_ptr(),
                value.to_glib_full())
        }
    }

    #[allow(dead_code)]
    pub fn add_value(&mut self, v: Variant) {
        self.add_raw_value(v.to_glib_full())
    }

    #[allow(dead_code)]
    pub fn add_raw_value(&mut self, v: *mut glib_sys::GVariant) {
        unsafe {
            glib_sys::g_variant_builder_add_value(self.0, v)
        }
    }

    pub fn to_raw_value(self) -> *mut glib_sys::GVariant {
        unsafe {
            glib_sys::g_variant_builder_end(self.0)
        }
    }
}

impl Drop for VariantBuilder {
    fn drop(&mut self) {
        unsafe {
            glib_sys::g_variant_builder_unref(self.0)
        }
    }
}

