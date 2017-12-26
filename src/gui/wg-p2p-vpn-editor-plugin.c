
#include "shared/nm-default.h"
#include <libnm/nm-connection.h>
#include <libnm/nm-vpn-editor-plugin.h>

#include "wg-p2p-vpn-editor.h"
#include "wg-p2p-vpn-editor-plugin.h"

enum {
	PROP_0,
	PROP_NAME,
	PROP_DESC,
	PROP_SERVICE
};

#define WG_P2P_VPN_PLUGIN_NAME _("WireGuard VPN")
#define WG_P2P_VPN_PLUGIN_DESC _("www.wireguard.com")
#define NM_VPN_SERVICE_TYPE_WG_P2PVPN "org.freedesktop.NetworkManager.wg-p2p-vpn"

static void
wg_p2p_vpn_editor_plugin_interface_init (NMVpnEditorPluginInterface *iface_class);

G_DEFINE_TYPE_EXTENDED (WgP2pVpnEditorPlugin, wg_p2p_vpn_editor_plugin, G_TYPE_OBJECT, 0,
                        G_IMPLEMENT_INTERFACE (NM_TYPE_VPN_EDITOR_PLUGIN,
                                               wg_p2p_vpn_editor_plugin_interface_init))
/*
static NMVpnEditor *
_call_editor_factory (gpointer factory,
                      NMVpnEditorPlugin *editor_plugin,
                      NMConnection *connection,
                      gpointer user_data,
                      GError **error)
{
	g_message("_call_editor_factory");
	return ((NMVpnEditorFactory) factory) (editor_plugin,
	                                       connection,
	                                       error);
}*/

static NMVpnEditor *
get_editor (NMVpnEditorPlugin *iface, NMConnection *connection, GError **error)
{
	g_message("get_editor");
	g_return_val_if_fail (WG_P2P_VPN_IS_EDITOR_PLUGIN (iface), NULL);
	g_return_val_if_fail (NM_IS_CONNECTION (connection), NULL);
	g_return_val_if_fail (!error || !*error, NULL);

	return wg_p2p_vpn_editor_new (connection, error);
}

static guint32
get_capabilities (NMVpnEditorPlugin *iface)
{
	return (0);//NM_VPN_EDITOR_PLUGIN_CAPABILITY_IMPORT |
	        //NM_VPN_EDITOR_PLUGIN_CAPABILITY_EXPORT |
	        //NM_VPN_EDITOR_PLUGIN_CAPABILITY_IPV6);
}


static void
wg_p2p_vpn_editor_plugin_interface_init (NMVpnEditorPluginInterface *iface_class)
{
	g_message("wg_p2p_vpn_editor_plugin_interface_init");
        iface_class->get_editor = get_editor;

        iface_class->get_capabilities = get_capabilities;

	/*
        iface_class->import_from_file = import;
        iface_class->export_to_file = export;
        iface_class->get_suggested_filename = get_suggested_filename;
*/
}

static void
get_property (GObject *object, guint prop_id,
              GValue *value, GParamSpec *pspec)
{
	g_message("get_property");
	switch (prop_id) {
	case PROP_NAME:
    	g_message("PROP_NAME");
		g_value_set_string (value, WG_P2P_VPN_PLUGIN_NAME);
		break;
	case PROP_DESC:
    	g_message("PROP_DESC");
    	g_message("PROP_DESC %s", WG_P2P_VPN_PLUGIN_DESC);
		g_value_set_string (value, WG_P2P_VPN_PLUGIN_DESC);
		break;
	case PROP_SERVICE:
    	g_message("PROP_SERVICE %s", NM_VPN_SERVICE_TYPE_WG_P2PVPN);
		g_value_set_string (value, NM_VPN_SERVICE_TYPE_WG_P2PVPN);
		break;
	default:
		G_OBJECT_WARN_INVALID_PROPERTY_ID (object, prop_id, pspec);
		break;
	}
	g_message("get_property done");
}

static void
wg_p2p_vpn_editor_plugin_init (WgP2pVpnEditorPlugin *plugin)
{
	g_message("wg_p2p_vpn_editor_plugin_init");
}

static void
wg_p2p_vpn_editor_plugin_class_init (WgP2pVpnEditorPluginClass *req_class)
{
	g_message("wg_p2p_vpn_editor_plugin_class_init");
	GObjectClass *object_class = G_OBJECT_CLASS (req_class);

	object_class->get_property = get_property;

	g_object_class_override_property (object_class,
	                                  PROP_NAME,
	                                  "name");

	g_object_class_override_property (object_class,
	                                  PROP_DESC,
	                                  "description");

	g_object_class_override_property (object_class,
	                                  PROP_SERVICE,
	                                  NM_VPN_EDITOR_PLUGIN_SERVICE);
}

G_MODULE_EXPORT NMVpnEditorPlugin *
nm_vpn_editor_plugin_factory (GError **error)
{
	g_message("nm_vpn_editor_plugin_factory");
	g_return_val_if_fail (!error || !*error, NULL);

	return g_object_new (WG_P2P_VPN_TYPE_EDITOR_PLUGIN, NULL);
}
