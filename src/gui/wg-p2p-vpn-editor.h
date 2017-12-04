#ifndef __WG_P2P_VPN_EDITOR_H__
#define __WG_P2P_VPN_EDITOR_H__

#define WG_P2P_VPN_TYPE_EDITOR                (wg_p2p_vpn_editor_plugin_widget_get_type ())
#define WG_P2P_VPN_EDITOR(obj)                (G_TYPE_CHECK_INSTANCE_CAST ((obj), WG_P2P_VPN_TYPE_EDITOR, WgP2pVpnEditor))
#define WG_P2P_VPN_EDITOR_CLASS(klass)        (G_TYPE_CHECK_CLASS_CAST ((klass), WG_P2P_VPN_TYPE_EDITOR, WgP2pVpnEditorClass))
#define WG_P2P_VPN_IS_EDITOR(obj)             (G_TYPE_CHECK_INSTANCE_TYPE ((obj), WG_P2P_VPN_TYPE_EDITOR))
#define WG_P2P_VPN_IS_EDITOR_CLASS(klass)     (G_TYPE_CHECK_CLASS_TYPE ((klass), WG_P2P_VPN_TYPE_EDITOR))
#define WG_P2P_VPN_EDITOR_GET_CLASS(obj)      (G_TYPE_INSTANCE_GET_CLASS ((obj), WG_P2P_VPN_TYPE_EDITOR, WgP2pVpnEditorClass))

typedef struct _WgP2pVpnEditor WgP2pVpnEditor;
typedef struct _WgP2pVpnEditorClass WgP2pVpnEditorClass;

struct _WgP2pVpnEditor {
	GObject parent;
};

struct _WgP2pVpnEditorClass {
	GObjectClass parent;
};

GType wg_p2p_vpn_editor_plugin_widget_get_type (void);

NMVpnEditor *wg_p2p_vpn_editor_new (NMConnection *connection, GError **error);

#endif
