#include <gio/gio.h>

#include "shared/nm-default.h"

// either:
#include "shared/nm-utils/nm-shared-utils.h"
#include "shared/nm-utils/nm-vpn-plugin-macros.h"
// or
//#include <libnm/nm-connection.h>
//#include <libnm/nm-vpn-editor.h>
//#include <libnm/nm-vpn-editor-plugin.h>

#include <libnm/nm-vpn-service-plugin.h>

#define NM_DBUS_SERVICE_WG_P2P_VPN "org.freedesktop.NetworkManager.wg-p2p-vpn"

/*********************************************************************/
#define NM_TYPE_WG_P2P_VPN_PLUGIN            (nm_wg_p2p_vpn_plugin_get_type ())
//#define NM_TYPE_WG_P2P_VPN_PLUGIN          (nm_vpn_service_plugin_get_type ())
#define NM_WG_P2P_VPN_PLUGIN(obj)            (G_TYPE_CHECK_INSTANCE_CAST ((obj), NM_TYPE_WG_P2P_VPN_PLUGIN, NMWgP2pVpnPlugin))
#define NM_WG_P2P_VPN_PLUGIN_CLASS(klass)    (G_TYPE_CHECK_CLASS_CAST ((klass), NM_TYPE_WG_P2P_VPN_PLUGIN, NMWgP2pVpnPluginClass))
#define NM_IS_WG_P2P_VPN_PLUGIN(obj)         (G_TYPE_CHECK_INSTANCE_TYPE ((obj), NM_TYPE_WG_P2P_VPN_PLUGIN))
#define NM_IS_WG_P2P_VPN_PLUGIN_CLASS(klass) (G_TYPE_CHECK_CLASS_TYPE ((klass), NM_TYPE_WG_P2P_VPN_PLUGIN))
#define NM_WG_P2P_VPN_PLUGIN_GET_CLASS(obj)  (G_TYPE_INSTANCE_GET_CLASS ((obj), NM_TYPE_WG_P2P_VPN_PLUGIN, NMWgP2pVpnPluginClass))

typedef struct {
	NMVpnServicePlugin parent;
} NMWgP2pVpnPlugin;

typedef struct {
	NMVpnServicePluginClass parent;
} NMWgP2pVpnPluginClass;


GType nm_wg_p2p_vpn_plugin_get_type (void);

NMWgP2pVpnPlugin *nm_wg_p2p_vpn_plugin_new (const char *bus_name);
/*********************************************************************/
G_DEFINE_TYPE (NMWgP2pVpnPlugin, nm_wg_p2p_vpn_plugin, NM_TYPE_VPN_SERVICE_PLUGIN)


static void
plugin_state_changed (NMWgP2pVpnPlugin *plugin,
                      NMVpnServiceState state,
                      gpointer user_data)
{
//	NMOpenvpnPluginPrivate *priv = NM_OPENVPN_PLUGIN_GET_PRIVATE (plugin);

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

/*
extern gboolean
connect (NMVpnServicePlugin   *plugin,
              NMConnection  *connection,
              GError       **error);
*/

static void log(FILE *fd, char *msg) {
    fwrite(msg, strlen(msg), 1, fd);
}

static gboolean
_connect (NMVpnServicePlugin   *plugin,
              NMConnection  *connection,
              GError       **error)
{
    printf("conn\n");
	return real_connect(plugin, connection, error);
}

static gboolean
real_disconnect (NMVpnServicePlugin *plugin,
                 GError **err)
{
    g_set_error (err, g_quark_from_static_string(""), 0,
               "real_disconnect not implemented");

    return FALSE;
}

#include <libnm/nm-setting-ip-config.h>

static gboolean
real_connect_interactive (NMVpnServicePlugin   *plugin,
                          NMConnection  *connection,
                          GVariant      *details,
                          GError       **error)
{
    /*
    FILE *fd = fopen("/tmp/1debug.log", "w");

    char buf[100];
//    void *s = nm_connection_get_setting_vpn(connection);
  //  sprintf(buf,"items=%d\n\0", nm_setting_vpn_get_num_data_items(s));
    void *s = nm_connection_get_setting_ip4_config(connection);
    sprintf(buf,"items=%d\n\0", nm_setting_ip_config_get_num_addresses(s));
    log(fd, buf);
    void *ip = nm_setting_ip_config_get_address(s, 0);
    char *m = nm_ip_address_get_address (ip);
    log(fd, m);
    fclose(fd);

    //    g_set_error (error, g_quark_from_static_string(""), 0,
  //         "real_connect_interactive not implemented");
    */
	return _connect(plugin, connection, error);
}

static gboolean
real_need_secrets (NMVpnServicePlugin *plugin,
                   NMConnection *connection,
                   const char **setting_name,
                   GError **error)
{
    //g_set_error (error, g_quark_from_static_string(""), 0,
      //         "real_needs_secrets not implemented");
    return FALSE;
}

static gboolean
real_new_secrets (NMVpnServicePlugin *base_plugin,
                  NMConnection *connection,
                  GError **error)
{
    g_set_error (error, g_quark_from_static_string(""), 0,
               "real_new_secrets not implemented");
    return FALSE;
}

static void
dispose (GObject *object)
{
	G_OBJECT_CLASS (nm_wg_p2p_vpn_plugin_parent_class)->dispose (object);
}

static void
nm_wg_p2p_vpn_plugin_class_init(NMWgP2pVpnPluginClass *plugin_class)
{
	GObjectClass *object_class = G_OBJECT_CLASS (plugin_class);
	NMVpnServicePluginClass *parent_class = NM_VPN_SERVICE_PLUGIN_CLASS (plugin_class);

	//g_type_class_add_private (object_class, sizeof (NMWgP2pVpnPluginPrivate));

	object_class->dispose = dispose;

	// virtual methods
	parent_class->connect      = connect;
	parent_class->connect_interactive = real_connect_interactive;
	parent_class->need_secrets = real_need_secrets;
	parent_class->disconnect   = real_disconnect;
	parent_class->new_secrets  = real_new_secrets;
}

static void
nm_wg_p2p_vpn_plugin_init (NMWgP2pVpnPlugin *plugin)
{
}

int
start(gchar *bus_name) {
    NMWgP2pVpnPlugin *plugin;
    GMainLoop *loop;

    plugin = nm_wg_p2p_vpn_plugin_new(bus_name);
	if (!plugin)
		exit (EXIT_FAILURE);

    loop = g_main_loop_new(NULL, FALSE);

    g_main_loop_run(loop);

    g_object_unref(plugin);
	g_main_loop_unref(loop);

    exit(EXIT_SUCCESS);
}

