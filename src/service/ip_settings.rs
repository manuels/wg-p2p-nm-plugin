#![allow(dead_code)]

use std;
use libc;

use std::net::IpAddr;

use vpn_settings::NMConnection;

extern "C" {
    fn nm_connection_get_setting_ip4_config(conn: *mut NMConnection) -> *mut i8;

    fn nm_setting_ip_config_get_num_addresses(settings: *mut i8) -> libc::c_uint;
    fn nm_setting_ip_config_get_address(settings: *mut i8, i: libc::c_uint) -> *mut i8;
    fn nm_ip_address_get_address(ip: *mut i8) -> *mut i8;
    fn nm_ip_address_get_prefix(ip: *mut i8) -> libc::c_uint;
}

pub struct IpV4Config(*mut i8);

impl IpV4Config {
    pub fn new(conn: *mut NMConnection) -> IpV4Config {
        let ptr = unsafe {
            nm_connection_get_setting_ip4_config(conn) as _
        };
        IpV4Config(ptr)
    }
}

impl IpConfigExt for IpV4Config {
    fn as_ptr(&self) -> *mut i8 {
        self.0
    }
}

pub trait IpConfigExt {
    fn as_ptr(&self) -> *mut i8;

    fn get_num_addresses(&self) -> usize {
        unsafe {
            nm_setting_ip_config_get_num_addresses(self.as_ptr()) as _
        }
    }

    fn get_address(&self, i: usize) -> Result<Option<IpAddr>, std::net::AddrParseError> {
        let addr = unsafe {
            let ip = nm_setting_ip_config_get_address(self.as_ptr(), i as _);
            if ip.is_null() {
                return Ok(None);
            };
            let addr = nm_ip_address_get_address(ip);
            std::ffi::CStr::from_ptr(addr)
        };
        Ok(Some(addr.to_string_lossy().parse()?))
    }

    fn get_prefix(&self, i: usize) -> Option<i8> {
        unsafe {
            let ip = nm_setting_ip_config_get_address(self.as_ptr(), i as _);
            if ip.is_null() {
                return None;
            };
            Some(nm_ip_address_get_prefix(ip) as _)
        }
    }
}
/*
    char buf[100];
//    void *s = nm_connection_get_setting_vpn(connection);
  //  sprintf(buf,"items=%d\n\0", nm_setting_vpn_get_num_data_items(s));
    void *s = nm_connection_get_setting_ip4_config(connection);
    sprintf(buf,"items=%d\n\0", nm_setting_ip_config_get_num_addresses(s));
    log(fd, buf);
    void *ip = nm_setting_ip_config_get_address(s, 0);
    char *m = nm_ip_address_get_address (ip);
    log(fd, m);
    fclose(fd);
*/
