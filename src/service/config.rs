use std::net::Ipv4Addr;

use glib_sys;

use variant::VariantBuilder;

const NM_VPN_PLUGIN_CONFIG_TUNDEV: &str = "tundev";
const NM_VPN_PLUGIN_CONFIG_HAS_IP4: &str = "has-ip4";
const NM_VPN_PLUGIN_CONFIG_HAS_IP6: &str = "has-ip6";
const NM_VPN_PLUGIN_IP4_CONFIG_ADDRESS: &str = "address";
const NM_VPN_PLUGIN_IP4_CONFIG_PREFIX: &str = "prefix";

pub fn create_ipv4_config() -> *mut glib_sys::GVariant {
    let mut builder = VariantBuilder::new("a{sv}").unwrap();

    let addr: Ipv4Addr = "10.0.0.1".parse().unwrap();
    let addr: u32 = addr.into();
    let prefix: u32 = 24;

    builder.add_dict_entry(NM_VPN_PLUGIN_IP4_CONFIG_ADDRESS, addr.to_be().into());
    builder.add_dict_entry(NM_VPN_PLUGIN_IP4_CONFIG_PREFIX, prefix.into());

    builder.to_raw_value()
}

pub fn create_ipv6_config() -> *mut glib_sys::GVariant {
    let mut builder = VariantBuilder::new("a{sv}").unwrap();

    //let addr: Ipv4Addr = "10.0.0.1".parse().unwrap();
    //let addr: u32 = addr.into();
    //let prefix: u32 = 24;

//    builder.add_dict_entry(NM_VPN_PLUGIN_IP4_CONFIG_ADDRESS, addr.to_be().into());
  //  builder.add_dict_entry(NM_VPN_PLUGIN_IP4_CONFIG_PREFIX, prefix.into());

    builder.to_raw_value()
}

pub fn create_config(dev: &str) -> *mut glib_sys::GVariant {
    let mut builder = VariantBuilder::new("a{sv}").unwrap();

    builder.add_dict_entry(NM_VPN_PLUGIN_CONFIG_TUNDEV, dev.into());
    builder.add_dict_entry(NM_VPN_PLUGIN_CONFIG_HAS_IP4, true.into());
    builder.add_dict_entry(NM_VPN_PLUGIN_CONFIG_HAS_IP6, true.into());

    builder.to_raw_value()
    /*
    /* string: VPN interface name (tun0, tap0, etc) */
    #define NM_VPN_PLUGIN_CONFIG_TUNDEV      "tundev"

    /* string: Login message */
    #define NM_VPN_PLUGIN_CONFIG_BANNER      "banner"

    /* uint32 / array of uint8: IP address of the public external VPN gateway (network byte order) */
    #define NM_VPN_PLUGIN_CONFIG_EXT_GATEWAY "gateway"

    /* uint32: Maximum Transfer Unit that the VPN interface should use */
    #define NM_VPN_PLUGIN_CONFIG_MTU         "mtu"
    */
}

