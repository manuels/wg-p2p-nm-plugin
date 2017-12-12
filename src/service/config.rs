use std::net::Ipv4Addr;

use glib_sys;

use variant::VariantBuilder;

const NM_VPN_PLUGIN_IP4_CONFIG_ADDRESS: &str = "address";
const NM_VPN_PLUGIN_IP4_CONFIG_PREFIX: &str = "prefix";

pub fn create_ipv4_config() -> *mut glib_sys::GVariant {
    let mut builder = VariantBuilder::new("a{sv}").unwrap();

    let addr: Ipv4Addr = "10.0.1.10".parse().unwrap();
    let addr: u32 = addr.into();
    let prefix: u32 = 10;

    builder.add_dict_entry(NM_VPN_PLUGIN_IP4_CONFIG_ADDRESS, addr.to_be().into());
    builder.add_dict_entry(NM_VPN_PLUGIN_IP4_CONFIG_PREFIX, prefix.into());

    builder.to_raw_value()
}

