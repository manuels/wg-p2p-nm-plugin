use std;

use glib;
use gdbus;
use glib_sys;
use gio_sys;

pub fn search_for_key<F>(v: &[u8], callback: F)
    where F: Fn(Result<Vec<Vec<u8>>, glib::Error>) + 'static
{
    println!("Searching...");
    let connection = unsafe {
        ::gio_sys::g_bus_get_sync(::gio_sys::G_BUS_TYPE_SESSION, ::std::ptr::null_mut(), ::std::ptr::null_mut())
    };
    let conn = gdbus::connection::Connection::new(connection);
    let msg = gdbus::message::Message::new_method_call("org.manuel.BulletinBoard", "/", "org.manuel.BulletinBoard", "Get");

    let trusted = 0;

    let array_type = std::ffi::CString::new("ay").unwrap();
    let key_raw = unsafe {
        glib_sys::g_variant_new_from_data(array_type.as_ptr() as *const _,
            v.as_ptr() as *const _, v.len(), trusted, None, 0 as *mut _) // TODO: drop callback
    };

    let app_id = std::ffi::CString::new("wg-p2p").unwrap();
    let app_id = unsafe {
        glib_sys::g_variant_new_string(app_id.as_ptr())
    };

    let msg_type = std::ffi::CString::new("(say)").unwrap();
    let args = unsafe {
        let builder = glib_sys::g_variant_builder_new(msg_type.as_ptr() as *const _);
        glib_sys::g_variant_builder_add_value(builder, app_id);
        glib_sys::g_variant_builder_add_value(builder, key_raw);

        let args = glib_sys::g_variant_builder_end(builder);
        glib_sys::g_variant_builder_unref(builder);
        args
    };

    unsafe {
        gio_sys::g_dbus_message_set_body(msg.to_glib(), args);
    };

    conn.send_message_with_reply(msg, ::gdbus::connection::SEND_MESSAGE_FLAGS_NONE, move |resp| {
        let results = resp.map(|r| {
            let mut results: Vec<Vec<u8>> = vec![];

            unsafe {
                let arr = glib_sys::g_variant_get_child_value(r.get_body().to_glib(), 0);

                let iter = glib_sys::g_variant_iter_new(arr);

                let mut child = glib_sys::g_variant_iter_next_value(iter);
                while !child.is_null() {
                    let data = glib_sys::g_variant_get_data_as_bytes(child);
                    let ptr = glib_sys::g_bytes_get_data(data, 0 as *mut _);
                    let size = glib_sys::g_bytes_get_size(data);
                    let data1 = Vec::from_raw_parts(ptr as *mut u8, size, size);
                    results.push(data1.clone());
                    ::std::mem::forget(data1);

                    child = glib_sys::g_variant_iter_next_value(iter);
                }

                glib_sys::g_variant_iter_free(iter);
            };

            results
        });

        callback(results);
    });
}

