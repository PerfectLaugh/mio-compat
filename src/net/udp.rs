use std::fmt;
use std::io;
use std::net::{self, Ipv4Addr, Ipv6Addr, SocketAddr};

use crate::poll::convert_ready_to_interests;
use mio::event::Source;

pub struct UdpSocket(mio::net::UdpSocket);

impl UdpSocket {
    pub fn bind(addr: &SocketAddr) -> io::Result<UdpSocket> {
        Ok(UdpSocket(mio::net::UdpSocket::bind(*addr)?))
    }

    pub fn from_socket(socket: net::UdpSocket) -> io::Result<UdpSocket> {
        Ok(UdpSocket(mio::net::UdpSocket::from_socket(socket)?))
    }

    #[cfg_attr(not(target_os = "freebsd"), doc = " ```")]
    #[cfg_attr(target_os = "freebsd", doc = " ```no_run")]
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.0.local_addr()
    }

    pub fn try_clone(&self) -> io::Result<UdpSocket> {
        Ok(UdpSocket(self.0.try_clone()?))
    }

    pub fn send_to(&self, buf: &[u8], target: &SocketAddr) -> io::Result<usize> {
        self.0.send_to(buf, *target)
    }

    pub fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.0.recv_from(buf)
    }

    pub fn send(&self, buf: &[u8]) -> io::Result<usize> {
        self.0.send(buf)
    }

    pub fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.recv(buf)
    }

    pub fn connect(&self, addr: SocketAddr) -> io::Result<()> {
        self.0.connect(addr)
    }

    pub fn set_broadcast(&self, on: bool) -> io::Result<()> {
        self.0.set_broadcast(on)
    }

    pub fn broadcast(&self) -> io::Result<bool> {
        self.0.broadcast()
    }

    pub fn set_multicast_loop_v4(&self, on: bool) -> io::Result<()> {
        self.0.set_multicast_loop_v4(on)
    }

    pub fn multicast_loop_v4(&self) -> io::Result<bool> {
        self.0.multicast_loop_v4()
    }

    pub fn set_multicast_ttl_v4(&self, ttl: u32) -> io::Result<()> {
        self.0.set_multicast_ttl_v4(ttl)
    }

    pub fn multicast_ttl_v4(&self) -> io::Result<u32> {
        self.0.multicast_ttl_v4()
    }

    pub fn set_multicast_loop_v6(&self, on: bool) -> io::Result<()> {
        self.0.set_multicast_loop_v6(on)
    }

    pub fn multicast_loop_v6(&self) -> io::Result<bool> {
        self.0.multicast_loop_v6()
    }

    pub fn set_ttl(&self, ttl: u32) -> io::Result<()> {
        self.0.set_ttl(ttl)
    }

    pub fn ttl(&self) -> io::Result<u32> {
        self.0.ttl()
    }

    pub fn join_multicast_v4(&self, multiaddr: &Ipv4Addr, interface: &Ipv4Addr) -> io::Result<()> {
        self.0.join_multicast_v4(*multiaddr, *interface)
    }

    pub fn join_multicast_v6(&self, multiaddr: &Ipv6Addr, interface: u32) -> io::Result<()> {
        self.0.join_multicast_v6(multiaddr, interface)
    }

    pub fn leave_multicast_v4(&self, multiaddr: &Ipv4Addr, interface: &Ipv4Addr) -> io::Result<()> {
        self.0.leave_multicast_v4(*multiaddr, *interface)
    }

    pub fn leave_multicast_v6(&self, multiaddr: &Ipv6Addr, interface: u32) -> io::Result<()> {
        self.0.leave_multicast_v6(multiaddr, interface)
    }

    pub fn set_only_v6(&self, only_v6: bool) -> io::Result<()> {
        self.0.set_only_v6(only_v6)
    }

    pub fn only_v6(&self) -> io::Result<bool> {
        self.0.only_v6()
    }

    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        self.0.take_error()
    }

    #[cfg(all(unix, not(target_os = "fuchsia")))]
    pub fn recv_bufs(&self, bufs: &mut [&mut IoVec]) -> io::Result<usize> {
        unreachable!();
    }

    #[cfg(all(unix, not(target_os = "fuchsia")))]
    pub fn send_bufs(&self, bufs: &[&IoVec]) -> io::Result<usize> {
        unreachable!();
    }
}

impl crate::Evented for UdpSocket {
    fn register(
        &self,
        poll: &crate::Poll,
        token: crate::Token,
        interest: crate::Ready,
        _opts: crate::PollOpt,
    ) -> io::Result<()> {
        let registry = unsafe { poll.registry() };
        self.0.register(
            registry,
            mio::Token(token.0),
            convert_ready_to_interests(interest).unwrap(),
        )
    }

    fn reregister(
        &self,
        poll: &crate::Poll,
        token: crate::Token,
        interest: crate::Ready,
        _opts: crate::PollOpt,
    ) -> io::Result<()> {
        let registry = unsafe { poll.registry() };
        self.0.reregister(
            registry,
            mio::Token(token.0),
            convert_ready_to_interests(interest).unwrap(),
        )
    }

    fn deregister(&self, poll: &crate::Poll) -> io::Result<()> {
        let registry = unsafe { poll.registry() };
        self.0.deregister(registry)
    }
}

impl fmt::Debug for UdpSocket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}
#[cfg(all(unix, not(target_os = "fuchsia")))]
impl IntoRawFd for UdpSocket {
    fn into_raw_fd(self) -> RawFd {
        self.0.into_raw_fd()
    }
}

#[cfg(all(unix, not(target_os = "fuchsia")))]
impl AsRawFd for UdpSocket {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

#[cfg(all(unix, not(target_os = "fuchsia")))]
impl FromRawFd for UdpSocket {
    unsafe fn from_raw_fd(fd: RawFd) -> UdpSocket {
        UdpSocket(mio::net::UdpSocket::from_raw_fd(fd))
    }
}
