use std;
use std::io;
use std::ffi::CString;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::mem::transmute;

use libc;
use mnl;

use netlink as nl;

use mnl::linux::netlink;
use mnl::linux::rtnetlink;
use mnl::linux::if_addr;
use mnl::linux::genetlink;

const WG_GENL_NAME: &str = "wireguard";
const WG_GENL_VERSION: u8 = 1;

pub struct Peer {
    pub public_key: [u8; 32],
    pub endpoint: Option<SocketAddr>,
    pub psk: Option<[u8; 32]>,
    pub keepalive: Option<u16>,
    pub allowed_ips: Vec<(IpAddr, u8)>,
}

#[repr(C)]
#[allow(dead_code)]
pub struct Genlmsghdr {
    pub cmd: u8,
    pub version: u8,
    pub reserved: u16,
}

#[repr(C)]
#[allow(dead_code)]
enum WgCmd {
	GetDevice,
	SetDevice,
}

#[repr(C)]
#[allow(dead_code)]
enum WgDeviceFlag {
	ReplacePeers = 1,
}

#[repr(C)]
#[allow(dead_code)]
enum WgDeviceAttr {
	Unspec,
	IfIndex,
	IfName,
	PrivateKey,
	PublicKey,
	Flags,
	ListenPort,
	FwMark,
	Peers,
}

#[repr(C)]
#[allow(dead_code)]
enum WgPeerFlag {
	RemoveMe = 1,
	ReplaceAllowedIps = 2,
}

#[repr(C)]
#[allow(dead_code)]
enum WgPeerAttr {
    Unspec,
	PublicKey,
	PresharedKey,
	Flags,
	Endpoint,
	PersistentKeepAliveInterval,
	LastHandshakeTime,
	RxBytes,
	TxBytes,
    AllowedIps,
}

#[repr(C)]
#[allow(dead_code)]
enum WgAllowedIpAttr {
	Unspec,
	Family,
	IpAddr,
    CidrMask,
}

pub struct Link {
    pub name: String
}

impl Link {
    pub fn create(typ: &str, name: Option<String>) -> Result<Link, io::Error> {
        let mut nl = nl::Socket::open(netlink::Family::ROUTE)?;

        let flags = netlink::NLM_F_REQUEST | netlink::NLM_F_ACK | netlink::NLM_F_ATOMIC | netlink::NLM_F_CREATE | netlink::NLM_F_EXCL | netlink::NLM_F_DUMP;
        let mut nlh = nl.new_msg(rtnetlink::RTM_NEWLINK, flags)?;

        let _ifm = nlh.put_sized_header::<rtnetlink::Ifinfomsg>()?;
        if let Some(ref name) = name {
        	nlh.put_str(mnl::linux::if_link::IFLA_IFNAME, &name)?;
    	}

        assert_eq!(typ, "wireguard");
        let something_something_wireguard = [0x0d, 0x00, 0x01, 0x00,
            0x77, 0x69, 0x72, 0x65, 0x67, 0x75, 0x61, 0x72, 0x64, // "wireguard"
            0x00, 0x00, 0x00] as [u8; 16];
        assert_eq!(mnl::linux::if_link::IFLA_LINKINFO, 18);
    	nlh.put(mnl::linux::if_link::IFLA_LINKINFO, &something_something_wireguard)?;

        nl.send_recv(nlh, None, &mut 0)?;
        nl.close()?;

        Ok(Link { name: name.unwrap() })
    }

    pub fn delete(self) -> Result<(), io::Error> {
        let mut nl = nl::Socket::open(netlink::Family::ROUTE)?;

        let flags = netlink::NLM_F_REQUEST | netlink::NLM_F_ACK | netlink::NLM_F_ATOMIC | netlink::NLM_F_CREATE | netlink::NLM_F_EXCL | netlink::NLM_F_DUMP;
        let mut nlh = nl.new_msg(rtnetlink::RTM_NEWLINK, flags)?;

        let _ifm = nlh.put_sized_header::<rtnetlink::Ifinfomsg>()?;
    	nlh.put_str(mnl::linux::if_link::IFLA_IFNAME, &self.name)?;

        nl.send_recv(nlh, None, &mut 0)?;
        nl.close()
    }

    pub fn add_route(&self, dst: IpAddr, prefix: u8, gateway: Option<IpAddr>) -> Result<(),io::Error> {
        let mut nl = nl::Socket::open(netlink::Family::ROUTE)?;
        let mut nlh = nl.new_msg(rtnetlink::RTM_NEWROUTE, netlink::NLM_F_REQUEST | netlink::NLM_F_CREATE | netlink::NLM_F_ACK)?;

        let rtm = nlh.put_sized_header::<rtnetlink::Rtmsg>()?;
        rtm.rtm_family = if dst.is_ipv6() { libc::AF_INET6 } else { libc::AF_INET } as u8;
        rtm.rtm_dst_len = prefix;
        rtm.rtm_src_len = 0;
        rtm.rtm_tos = 0;
        rtm.rtm_protocol = rtnetlink::RTPROT_STATIC;
        rtm.rtm_table = rtnetlink::RT_TABLE_MAIN as u8;
        rtm.rtm_type = rtnetlink::RTN_UNICAST;
        rtm.rtm_scope = if gateway.is_some() && dst.is_ipv6() { rtnetlink::RT_SCOPE_LINK } else { rtnetlink::RT_SCOPE_UNIVERSE };
        rtm.rtm_flags = 0;

        Self::msg_put_addr(&mut nlh, rtnetlink::RTA_DST, dst)?;

        let iface = unsafe {
            let name = CString::new(&self.name[..]).unwrap();
            libc::if_nametoindex(name.as_ptr())
        };
        nlh.put_u32(rtnetlink::RTA_OIF, iface)?;

        if let Some(gw) = gateway {
            Self::msg_put_addr(&mut nlh, rtnetlink::RTA_GATEWAY, gw)?;
        }

        nl.send_recv(nlh, None, &mut 0)?;
        nl.close()
    }

    fn msg_put_addr(nlh: &mut mnl::Nlmsg, attr: u16, addr: IpAddr) -> Result<(), io::Error> {
        match addr {
            IpAddr::V4(addr) => {
                let addr = unsafe { transmute(addr.octets()) };
                nlh.put_u32(attr, addr)
            },
            IpAddr::V6(addr) => {
                let mut segments = addr.segments();
                for s in segments.iter_mut() {
                    *s = s.to_be();
                }

                let addr: libc::in6_addr = unsafe { transmute(segments) };
                nlh.put(attr, &addr)
            },
        }
    }

    pub fn add_addr(&self, addr: IpAddr, prefix: u8) -> Result<(),io::Error> {
        let mut nl = nl::Socket::open(netlink::Family::ROUTE)?;
        let mut nlh = nl.new_msg(rtnetlink::RTM_NEWADDR, netlink::NLM_F_REQUEST | netlink::NLM_F_CREATE |
            netlink::NLM_F_EXCL | netlink::NLM_F_ACK)?;

        let iface = unsafe {
            let name = CString::new(&self.name[..]).unwrap();
            libc::if_nametoindex(name.as_ptr())
        };

        let ifm = nlh.put_sized_header::<if_addr::Ifaddrmsg>()?;
        ifm.ifa_family = if addr.is_ipv6() { libc::AF_INET6 } else { libc::AF_INET } as u8;
        ifm.ifa_index = iface;

        ifm.ifa_prefixlen = prefix;
        ifm.ifa_flags = if_addr::IFA_F_PERMANENT as u8;
        ifm.ifa_scope = rtnetlink::RT_SCOPE_UNIVERSE;

        Self::msg_put_addr(&mut nlh, if_addr::IFA_ADDRESS, addr)?;
        if addr.is_ipv4() {
            Self::msg_put_addr(&mut nlh, if_addr::IFA_LOCAL, addr)?;
        }

        nl.send_recv(nlh, None, &mut 0)?;
        nl.close()
    }

    pub fn set_up(&self, up: bool) -> Result<(),io::Error> {
        let mut nl = nl::Socket::open(netlink::Family::ROUTE)?;
        let mut nlh = nl.new_msg(rtnetlink::RTM_NEWLINK, netlink::NLM_F_REQUEST | netlink::NLM_F_ACK)?;

        let iface = unsafe {
            let name = CString::new(&self.name[..]).unwrap();
            libc::if_nametoindex(name.as_ptr())
        };

        let ifm = nlh.put_sized_header::<rtnetlink::Ifinfomsg>()?;
        ifm.ifi_family = libc::AF_UNSPEC as u8;
        ifm.ifi_change |= mnl::linux::ifh::IFF_UP;
        ifm.ifi_index = iface as _;
        if up {
            ifm.ifi_flags |= mnl::linux::ifh::IFF_UP;
        } else {
            ifm.ifi_flags &= !mnl::linux::ifh::IFF_UP;
        }

        nl.send_recv(nlh, None, &mut 0)?;
        nl.close()
    }

    fn get_family_id(family_name: &str) -> Result<u16, io::Error> {
        let mut nl = nl::Socket::open(netlink::Family::GENERIC)?;
        let mut nlh = nl.new_msg(genetlink::GENL_ID_CTRL, netlink::NLM_F_REQUEST | netlink::NLM_F_ACK)?;

        let genl = nlh.put_sized_header::<Genlmsghdr>()?;
    	genl.cmd = genetlink::CTRL_CMD_GETFAMILY;
        genl.version = 1;

        nlh.put_strz(genetlink::CTRL_ATTR_FAMILY_NAME, family_name)?;

        let get_family_id = |nlh: mnl::Nlmsg, family: &mut Option<u16>| {
            let get_family_id_attr = |attr: &mnl::Attr, family: &mut Option<u16>| {
                if attr.atype() == genetlink::CTRL_ATTR_FAMILY_ID {
                    if attr.validate(mnl::AttrDataType::U16).is_err() {
                        return mnl::CbRet::ERROR;
                    } else {
                        *family = Some(attr.u16());
                    }
                }
                mnl::CbRet::OK
            };

            let offset = std::mem::size_of::<genetlink::Genlmsghdr>();
            let res = nlh.parse(offset, get_family_id_attr, family);
            res.unwrap_or(mnl::CbRet::ERROR)
        };

        let mut family: Option<u16> = None;
        nl.send_recv(nlh, Some(get_family_id), &mut family)?;
        nl.close()?;

        family.ok_or(io::Error::from_raw_os_error(libc::EOPNOTSUPP))
    }

    pub fn set_wireguard(&mut self,
                         private_key: [u8; 32],
                         listen_port: u16,
                         fwmark: Option<u32>,
                         peer_list: &[Peer])
        -> Result<(), io::Error>
    {
        let typ = Self::get_family_id(WG_GENL_NAME)?;
        let mut nl = nl::Socket::open(netlink::Family::GENERIC)?;
        let mut nlh = nl.new_msg(typ, netlink::NLM_F_REQUEST | netlink::NLM_F_ACK)?;

        let genl = nlh.put_sized_header::<Genlmsghdr>()?;
    	genl.cmd = WgCmd::SetDevice as _;
        genl.version = WG_GENL_VERSION;

        nlh.put_strz(WgDeviceAttr::IfName as _, &self.name)?;

        nlh.put(WgDeviceAttr::PrivateKey as _, &private_key)?;
        nlh.put_u16(WgDeviceAttr::ListenPort as _, listen_port)?;
        if let Some(fwmark) = fwmark {
            nlh.put_u32(WgDeviceAttr::FwMark as _, fwmark)?;
        }

	    nlh.put_u32(WgDeviceAttr::Flags as _, WgDeviceFlag::ReplacePeers as _)?;

        let mut nest = nlh.nest_start(WgDeviceAttr::Peers as _)?;
        for (i, peer) in peer_list.iter().enumerate() {
            let mut peer_nest = nlh.nest_start(i as u16)?;

            nlh.put(WgPeerAttr::PublicKey as _, &peer.public_key)?;
            if let Some(ref psk) = peer.psk {
                nlh.put(WgPeerAttr::PresharedKey as _, psk)?;
            }

            if let Some(addr) = peer.endpoint {
                match addr {
                    SocketAddr::V4(addr) => {
                        let addr = libc::sockaddr_in {
                            sin_family: libc::AF_INET as _,
                            sin_port: addr.port().to_be(),
                            sin_addr: unsafe { transmute(addr.ip().octets()) },
                            sin_zero: [0;8],
                        };
                        nlh.put(WgPeerAttr::Endpoint as _, &addr)?;
                    },
                    SocketAddr::V6(addr) => {
                        let mut segments = addr.ip().segments();
                        for s in segments.iter_mut() {
                            *s = s.to_be();
                        }

                        let addr = libc::sockaddr_in6 {
                            sin6_family: libc::AF_INET6 as _,
                            sin6_port: addr.port().to_be(),
                            sin6_addr: unsafe { transmute(segments) },
                            sin6_flowinfo: 0,
                            sin6_scope_id: 0,
                        };
                        nlh.put(WgPeerAttr::Endpoint as _, &addr)?;
                    },
                };
            }

            if let Some(keepalive) = peer.keepalive {
                nlh.put_u16(WgPeerAttr::PersistentKeepAliveInterval as _, keepalive)?;
            }

            let mut ip_list_nest = nlh.nest_start(WgPeerAttr::AllowedIps as _)?;
            for (j, &(ip, cidr)) in peer.allowed_ips.iter().enumerate() {
                let mut ip_nest = nlh.nest_start(j as _)?;
                nlh.put_u16(WgAllowedIpAttr::Family as _, if ip.is_ipv6() { libc::AF_INET6 } else { libc::AF_INET } as _)?;
                Self::msg_put_addr(&mut nlh, WgAllowedIpAttr::IpAddr as _, ip)?;
                nlh.put_u8(WgAllowedIpAttr::CidrMask as _, cidr)?;
                nlh.nest_end(&mut ip_nest);
            }
            nlh.nest_end(&mut ip_list_nest);
            nlh.nest_end(&mut peer_nest);
        }

        nlh.nest_end(&mut nest);

        nl.send_recv(nlh, None, &mut 0)?;
        nl.close()
    }
}

