use gtk;
use glib;
use gtk_sys;
use gobject_sys;

use std;
use std::io::Read;
use std::io::Write;
use std::process::Command;
use std::process::Stdio;

use drunken_bishop;
use bulletinboard;
use vpn_settings;
use vpn_settings::VpnSettings;
use vpn_settings::NMConnection;

use gtk::prelude::*;
use glib::translate::ToGlibPtr;

const UI_PATH: &str = "/usr/share/gnome-vpn-properties/wg-p2p/wg-p2p-vpn-editor.ui";

#[allow(dead_code)]
#[repr(C)]
enum NMSettingSecretFlags {
    None,
    AgentOwned,
    NotSaved,
    NotRequired,
}

extern "C" {
    fn nma_utils_setup_password_storage(passwd_entry: *const gtk_sys::GtkEntry,
                                        initial_flags: NMSettingSecretFlags ,
                                        setting: *mut i8,
                                        password_flags_name: *const i8,
                                        with_not_required: bool,
                                        ask_mode: bool);
    fn nm_setting_set_secret_flags(setting: *mut i8,
                                   secret_name: *const i8,
                                   flags: NMSettingSecretFlags,
                                   error: *mut *mut i8) -> bool;
    fn nma_utils_menu_to_secret_flags(entry: *const gtk_sys::GtkEntry) -> NMSettingSecretFlags;
}

#[derive(Clone)]
pub struct Editor {
    ptr: *mut u8,
    builder: gtk::Builder,
    main: gtk::Widget,
    generate: gtk::Button,
    find_btn: gtk::Button,
    find_dlg: gtk::Dialog,
    remote_peer_name: gtk::Entry,
    remote_public_key: gtk::Entry,
    interface_name: gtk::Entry,
    endpoint_address: gtk::Entry,
    advanced_settings_btn: gtk::Button,
    advanced_settings_dlg: gtk::Dialog,
    endpoint_method: gtk::RadioButton,
    local_name: gtk::Entry,
    local_private_key: gtk::Entry,
    local_public_key: gtk::Entry,
    local_port: gtk::Entry,
    key_result_list: gtk::TreeView,
    hint: gtk::InfoBar,
    search_status: gtk::Label,
    latest_search_term: Option<String>,
}

fn escape_markup(input: &str) -> String {
    input.replace("&", "&amp;")
}

impl Editor {
    pub fn new(ptr: *mut u8, conn: *mut u8) -> Editor {
        let builder = gtk::Builder::new();
        builder.add_from_file(UI_PATH).unwrap();

        let find_btn = builder.get_object("find").unwrap();
        let find_dlg = builder.get_object("find-dialog").unwrap();
        let remote_peer_name = builder.get_object("remote-peer-name").unwrap();
        let local_private_key = builder.get_object("local-private-key").unwrap();
        let local_public_key = builder.get_object("local-public-key").unwrap();
        let local_name = builder.get_object("local-name").unwrap();
        let local_port = builder.get_object("local-port").unwrap();
        let remote_public_key = builder.get_object("remote-public-key").unwrap();
        let key_result_list = builder.get_object("key-result-list").unwrap();
        let endpoint_address = builder.get_object("endpoint-address").unwrap();
        let endpoint_method = builder.get_object("endpoint-method").unwrap();
        let hint = builder.get_object("hint").unwrap();
        let search_status = builder.get_object("search-status").unwrap();
        let generate = builder.get_object("generate").unwrap();
        let advanced_settings_btn = builder.get_object("advanced-settings").unwrap();
        let advanced_settings_dlg = builder.get_object("advanced-settings-dlg").unwrap();
        let interface_name = builder.get_object("interface-name").unwrap();
        let main = builder.get_object("wg-p2p-vpn-vbox").unwrap();

        let mut editor = Editor {
            ptr,
            main,
            builder,
            generate,
            find_btn,
            find_dlg,
            local_port,
            local_private_key,
            local_public_key,
            local_name,
            interface_name,
            remote_peer_name,
            remote_public_key,
            endpoint_address,
            endpoint_method,
            key_result_list,
            hint,
            advanced_settings_btn,
            advanced_settings_dlg,
            search_status,
            latest_search_term: None,
        };
        editor.set_connection(conn);

        editor
    }

    fn set_connection(&mut self, conn: *mut u8) {
        let signal = ::std::ffi::CString::new("changed").unwrap();
        unsafe {
            gobject_sys::g_signal_emit_by_name (self.ptr as *mut _, signal.as_ptr());
        }

        let this = self.clone();
        self.find_btn.connect_clicked(move |_| {
            this.find_dlg.set_visible(true);
        });

        let this = self.clone();
        self.generate.connect_clicked(move |_| {
            let res = Command::new("/usr/bin/wg")
                .arg("genkey")
                .output();

            if let Ok(output) = res {
                if let Ok(key) = std::str::from_utf8(&output.stdout) {
                    this.local_private_key.set_text(key.trim());
                }
            }
        });

        let this = self.clone();
        self.local_private_key.connect_changed(move |_| this.emit_changed());
        let this = self.clone();
        self.local_name.connect_changed(move |_| this.emit_changed());
        let this = self.clone();
        self.local_port.connect_changed(move |_| this.emit_changed());
        let this = self.clone();
        self.remote_public_key.connect_changed(move |_| this.emit_changed());
        //self.endpoint_method.connect_changed(move |_| emit_changed(editor));
        let this = self.clone();
        self.endpoint_address.connect_changed(move |_| this.emit_changed());

        let this = self.clone();
        self.advanced_settings_dlg.connect_response(move |_dlg, resp| {
            if resp == gtk::ResponseType::Ok.into() {
                this.emit_changed()
            } else {
                let settings = VpnSettings::new(conn as *mut _);
                if let Some(iface) = settings.get_data_item(vpn_settings::WG_P2P_VPN_INTERFACE_NAME) {
                    this.interface_name.set_text(&iface);
                }
            }
            this.advanced_settings_dlg.set_visible(false);
        });

        let this = self.clone();
        self.advanced_settings_btn.connect_clicked(move |_| {
            this.advanced_settings_dlg.run();
        });

        let this = self.clone();
        self.remote_public_key.connect_changed(move |_| {
            let pos = gtk::EntryIconPosition::Secondary;
            let key = this.remote_public_key.get_text().unwrap();
            let key = glib::base64_decode(&key.trim());

            if key.len() != 32 {
                this.remote_public_key.set_icon_tooltip_markup(pos, "Invalid Public Key!");
            } else {
                let text = drunken_bishop::drunken_bishop(&key, drunken_bishop::OPENSSL);
                let text = format!("<tt>{}</tt>", escape_markup(text.trim()));
                this.remote_public_key.set_icon_tooltip_markup(pos, &text[..]);
            }
        });

        let this = self.clone();
        self.local_public_key.connect_changed(move |_| {
            let pos = gtk::EntryIconPosition::Secondary;
            let key = this.local_public_key.get_text().unwrap();
            let key = glib::base64_decode(&key.trim());

            if key.len() != 32 {
                this.local_public_key.set_icon_tooltip_markup(pos, "Invalid Private Key!");
            } else {
                let text = drunken_bishop::drunken_bishop(&key, drunken_bishop::OPENSSL);
                let text = format!("<tt>{}</tt>", escape_markup(text.trim()));
                this.local_public_key.set_icon_tooltip_markup(pos, &text[..]);
            }
        });

        let this = self.clone();
        self.local_private_key.connect_changed(move |_| {
            let key = this.local_private_key.get_text().unwrap();

            let process = Command::new("/usr/bin/wg")
                .arg("pubkey")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn();
            let process = match process {
                Ok(p) => p,
                Err(_) => return,
            };

            if let Some(mut stdin) = process.stdin {
                stdin.write_all(key.as_bytes()).expect("Error writing to wg's stdin");
            } else {
                return;
            }

            if let Some(mut stdout) = process.stdout {
                let mut key = String::new();
                stdout.read_to_string(&mut key).expect("Error readling from wg's stdout");
                this.local_public_key.set_text(key.trim());
            }
        });

        let this = self.clone();
        self.remote_peer_name.connect_activate(move |_| {
            this.find_dlg.response(gtk::ResponseType::Ok.into())
        });

        let this = self.clone();
        self.find_dlg.connect_response(move |_dlg, resp| {
            if resp == gtk::ResponseType::Cancel.into() {
                this.find_dlg.set_visible(false);
                return;
            }

            let name = this.remote_peer_name.get_text().unwrap();
            let sel = this.key_result_list.get_selection().get_selected();
            if Some(name.clone()) == this.latest_search_term {
                if let Some((model, iter)) = sel {
                    let key = model.get_value(&iter, 0);
                    this.remote_public_key.set_text(key.get().unwrap());
                    this.find_dlg.set_visible(false);
                    return;
                }
            }

            this.hint.set_visible(true);
            this.search_status.set_text("Searching…");
            let mut this = this.clone();
            this.latest_search_term = Some(name.clone());
            bulletinboard::search_for_key(name.as_bytes(), move |res| this.on_keys_found(res));
        });

        let col = gtk::TreeViewColumn::new();
        let cell = gtk::CellRendererText::new();
        col.set_title("Public Key");
        col.pack_start(&cell, true);
        col.add_attribute(&cell, "text", 0);
        self.key_result_list.append_column(&col);

        //
        let settings = VpnSettings::new(conn as *mut _);

        unsafe {
            let s = vpn_settings::nm_connection_get_setting_vpn(conn as *mut _);
            let key = std::ffi::CString::new(vpn_settings::WG_P2P_VPN_LOCAL_PRIVATE_KEY).unwrap();
            nma_utils_setup_password_storage(self.local_private_key.to_glib_none().0,
                                             NMSettingSecretFlags::None,
                                             s as *mut _,
                                             key.as_ptr() as *const _,
                                             false, /*with_not_required*/
                                             false /*ask_mode*/);
        };

        let local_private_key = settings.get_data_item(vpn_settings::WG_P2P_VPN_LOCAL_PRIVATE_KEY);
        self.local_private_key.set_text(local_private_key.as_ref().map_or("", |s| &**s));

        let local_name = settings.get_data_item(vpn_settings::WG_P2P_VPN_LOCAL_NAME);
        let local_port = settings.get_data_item(vpn_settings::WG_P2P_VPN_LOCAL_PORT);
        let _endpoint_method = settings.get_data_item(vpn_settings::WG_P2P_VPN_ENDPOINT_METHOD);
        let endpoint_address = settings.get_data_item(vpn_settings::WG_P2P_VPN_ENDPOINT_ADDRESS);
        let remote_public_key = settings.get_data_item(vpn_settings::WG_P2P_VPN_REMOTE_PUBLIC_KEY);
        let iface = settings.get_data_item(vpn_settings::WG_P2P_VPN_INTERFACE_NAME);
        let local_public_key = settings.get_data_item(vpn_settings::WG_P2P_VPN_LOCAL_PUBLIC_KEY);

        self.local_name.set_text(local_name.as_ref().map_or("", |s| &**s));
        self.local_port.set_text(local_port.as_ref().map_or("", |s| &**s));
        self.endpoint_address.set_text(endpoint_address.as_ref().map_or("", |s| &**s));
        self.remote_public_key.set_text(remote_public_key.as_ref().map_or("", |s| &**s));
        self.interface_name.set_text(iface.as_ref().map_or("wg0", |s| &**s));
        self.local_public_key.set_text(local_public_key.as_ref().map_or("", |s| &**s));
    }

    fn emit_changed(&self) {
        let name = std::ffi::CString::new("changed").unwrap();
        unsafe {
            gobject_sys::g_signal_emit_by_name(self.ptr as *mut _, name.as_ptr());
        }
    }

    pub fn get_widget(&self) -> *mut glib::object::GObject {
        use glib::translate::ToGlibPtr;
        return self.main.to_glib_full();
    }

    fn on_keys_found(&self, results: Result<Vec<Vec<u8>>, glib::Error>) {
        self.key_result_list.set_model(None as Option<&gtk::ListStore>);

        let results = match results {
            Ok(r) => r,
            Err(e) => {
                let msg = format!("Error: {:?}", e);
                self.search_status.set_text(&msg);
                return;
            },
        };

        println!("Found {} (maybe invalid) results", results.len());
        let results:Vec<&[u8]> = results.iter().filter_map(|r| {
            if r.len() != 64 {
                return None;
            }

            let mut sha = glib::Checksum::new(glib::ChecksumType::Sha256);
            let (expected_digest, remote_key) = r.split_at(32);
            sha.update(&remote_key);

            if sha.get_digest() == expected_digest {
                Some(remote_key)
            } else {
                None
            }
        }).collect();
        println!("Found {} valid results", results.len());

        if results.len() == 0 {
            self.search_status.set_text("No public keys found!");
            return;
        }

        let model = gtk::ListStore::new(&[gtk::Type::String]);
        for res in results {
            let b64 = glib::base64_encode(&res).unwrap_or("<invalid>".to_string());
            println!("result: {:?}", b64);
            let iter = model.append();
            model.set_value(&iter, 0, &b64.to_value());

        }

        self.hint.set_visible(false);
        self.key_result_list.set_model(&model);
    }

    pub fn update_connection(&self,
                             conn: *mut NMConnection,
                             _err: *mut *mut u8) -> bool
    {
        let mut settings = VpnSettings::new(conn);

        let local_name = self.local_name.get_text().unwrap_or("".to_string());
        let local_port = self.local_port.get_text().unwrap_or("".to_string());
        let endpoint_address = self.endpoint_address.get_text().unwrap_or("".to_string());
        let interface_name = self.interface_name.get_text().unwrap_or("".to_string());
        let local_public_key = self.local_public_key.get_text().unwrap_or("".to_string());
        /* TODO
        let endpoint_method = match self.endpoint_method.is_active() {
            true => "manual",
            false => "p2p",
        };
        */
        let endpoint_method = "manual";
        let remote_public_key = self.remote_public_key.get_text().unwrap_or("".to_string());

        settings.add_data_item(vpn_settings::WG_P2P_VPN_LOCAL_NAME, &local_name);
        settings.add_data_item(vpn_settings::WG_P2P_VPN_LOCAL_PORT, &local_port);
        settings.add_data_item(vpn_settings::WG_P2P_VPN_ENDPOINT_METHOD, &endpoint_method);
        settings.add_data_item(vpn_settings::WG_P2P_VPN_ENDPOINT_ADDRESS, &endpoint_address);
        settings.add_data_item(vpn_settings::WG_P2P_VPN_REMOTE_PUBLIC_KEY, &remote_public_key);
        settings.add_data_item(vpn_settings::WG_P2P_VPN_INTERFACE_NAME, &interface_name);
        settings.add_data_item(vpn_settings::WG_P2P_VPN_LOCAL_PUBLIC_KEY, &local_public_key);

        let local_private_key = self.local_private_key.get_text().unwrap_or("".to_string());
        settings.add_data_item(vpn_settings::WG_P2P_VPN_LOCAL_PRIVATE_KEY, &local_private_key);

        unsafe {
        	let key = vpn_settings::WG_P2P_VPN_LOCAL_PRIVATE_KEY;
        	let key = std::ffi::CString::new(key).unwrap();
            let s = vpn_settings::nm_connection_get_setting_vpn(conn as *mut _);
        	let pw_flags = nma_utils_menu_to_secret_flags(self.local_private_key.to_glib_none().0);
            nm_setting_set_secret_flags(s as *mut _, key.as_ptr(), pw_flags, std::ptr::null_mut());
        };

        true
    }

}
