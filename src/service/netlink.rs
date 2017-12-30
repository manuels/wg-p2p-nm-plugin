use std::io::Result;

use mnl;
use mnl::linux::netlink;
use time;

pub struct Socket<'a> {
    nl: &'a mut mnl::Socket,
    buf: Vec<u8>,
}

impl<'a> Socket<'a> {
    pub fn open(family: netlink::Family) -> Result<Socket<'a>> {
        let nl = mnl::Socket::open(family)?;
        nl.bind(0, mnl::SOCKET_AUTOPID)?;

        Ok(Socket {
            nl,
            buf: vec![0; mnl::SOCKET_BUFFER_SIZE()],
        })
    }

    pub fn new_msg(&mut self, typ: u16, flags: u16) -> Result<mnl::Nlmsg<'a>> {
        let seq = time::now().to_timespec().sec as u32;

        let nlh = mnl::Nlmsg::new(&mut self.buf)?;
        *nlh.nlmsg_type = typ;
        *nlh.nlmsg_flags = flags;
        *nlh.nlmsg_seq = seq;

        Ok(nlh)
    }

    pub fn send_recv<T>(&mut self,
                        nlh: mnl::Nlmsg,
                        cb: Option<for<'r> fn(mnl::Nlmsg<'r>, &mut T) -> mnl::CbRet>,
                        data: &mut T)
        -> Result<()>
    {
        self.nl.send_nlmsg(&nlh)?;

        let nrecv = self.nl.recvfrom(&mut self.buf)?;
        mnl::cb_run(&self.buf[0..nrecv], *nlh.nlmsg_seq, self.nl.portid(),
                    cb, data)?;

        Ok(())
    }

    pub fn close(&mut self) -> Result<()> {
       self.nl.close()
    }
}

