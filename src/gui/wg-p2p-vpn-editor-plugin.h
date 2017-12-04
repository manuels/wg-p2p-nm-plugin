#ifndef __WG_P2P_VPN_EDITOR_PLUGIN_H__
#define __WG_P2P_VPN_EDITOR_PLUGIN_H__

#include <glib-object.h>

#define WG_P2P_VPN_TYPE_EDITOR_PLUGIN                       (wg_p2p_vpn_editor_plugin_get_type ())
#define WG_P2P_VPN_EDITOR_PLUGIN(obj)                       (G_TYPE_CHECK_INSTANCE_CAST ((obj), WG_P2P_VPN_TYPE_EDITOR_PLUGIN, WgP2pVpnEditorPlugin))
#define WG_P2P_VPN_EDITOR_PLUGIN_CLASS(klass)               (G_TYPE_CHECK_CLASS_CAST ((klass), WG_P2P_VPN_TYPE_EDITOR_PLUGIN, WgP2pVpnEditorPluginClass))
#define WG_P2P_VPN_IS_EDITOR_PLUGIN(obj)                    (G_TYPE_CHECK_INSTANCE_TYPE ((obj), WG_P2P_VPN_TYPE_EDITOR_PLUGIN))
#define WG_P2P_VPN_IS_EDITOR_PLUGIN_CLASS(klass)            (G_TYPE_CHECK_CLASS_TYPE ((klass), WG_P2P_VPN_TYPE_EDITOR_PLUGIN))
#define WG_P2P_VPN_EDITOR_PLUGIN_GET_CLASS(obj)             (G_TYPE_INSTANCE_GET_CLASS ((obj), WG_P2P_VPN_TYPE_EDITOR_PLUGIN, WgP2pVpnEditorPluginClass))

typedef struct _WgP2pVpnEditorPlugin WgP2pVpnEditorPlugin;
typedef struct _WgP2pVpnEditorPluginClass WgP2pVpnEditorPluginClass;

struct _WgP2pVpnEditorPlugin {
	GObject parent;
};

struct _WgP2pVpnEditorPluginClass {
	GObjectClass parent;
};

typedef NMVpnEditor *(*NMVpnEditorFactory) (NMVpnEditorPlugin *editor_plugin,
                                            NMConnection *connection,
                                            GError **error);

NMVpnEditor *
nm_vpn_editor_factory_wg_p2pvpn (NMVpnEditorPlugin *editor_plugin,
                             NMConnection *connection,
                             GError **error);

#endif /* __VIEWER_FILE_H__ */
