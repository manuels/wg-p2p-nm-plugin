#![feature(catch_expr)]

extern crate glib_sys;
extern crate glib;
extern crate libc;
extern crate docopt;

const USAGE: &'static str = "WireGuard P2P NetworkManager Service.

Usage:
  wg-p2p-vpn-service [--bus-name <name>]

Options:
  -h --help           Show this screen.
  --bus-name <name>   D-Bus name [default: org.freedesktop.NetworkManager.wg-p2p-vpn].
";

use std::io::Result;
use std::io::Write;
use std::process::Command;
use docopt::Docopt;

mod variant;
mod config;
mod vpn_settings;
mod ip_settings;

use config::create_ipv4_config;
use vpn_settings::VpnSettings;
use ip_settings::IpV4Config;
use ip_settings::IpConfigExt;

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
    fn nm_vpn_service_plugin_set_ip4_config(plugin: *mut NMVpnServicePlugin,
                                            new_config: *mut glib_sys::GVariant);
}

fn create_config_file(conn: *mut vpn_settings::NMConnection) -> Option<String> {
    let vpn = VpnSettings::new(conn);

    let private_key = vpn.get_data_item(vpn_settings::WG_P2P_VPN_LOCAL_PRIVATE_KEY)?;
    let listen_port = vpn.get_data_item(vpn_settings::WG_P2P_VPN_LOCAL_PORT)?;
    let remote_public_key = vpn.get_data_item(vpn_settings::WG_P2P_VPN_REMOTE_PUBLIC_KEY)?;
    let endpoint_addr = vpn.get_data_item(vpn_settings::WG_P2P_VPN_ENDPOINT_ADDRESS)?;

    let interface = format!("[Interface]
PrivateKey = {}
ListenPort = {}
", private_key, listen_port);

    let peer = format!("[Peer]
PublicKey = {}
Endpoint = {}
AllowedIPs = 0.0.0.0/0, ::0/0
", remote_public_key, endpoint_addr);

    Some([interface, peer].concat())
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

    let res: Result<()> = do catch {
        let _ = Command::new("/usr/bin/ip")
            .args(&["link", "del", "dev", iface])
            .output();

        Command::new("/usr/bin/ip")
            .args(&["link", "add", "dev", iface, "type", "wireguard"])
            .output()?;

        let ipv4 = IpV4Config::new(conn);
        let ipv4_addr = ipv4.get_address(0).unwrap_or(None);
        let ipv4_prefix = ipv4.get_prefix(0).map(|v| format!("/{}", v)).unwrap_or("".to_string());

        if let Some(addr) = ipv4_addr {
            let addr = format!("{}{}", addr, ipv4_prefix);
            Command::new("/usr/bin/ip")
                .args(&["addr", "add", &addr, "dev", iface])
                .output()?;
        }

        let conf = create_config_file(conn).unwrap();

        let mut process = Command::new("/usr/bin/wg")
            .stdin(std::process::Stdio::piped())
            .args(&["setconf", iface, "/dev/stdin"])
            .spawn()?;

        if let Some(mut stdin) = process.stdin.as_mut() {
            stdin.write(conf.as_bytes())?;
        }

        process.wait()?;

        Command::new("/usr/bin/ip")
            .args(&["link", "set", iface, "up"])
            .output()?;

        Ok(())
    };

    if let Err(_) = res {
        use glib::translate::ToGlibPtr;
        unsafe {
            // TODO: unsure
            *error = glib::Error::new(glib::FileError::Failed, "TODO wg failed").to_glib_full();
        };
        return false as _;
    }

    let plugin = VpnServicePlugin(plugin);
    glib::source::timeout_add(0, move || {
        let ptr = create_ipv4_config();

        unsafe {
        	nm_vpn_service_plugin_set_ip4_config(plugin.0 as *mut _, ptr);
	    };
    	glib::Continue(false)
    });

    true as _
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

/*
    flags_dump = NLM_F_REQUEST | NLM_F_DUMP
    flags_req = NLM_F_REQUEST | NLM_F_ACK
    flags_create = flags_req | NLM_F_CREATE | NLM_F_EXCL

     'add': (RTM_NEWLINK, flags_create),
         msg = ifinfmsg()
        # ifinfmsg fields
        #
        # ifi_family
        # ifi_type
        # ifi_index
        # ifi_flags
        # ifi_change

            mask = 1                  # IFF_UP mask
            if kwarg['state'].lower() == 'up':
                flags = 1             # 0 (down) or 1 (up)
            del kwarg['state']

        msg['flags'] = flags
        msg['change'] = mask

    ret = self.nlm_request(msg,
                           msg_type=command,
                           msg_flags=msg_flags)

////////////////////////////////////

    let mut nlh = mnl::Nlmsg::new(&mut buf).unwrap();
    *nlh.nlmsg_type = rtnetlink::RTM_NEWLINK;
    *nlh.nlmsg_flags = netlink::NLM_F_REQUEST | netlink::NLM_F_CREATE | netlink::NLM_F_ACK;
    *nlh.nlmsg_seq = seq;
    let ifm = nlh.put_sized_header::<rtnetlink::Ifinfomsg>().unwrap();
    ifm.ifi_family = 0; // no libc::AF_UNSPEC;
    ifm.ifi_change = change;
    ifm.ifi_flags = flags;

    nlh.put_str(if_link::IFLA_IFNAME, &args[1]).unwrap();

    let my_stdout = StdoutRawFd::Dummy;
    nlh.fprintf(&my_stdout, size_of::<rtnetlink::Ifinfomsg>());

    nl.send_nlmsg(&nlh)
.unwrap_or_else(|errno| panic!("mnl_socket_sendto: {}", errno));
*/

