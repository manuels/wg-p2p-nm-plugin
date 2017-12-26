#![feature(catch_expr)]
#![feature(nll)]

extern crate glib_sys;
extern crate glib;
extern crate libc;
extern crate docopt;
extern crate crslmnl as mnl;
extern crate time;

const USAGE: &'static str = "WireGuard P2P NetworkManager Service.

Usage:
  wg-p2p-vpn-service [--bus-name <name>]

Options:
  -h --help           Show this screen.
  --bus-name <name>   D-Bus name [default: org.freedesktop.NetworkManager.wg-p2p-vpn].
";

use std::io::Result;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Write;
use std::process::Command;
use docopt::Docopt;

mod tests;
mod link;
mod variant;
mod config;
mod vpn_settings;
mod ip_settings;

use glib::translate::ToGlibPtr;

use link::Link;

use config::create_config;
use config::create_ipv4_config;
use config::create_ipv6_config;
use vpn_settings::VpnSettings;
use ip_settings::Ipv4Config;
use ip_settings::Ipv6Config;
use ip_settings::IpConfigExt;

static mut LINK: Option<Link> = None;

type NMVpnServicePlugin = i8;
struct VpnServicePlugin(*mut NMVpnServicePlugin);

unsafe impl Send for VpnServicePlugin {}
unsafe impl Sync for VpnServicePlugin {}

#[link(name="service", kind="static")]
#[link(name="gio-2.0")]
#[link(name="gobject-2.0")]
#[link(name="glib-2.0")]
#[link(name="nm")]
extern "C" {
    fn g_object_unref(ptr: *const i8);
    fn nm_wg_p2p_vpn_plugin_new(bus_name: *const i8) -> *mut i8;
    fn nm_vpn_service_plugin_set_config(plugin: *mut NMVpnServicePlugin,
                                        new_config: *mut glib_sys::GVariant);
    fn nm_vpn_service_plugin_set_ip4_config(plugin: *mut NMVpnServicePlugin,
                                            new_config: *mut glib_sys::GVariant);
    fn nm_vpn_service_plugin_set_ip6_config(plugin: *mut NMVpnServicePlugin,
                                            new_config: *mut glib_sys::GVariant);
}

fn create_config_file(conn: *mut vpn_settings::NMConnection) -> Result<String> {
    let vpn = VpnSettings::new(conn);

    let private_key = vpn.get_data_item(vpn_settings::WG_P2P_VPN_LOCAL_PRIVATE_KEY);
    let listen_port = vpn.get_data_item(vpn_settings::WG_P2P_VPN_LOCAL_PORT);
    let remote_public_key = vpn.get_data_item(vpn_settings::WG_P2P_VPN_REMOTE_PUBLIC_KEY);
    let endpoint_addr = vpn.get_data_item(vpn_settings::WG_P2P_VPN_ENDPOINT_ADDRESS);

    let private_key = private_key.ok_or(Error::new(ErrorKind::Other, "Private Key missing!"))?;
    let listen_port = listen_port.ok_or(Error::new(ErrorKind::Other, "ListenPort missing!"))?;
    let remote_public_key = remote_public_key.ok_or(Error::new(ErrorKind::Other, "Remote Public Key missing!"))?;
    let endpoint_addr = endpoint_addr.ok_or(Error::new(ErrorKind::Other, "Endpoint missing!"))?;

    let interface = format!("[Interface]
PrivateKey = {}
ListenPort = {}
", private_key, listen_port);

    let peer = format!("[Peer]
PublicKey = {}
Endpoint = {}
AllowedIPs = 0.0.0.0/0, ::0/0
", remote_public_key, endpoint_addr);

    Ok([interface, peer].concat())
}

fn apply_device_config<T:IpConfigExt>(link: &mut Link, ip: &T) -> Result<()> {
    for i in 0..ip.get_num_addresses() {
        let ip_addr = ip.get_address(i).unwrap_or(None);
        let ip_prefix = ip.get_prefix(i).unwrap_or(0);

        if let Some(addr) = ip_addr {
            link.add_addr(addr, ip_prefix as _)?;
        }
    }

    for i in 0..ip.get_num_routes() {
        let gw = ip.get_gateway().unwrap_or(None);
        let ip_addr = ip.get_route_dst(i).unwrap_or(None);
        let ip_prefix = ip.get_route_prefix(i).unwrap_or(0);

        if let Some(addr) = ip_addr {
            link.add_route(addr, ip_prefix as _, gw)?;
        }
    }

    Ok(())
}

#[no_mangle]
pub fn rust_disconnect(plugin: *mut NMVpnServicePlugin,
                       error:  *mut *const glib_sys::GError) -> u8
{
    let link = unsafe { LINK.take() }.unwrap();
    link.delete().is_ok() as _
}

#[no_mangle]
pub fn rust_connect(plugin: *mut NMVpnServicePlugin,
               conn:    *mut vpn_settings::NMConnection,
               error:   *mut *const glib_sys::GError) -> u8
{
    assert!(!conn.is_null());

    let settings = VpnSettings::new(conn as *mut _);
    let iface = settings.get_data_item(vpn_settings::WG_P2P_VPN_INTERFACE_NAME);
    let iface = iface.as_ref().map(|s| &**s).unwrap_or("wg0");

    let res: Result<Link> = do catch {
        let mut link = Link::create("wireguard", Some(iface.to_string()))?;

        apply_device_config(&mut link, &Ipv4Config::new(conn))?;
        apply_device_config(&mut link, &Ipv6Config::new(conn))?;

        let mut process = Command::new("/usr/bin/wg")
            .stdin(std::process::Stdio::piped())
            .args(&["setconf", iface, "/dev/stdin"])
            .spawn()?;

        if let Some(mut stdin) = process.stdin.as_mut() {
            let conf = create_config_file(conn)?;
            stdin.write_all(conf.as_bytes())?;
        } else {
            Err(Error::new(ErrorKind::Other, "Opening /usr/bin/wg stdin failed!"))?
        }

        process.wait()?;

        link.set_up(true)?;

        Ok(link)
    };

    match res {
        Err(err) => {
            let err = format!("{:?}", err);
            unsafe {
                *error = glib::Error::new(glib::FileError::Failed, &err).to_glib_full();
            };
            false as _
        },
        Ok(dev) => {
            let name = dev.name.to_string();
            unsafe { LINK = Some(dev) };
            let plugin = VpnServicePlugin(plugin);
            glib::source::timeout_add(0, move || {
                let cfg = create_config(&name);
                let ipv4 = create_ipv4_config();
                let ipv6 = create_ipv6_config();

                // nm_vpn_service_plugin_failure(plugin, NM_VPN_PLUGIN_FAILURE_CONNECT_FAILED);

                unsafe {
                	nm_vpn_service_plugin_set_config(plugin.0 as *mut _, cfg);
                	nm_vpn_service_plugin_set_ip4_config(plugin.0 as *mut _, ipv4);
                	nm_vpn_service_plugin_set_ip6_config(plugin.0 as *mut _, ipv6);
	            };
            	glib::Continue(false)
            });

            true as _
        }
    }
}

fn main() {
    let args = Docopt::new(USAGE)
                      .and_then(|d| d.argv(std::env::args().into_iter()).parse())
                      .unwrap_or_else(|e| e.exit());
    let bus_name = args.get_str("--bus-name");
    let bus_name = std::ffi::CString::new(bus_name).unwrap();

    let plugin = unsafe {
        nm_wg_p2p_vpn_plugin_new(bus_name.as_ptr())
    };

    glib::MainLoop::new(None, false).run();

    unsafe {
        g_object_unref(plugin);
    }
}

