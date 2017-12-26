use std::io;
use std::ffi::CString;
use std::net::IpAddr;
use std::mem::transmute;

use libc;
use mnl;
use time;

use mnl::linux::netlink;
use mnl::linux::rtnetlink;
use mnl::linux::if_addr;

pub struct Link {
    pub name: String
}

impl Link {
    pub fn create(typ: &str, name: Option<String>) -> Result<Link, io::Error> {
        let nl = mnl::Socket::open(netlink::Family::ROUTE)?;
        nl.bind(0, mnl::SOCKET_AUTOPID)?;
        let portid = nl.portid();

        let mut buf = vec![0u8; mnl::SOCKET_BUFFER_SIZE()];
        let seq = time::now().to_timespec().sec as u32;

        let mut nlh = mnl::Nlmsg::new(&mut buf)?;
        *nlh.nlmsg_type = rtnetlink::RTM_NEWLINK;
        *nlh.nlmsg_flags = netlink::NLM_F_REQUEST | netlink::NLM_F_ACK | netlink::NLM_F_ATOMIC | netlink::NLM_F_CREATE | netlink::NLM_F_EXCL | netlink::NLM_F_DUMP;
        *nlh.nlmsg_seq = seq;

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

        nl.send_nlmsg(&nlh)?;

        let nrecv = nl.recvfrom(&mut buf)?;
        mnl::cb_run::<u8>(&buf[0..nrecv], seq, portid, None, &mut 0)?;

        nl.close()?;

        Ok(Link { name: name.unwrap() })
    }

    pub fn delete(self) -> Result<(), io::Error> {
        let nl = mnl::Socket::open(netlink::Family::ROUTE)?;
        nl.bind(0, mnl::SOCKET_AUTOPID)?;
        let portid = nl.portid();

        let mut buf = vec![0u8; mnl::SOCKET_BUFFER_SIZE()];
        let seq = time::now().to_timespec().sec as u32;

        let mut nlh = mnl::Nlmsg::new(&mut buf)?;
        *nlh.nlmsg_type = rtnetlink::RTM_DELLINK;
        *nlh.nlmsg_flags = netlink::NLM_F_REQUEST | netlink::NLM_F_ACK | netlink::NLM_F_ATOMIC | netlink::NLM_F_CREATE | netlink::NLM_F_EXCL | netlink::NLM_F_DUMP;
        *nlh.nlmsg_seq = seq;

        let _ifm = nlh.put_sized_header::<rtnetlink::Ifinfomsg>()?;
    	nlh.put_str(mnl::linux::if_link::IFLA_IFNAME, &self.name)?;

        nl.send_nlmsg(&nlh)?;

        let nrecv = nl.recvfrom(&mut buf)?;
        mnl::cb_run::<u8>(&buf[0..nrecv], seq, portid, None, &mut 0)?;

        nl.close()
    }

    pub fn add_route(&self, dst: IpAddr, prefix: u8, gateway: Option<IpAddr>) -> Result<(),io::Error> {
        let nl = mnl::Socket::open(netlink::Family::ROUTE)?;
        nl.bind(0, mnl::SOCKET_AUTOPID)?;
        let portid = nl.portid();

        let iface = unsafe {
            let name = CString::new(&self.name[..]).unwrap();
            libc::if_nametoindex(name.as_ptr())
        };

        let mut buf = vec![0u8; mnl::SOCKET_BUFFER_SIZE()];
        let seq = time::now().to_timespec().sec as u32;

        let mut nlh = mnl::Nlmsg::new(&mut buf)?;
        *nlh.nlmsg_type = rtnetlink::RTM_NEWROUTE;
        *nlh.nlmsg_flags = netlink::NLM_F_REQUEST | netlink::NLM_F_CREATE | netlink::NLM_F_ACK;
        *nlh.nlmsg_seq = seq;

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
        nlh.put_u32(rtnetlink::RTA_OIF, iface)?;
        if let Some(gw) = gateway {
            Self::msg_put_addr(&mut nlh, rtnetlink::RTA_GATEWAY, gw)?;
        }
        nl.send_nlmsg(&nlh)?;

        let nrecv = nl.recvfrom(&mut buf)?;
        mnl::cb_run::<u8>(&buf[0..nrecv], seq, portid, None, &mut 0)?;

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
//                let addr = unsafe { transmute::<[u16;8], libc::in6_addr>(segments) };
                let addr: libc::in6_addr = unsafe { transmute(segments) };
                nlh.put(attr, &addr)
                },
        }
    }

    pub fn add_addr(&self, addr: IpAddr, prefix: u8) -> Result<(),io::Error> {
        let nl = mnl::Socket::open(netlink::Family::ROUTE)?;
        nl.bind(0, mnl::SOCKET_AUTOPID)?;
        let portid = nl.portid();

        let iface = unsafe {
            let name = CString::new(&self.name[..]).unwrap();
            libc::if_nametoindex(name.as_ptr())
        };

        let mut buf = vec![0u8; mnl::SOCKET_BUFFER_SIZE()];
        let seq = time::now().to_timespec().sec as u32;

        let mut nlh = mnl::Nlmsg::new(&mut buf)?;
        *nlh.nlmsg_type = rtnetlink::RTM_NEWADDR;
        *nlh.nlmsg_flags = netlink::NLM_F_REQUEST | netlink::NLM_F_CREATE |
            netlink::NLM_F_EXCL | netlink::NLM_F_ACK;
        *nlh.nlmsg_seq = seq;

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

        nl.send_nlmsg(&nlh)?;

        let nrecv = nl.recvfrom(&mut buf)?;
        mnl::cb_run::<u8>(&buf[0..nrecv], seq, portid, None, &mut 0)?;

        nl.close()
    }

    pub fn set_up(&self, up: bool) -> Result<(),io::Error> {
        let nl = mnl::Socket::open(netlink::Family::ROUTE)?;
        nl.bind(0, mnl::SOCKET_AUTOPID)?;
        let portid = nl.portid();

        let iface = unsafe {
            let name = CString::new(&self.name[..]).unwrap();
            libc::if_nametoindex(name.as_ptr())
        };

        let mut buf = vec![0u8; mnl::SOCKET_BUFFER_SIZE()];
        let seq = time::now().to_timespec().sec as u32;

        let mut nlh = mnl::Nlmsg::new(&mut buf)?;
        *nlh.nlmsg_type = rtnetlink::RTM_NEWLINK;
        *nlh.nlmsg_flags = netlink::NLM_F_REQUEST | netlink::NLM_F_ACK;
        *nlh.nlmsg_seq = seq;

        let ifm = nlh.put_sized_header::<rtnetlink::Ifinfomsg>()?;
        ifm.ifi_family = libc::AF_UNSPEC as u8;
        ifm.ifi_change |= mnl::linux::ifh::IFF_UP;
        ifm.ifi_index = iface as _;
        if up {
            ifm.ifi_flags |= mnl::linux::ifh::IFF_UP;
        } else {
            ifm.ifi_flags &= !mnl::linux::ifh::IFF_UP;
        }

        nl.send_nlmsg(&nlh)?;

        let nrecv = nl.recvfrom(&mut buf)?;
        mnl::cb_run::<u8>(&buf[0..nrecv], seq, portid, None, &mut 0)?;

        nl.close()
    }
}

