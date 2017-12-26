#![allow(dead_code)]

use std;
use libc;

use std::net::IpAddr;

use vpn_settings::NMConnection;

extern "C" {
    fn nm_connection_get_setting_ip4_config(conn: *mut NMConnection) -> *mut i8;
    fn nm_connection_get_setting_ip6_config(conn: *mut NMConnection) -> *mut i8;

    fn nm_setting_ip_config_get_num_routes (settings: *mut i8) -> libc::c_uint;
    fn nm_setting_ip_config_get_num_addresses(settings: *mut i8) -> libc::c_uint;
    fn nm_setting_ip_config_get_address(settings: *mut i8, i: usize) -> *mut i8;
    fn nm_setting_ip_config_get_gateway(settings: *mut i8) -> *const i8;
    fn nm_ip_address_get_address(ip: *mut i8) -> *const i8;
    fn nm_ip_address_get_prefix(ip: *const i8) -> libc::c_uint;

    fn nm_setting_ip_config_get_route(settings: *mut i8, i: usize) -> *mut i8;
    fn nm_ip_route_get_dest(route: *mut i8) -> *const i8;
    fn nm_ip_route_get_prefix(route: *mut i8) -> libc::c_uint;
}

pub struct Ipv4Config(*mut i8);
pub struct Ipv6Config(*mut i8);

impl Ipv4Config {
    pub fn new(conn: *mut NMConnection) -> Ipv4Config {
        let ptr = unsafe {
            nm_connection_get_setting_ip4_config(conn) as _
        };
        Ipv4Config(ptr)
    }
}

impl Ipv6Config {
    pub fn new(conn: *mut NMConnection) -> Ipv6Config {
        let ptr = unsafe {
            nm_connection_get_setting_ip6_config(conn) as _
        };
        Ipv6Config(ptr)
    }
}

impl IpConfigExt for Ipv4Config {
    fn as_ptr(&self) -> *mut i8 {
        self.0
    }
}

impl IpConfigExt for Ipv6Config {
    fn as_ptr(&self) -> *mut i8 {
        self.0
    }
}

pub trait IpConfigExt {
    fn as_ptr(&self) -> *mut i8;

    fn get_num_routes(&self) -> usize {
        unsafe {
            nm_setting_ip_config_get_num_routes(self.as_ptr()) as _
        }
    }

    fn get_route_dst(&self, i: usize) -> Result<Option<IpAddr>, std::net::AddrParseError> {
        let addr = unsafe {
            let route = nm_setting_ip_config_get_route(self.as_ptr(), i as _);
            if route.is_null() {
                return Ok(None);
            };
            let addr = nm_ip_route_get_dest(route);
            std::ffi::CStr::from_ptr(addr)
        };
        Ok(Some(addr.to_string_lossy().parse()?))
    }

    fn get_route_prefix(&self, i: usize) -> Option<i8> {
        unsafe {
            let route = nm_setting_ip_config_get_route(self.as_ptr(), i as _);
            if route.is_null() {
                return None;
            };
            Some(nm_ip_route_get_prefix(route) as _)
        }
    }

    fn get_gateway(&self) -> Result<Option<IpAddr>, std::net::AddrParseError> {
        let addr = unsafe {
            let addr = nm_setting_ip_config_get_gateway(self.as_ptr());
            if addr.is_null() {
                return Ok(None);
            };
            std::ffi::CStr::from_ptr(addr)
        };
        Ok(Some(addr.to_string_lossy().parse()?))
    }


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

