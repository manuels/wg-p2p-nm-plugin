#include "shared/nm-default.h"
#include <libnm/nm-connection.h>
#include <libnm/nm-vpn-editor.h>
#include <libnm/nm-vpn-editor-plugin.h>

#include "wg-p2p-vpn-editor.h"

#include <gtk/gtk.h>

static void wg_p2p_vpn_editor_plugin_widget_interface_init (NMVpnEditorInterface *iface_class);

G_DEFINE_TYPE_EXTENDED (WgP2pVpnEditor, wg_p2p_vpn_editor_plugin_widget, G_TYPE_OBJECT, 0,
                        G_IMPLEMENT_INTERFACE (NM_TYPE_VPN_EDITOR,
                                               wg_p2p_vpn_editor_plugin_widget_interface_init))

#define WG_P2P_VPN_EDITOR_GET_PRIVATE(o) (G_TYPE_INSTANCE_GET_PRIVATE ((o), WG_P2P_VPN_TYPE_EDITOR, WgP2pVpnEditorPrivate))

// declarations for rust code
GObject *get_widget(NMVpnEditor *self);
gboolean update_connection (NMVpnEditor *iface, NMConnection *connection, GError **error);


static void
wg_p2p_vpn_editor_plugin_widget_init (WgP2pVpnEditor *plugin)
{
	g_message("wg_p2p_vpn_editor_plugin_widget_init");
}

static void
destructor_dispose (GObject *object)
{
	g_message("dispose");
	WgP2pVpnEditor *plugin = WG_P2P_VPN_EDITOR (object);
	G_OBJECT_CLASS (wg_p2p_vpn_editor_plugin_widget_parent_class)->dispose (object);
}

static void
wg_p2p_vpn_editor_plugin_widget_class_init (WgP2pVpnEditorClass *req_class)
{
	g_message("wg_p2p_vpn_editor_plugin_widget_class_init");
	GObjectClass *object_class = G_OBJECT_CLASS (req_class);

	object_class->dispose = destructor_dispose;
}

/*
static gboolean
update_connection (NMVpnEditor *iface,
                   NMConnection *connection,
                   GError **error)
{
	g_message("update_connection");
	return true;
}
*/

static void
wg_p2p_vpn_editor_plugin_widget_interface_init (NMVpnEditorInterface *iface_class)
{
	g_message("wg_p2p_vpn_editor_plugin_widget_interface_init");
	/* interface implementation */
	iface_class->get_widget = get_widget;
	iface_class->update_connection = update_connection;
}

NMVpnEditor *wg_p2p_vpn_editor_new (NMConnection *connection, GError **error) {
	WgP2pVpnEditor *object = g_object_new (WG_P2P_VPN_TYPE_EDITOR, NULL);

	if (!object) {
		g_set_error_literal (error, NM_CONNECTION_ERROR, 0, _("could not create wg-p2p-vpn object"));
		return NULL;
	}

    init(object, connection);

	return (NMVpnEditor*) object;
}

