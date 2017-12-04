#![feature(catch_expr)]

extern crate glib_sys;
extern crate libc;
extern crate docopt;
extern crate tempfile;

const USAGE: &'static str = "WireGuard P2P NetworkManager Service.

Usage:
  wg-p2p-vpn-service [--bus-name <name>]

Options:
  -h --help           Show this screen.
  --bus-name <name>   D-Bus name [default: org.freedesktop.NetworkManager.wg-p2p-vpn].
";

use std::path::PathBuf;
use std::fs::File;
use std::io::Result;
use std::io::Write;
use std::process::Command;
use docopt::Docopt;

mod vpn_settings;
mod ip_settings;

#[repr(u8)]
enum Bool { False = 0, True = 1 }

#[link(name="service", kind="static")]
#[link(name="gio-2.0")]
#[link(name="gobject-2.0")]
#[link(name="glib-2.0")]
#[link(name="nm")]
extern {
    pub fn start(bus_name: *const i8) -> u8;
}

use vpn_settings::VpnSettings;
use ip_settings::IpV4Config;
use ip_settings::IpConfigExt;

fn g_set_error(err: *mut *mut glib_sys::GError,
               msg: String) {
    let msg = std::ffi::CString::new(&msg[..]).unwrap();
    unsafe {
        glib_sys::g_set_error_literal(err, 0, 0, msg.as_ptr());
    }
}

fn create_config_file(conn: *mut vpn_settings::NMConnection) -> Result<(PathBuf, File)> {
    let (path, mut file) = if true {
        let path = "/tmp/fooconf";
        let mut file = std::fs::File::create(path.clone())?;
        (path.into(), file)
    } else {
        let mut file = tempfile::NamedTempFile::new()?;
        let path = file.path().to_path_buf();
        let mut file:File = file.into();
        (path, file)
    };
    // TODO: use correct temp file!

    use std::os::unix::fs::PermissionsExt;
    let mut permissions = file.metadata()?.permissions();
    permissions.set_mode(0o600);
    file.set_permissions(permissions)?;

    let vpn = VpnSettings::new(conn);

    let res = do catch {
        let private_key = "MOEtPGhTswGxgxWBoG9o7zj8kfIedyy2scg9rxJqZks=";//vpn.get_secret_item(vpn_settings::WG_P2P_VPN_LOCAL_PRIVATE_KEY)?;
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
", remote_public_key, endpoint_addr);

        Some((interface, peer))
    };

    if let Some((interface, peer)) = res {
        file.write_all(interface.as_bytes())?;
        file.write_all(peer.as_bytes())?;
    } else {
        let err = "Missing information for WireGuard device";
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, err));
    }

    file.flush()?;
    return Ok((path.into(), file));
}

/*
static gboolean
real_connect (NMVpnServicePlugin   *plugin,
              NMConnection  *connection,
              GError       **error)
*/
#[no_mangle]
pub fn real_connect(_plugin: *mut libc::c_void,
               conn:    *mut vpn_settings::NMConnection,
               error:   *mut *mut glib_sys::GError) -> u8
{
    assert!(!conn.is_null());

    let res: Result<()> = do catch {
        Command::new("/usr/bin/ip")
            .arg("link")
            .arg("add")
            .arg("dev")
            .arg("wg0")
            .arg("type")
            .arg("wireguard")
            .output()?;

        let ipv4 = IpV4Config::new(conn);
        let ipv4_addr = ipv4.get_address(0).unwrap_or(None);
        let ipv4_prefix = ipv4.get_prefix(0).map(|v| format!("/{}", v)).unwrap_or("".to_string());

        if let Some(addr) = ipv4_addr {
            Command::new("/usr/bin/ip")
                .arg("addr")
                .arg("add")
                .arg(format!("{}{}", addr, ipv4_prefix))
                .arg("dev")
                .arg("wg0")
                .output()?;
        }

        let (path, conf_file) = create_config_file(conn).unwrap();

        Command::new("/usr/bin/wg")
            .arg("setconf")
            .arg("wg0")
            .arg(path.clone())
            .output()?;

        drop(conf_file);
        std::fs::remove_file(path)?;

        Ok(())
    };

    if let Err(_) = res {
        g_set_error(error, "TODO".into());//e.to_string());
        return Bool::False as _;
    }

    return Bool::True as _;
}

fn main() {
    let args = Docopt::new(USAGE)
                      .and_then(|d| d.argv(std::env::args().into_iter()).parse())
                      .unwrap_or_else(|e| e.exit());
    let bus_name = args.get_str("--bus-name");
    let bus_name = std::ffi::CString::new(bus_name).unwrap();

    unsafe {
        start(bus_name.as_ptr());
    }
}


