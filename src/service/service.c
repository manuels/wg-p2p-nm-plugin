#include <gio/gio.h>

#include "shared/nm-default.h"
#include <libnm/NetworkManager.h>

#define NM_DBUS_SERVICE_WG_P2P_VPN "org.freedesktop.NetworkManager.wg-p2p-vpn"

/*********************************************************************/
#define NM_TYPE_WG_P2P_VPN_PLUGIN            (nm_wg_p2p_vpn_plugin_get_type ())
#define NM_WG_P2P_VPN_PLUGIN(obj)            (G_TYPE_CHECK_INSTANCE_CAST ((obj), NM_TYPE_WG_P2P_VPN_PLUGIN, NMWgP2pVpnPlugin))
#define NM_WG_P2P_VPN_PLUGIN_CLASS(klass)    (G_TYPE_CHECK_CLASS_CAST ((klass), NM_TYPE_WG_P2P_VPN_PLUGIN, NMWgP2pVpnPluginClass))
#define NM_IS_WG_P2P_VPN_PLUGIN(obj)         (G_TYPE_CHECK_INSTANCE_TYPE ((obj), NM_TYPE_WG_P2P_VPN_PLUGIN))
#define NM_IS_WG_P2P_VPN_PLUGIN_CLASS(klass) (G_TYPE_CHECK_CLASS_TYPE ((klass), NM_TYPE_WG_P2P_VPN_PLUGIN))
#define NM_WG_P2P_VPN_PLUGIN_GET_CLASS(obj)  (G_TYPE_INSTANCE_GET_CLASS ((obj), NM_TYPE_WG_P2P_VPN_PLUGIN, NMWgP2pVpnPluginClass))

typedef struct {
    int *rust;
	NMVpnServicePlugin parent;
} NMWgP2pVpnPlugin;

typedef struct {
	NMVpnServicePluginClass parent;
} NMWgP2pVpnPluginClass;

extern gboolean rust_connect(NMVpnServicePlugin  *plugin,
                             int                **rust,
                             NMConnection        *connection,
                             GError             **error);

extern gboolean rust_disconnect(NMVpnServicePlugin *plugin,
                                GError **err);

GType nm_wg_p2p_vpn_plugin_get_type (void);

NMWgP2pVpnPlugin *nm_wg_p2p_vpn_plugin_new (const char *bus_name);
/*********************************************************************/
G_DEFINE_TYPE (NMWgP2pVpnPlugin, nm_wg_p2p_vpn_plugin, NM_TYPE_VPN_SERVICE_PLUGIN)

static void
nm_wg_p2p_vpn_plugin_init (NMWgP2pVpnPlugin *plugin)
{
}

static void
plugin_state_changed (NMWgP2pVpnPlugin *plugin,
                      NMVpnServiceState state,
                      gpointer user_data)
{
	switch (state) {
	case NM_VPN_SERVICE_STATE_UNKNOWN:
	case NM_VPN_SERVICE_STATE_INIT:
	case NM_VPN_SERVICE_STATE_SHUTDOWN:
	case NM_VPN_SERVICE_STATE_STOPPING:
	case NM_VPN_SERVICE_STATE_STOPPED:
		/* Cleanup on failure */
		//nm_clear_g_source (&priv->connect_timer);
		///nm_openvpn_disconnect_management_socket (plugin);
		break;
	default:
		break;
	}
}

NMWgP2pVpnPlugin *
nm_wg_p2p_vpn_plugin_new (const char *bus_name)
{
	NMWgP2pVpnPlugin *plugin;
	GError *error = NULL;

	plugin = (NMWgP2pVpnPlugin *) g_initable_new(NM_TYPE_WG_P2P_VPN_PLUGIN, NULL, &error,
	                                             NM_VPN_SERVICE_PLUGIN_DBUS_SERVICE_NAME, bus_name,
	                                             NM_VPN_SERVICE_PLUGIN_DBUS_WATCH_PEER, FALSE, //!gl.debug,
	                                             NULL);
	if (plugin) {
		g_signal_connect (G_OBJECT (plugin), "state-changed", G_CALLBACK (plugin_state_changed), NULL);
	} else {
		printf("Failed to initialize a plugin instance: %s", error->message);
		g_error_free (error);
	}

	return plugin;
}

static void
dispose (GObject *object)
{
	G_OBJECT_CLASS (nm_wg_p2p_vpn_plugin_parent_class)->dispose (object);
}

static gboolean
real_disconnect (NMVpnServicePlugin *plugin,
                 GError **err)
{
    return rust_disconnect(plugin, err);
}

static gboolean
real_connect_interactive (NMWgP2pVpnPlugin   *plugin,
                          NMConnection  *connection,
                          GVariant      *details,
                          GError       **error)
{
	return rust_connect(plugin, &(plugin->rust), connection, error);
}

static gboolean
real_connect (NMWgP2pVpnPlugin   *plugin,
              NMConnection  *connection,
              GError       **error)
{
	return rust_connect(plugin, &(plugin->rust), connection, error);
}

static gboolean
real_need_secrets (NMVpnServicePlugin *plugin,
                   NMConnection *connection,
                   const char **setting_name,
                   GError **error)
{
    return FALSE; // TODO Will crash if TRUE ->  Why?
}

static gboolean
real_new_secrets (NMVpnServicePlugin *base_plugin,
                  NMConnection *connection,
                  GError **error)
{
    return TRUE;
}

static void
nm_wg_p2p_vpn_plugin_class_init(NMWgP2pVpnPluginClass *plugin_class)
{
	GObjectClass *object_class = G_OBJECT_CLASS (plugin_class);
	NMVpnServicePluginClass *parent_class = NM_VPN_SERVICE_PLUGIN_CLASS (plugin_class);

	object_class->dispose = dispose;

	// virtual methods
	parent_class->connect      = real_connect;
	parent_class->connect_interactive = real_connect_interactive;
	parent_class->need_secrets = real_need_secrets;
	parent_class->disconnect   = real_disconnect;
	parent_class->new_secrets  = real_new_secrets;
}

