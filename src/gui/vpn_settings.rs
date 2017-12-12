#![allow(dead_code)]
use std::ffi::{CStr, CString};

use libc;

pub type NMSettingVpn = libc::c_void;
pub type NMConnection = libc::c_void;

pub struct VpnSettings(*mut NMSettingVpn);

pub const WG_P2P_VPN_LOCAL_NAME: &str = "local-name";
pub const WG_P2P_VPN_LOCAL_PORT: &str = "local-port";
pub const WG_P2P_VPN_ENDPOINT_ADDRESS: &str = "endpoint-address";
pub const WG_P2P_VPN_ENDPOINT_METHOD: &str = "endpoint-method";
pub const WG_P2P_VPN_REMOTE_PUBLIC_KEY: &str = "remote-public-key";
pub const WG_P2P_VPN_LOCAL_PRIVATE_KEY: &str = "local-private-key";
pub const WG_P2P_VPN_INTERFACE_NAME: &str = "interface-name";

extern {
    fn nm_connection_get_setting_vpn(connection: *mut NMConnection) -> *mut NMSettingVpn;
    fn nm_setting_vpn_get_data_item(settings: *mut NMSettingVpn, key: *const i8) -> *const i8;
    fn nm_setting_vpn_add_data_item(settings: *mut NMSettingVpn, key: *const i8, value: *const i8) -> *const i8;
    fn nm_setting_vpn_get_secret(settings: *mut NMSettingVpn, key: *const i8) -> *const i8;
    fn nm_setting_vpn_add_secret(settings: *mut NMSettingVpn, key: *const i8, value: *const i8) -> *const i8;

    fn nm_setting_set_secret_flags(settings: *mut NMSettingVpn, key: *const i8, flags: u16, err: *mut *mut u8) -> u8;
}

impl VpnSettings {
    pub fn new(conn: *mut NMConnection) -> VpnSettings {
        assert!(!conn.is_null());
        let ptr = unsafe {
            nm_connection_get_setting_vpn(conn)
        };

        VpnSettings(ptr)
    }

    pub fn as_ptr(&self) -> *mut NMSettingVpn {
        self.0
    }

    pub fn get_data_item(&self, key: &str) -> Option<String> {
        let key = CString::new(key).unwrap();
        unsafe {
            let ptr = nm_setting_vpn_get_data_item(self.0, key.as_ptr());
            if ptr.is_null() {
                None
            } else {
                Some(CStr::from_ptr(ptr).to_string_lossy().into_owned())
            }
        }
    }

    pub fn add_data_item(&mut self, key: &str, value: &str) {
        if key.len() == 0 || value.len() == 0 {
            return;
        }

        let key = CString::new(key).unwrap();
        let value = CString::new(value).unwrap();
        unsafe {
            nm_setting_vpn_add_data_item(self.0, key.as_ptr(), value.as_ptr());
        }
    }

    pub fn get_secret_item(&self, key: &str) -> Option<String> {
        let key = CString::new(key).unwrap();
        unsafe {
            let ptr = nm_setting_vpn_get_secret(self.0, key.as_ptr());
            if ptr.is_null() {
                None
            } else {
                Some(CStr::from_ptr(ptr).to_string_lossy().into_owned())
            }
        }
    }

    pub fn add_secret_item(&mut self, key: &str, value: &str) {
        if key.len() == 0 || value.len() == 0 {
            return;
        }

        let key = CString::new(key).unwrap();
        let value = CString::new(value).unwrap();
        unsafe {
            nm_setting_vpn_add_secret(self.0, key.as_ptr(), value.as_ptr());

            let r = nm_setting_set_secret_flags(self.0, key.as_ptr(), 1, 0 as *mut _);
            assert!(r != 0);
        }
    }
}
