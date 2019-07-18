use std::fmt;
use std::io;
use std::io::{Read, Write};
use std::net::{self, SocketAddr, Shutdown};
use std::time::Duration;

use iovec::IoVec;

use mio::event::Source;
use crate::poll::convert_ready_to_interests;

pub struct TcpStream(mio::net::TcpStream);

impl TcpStream {
    pub fn connect(addr: &SocketAddr) -> io::Result<TcpStream> {
        Ok(TcpStream(mio::net::TcpStream::connect(*addr)?))
    }

    pub fn connect_stream(stream: net::TcpStream,
                          addr: &SocketAddr) -> io::Result<TcpStream> {
        Ok(TcpStream(mio::net::TcpStream::connect_stream(stream, *addr)?))
    }

    pub fn from_stream(stream: net::TcpStream) -> io::Result<TcpStream> {
        Ok(TcpStream(mio::net::TcpStream::from_stream(stream)?))
    }

    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.0.peer_addr()
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.0.local_addr()
    }

    pub fn try_clone(&self) -> io::Result<TcpStream> {
        Ok(TcpStream(self.0.try_clone()?))
    }

    pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        self.0.shutdown(how)
    }

    pub fn set_nodelay(&self, nodelay: bool) -> io::Result<()> {
        self.0.set_nodelay(nodelay)
    }

    pub fn nodelay(&self) -> io::Result<bool> {
        self.0.nodelay()
    }

    pub fn set_recv_buffer_size(&self, size: usize) -> io::Result<()> {
        self.0.set_recv_buffer_size(size)
    }

    pub fn recv_buffer_size(&self) -> io::Result<usize> {
        self.0.recv_buffer_size()
    }

    pub fn set_send_buffer_size(&self, size: usize) -> io::Result<()> {
        self.0.set_send_buffer_size(size)
    }

    pub fn send_buffer_size(&self) -> io::Result<usize> {
        self.0.send_buffer_size()
    }

    pub fn set_keepalive(&self, keepalive: Option<Duration>) -> io::Result<()> {
        self.0.set_keepalive(keepalive)
    }

    pub fn keepalive(&self) -> io::Result<Option<Duration>> {
        self.0.keepalive()
    }

    pub fn set_ttl(&self, ttl: u32) -> io::Result<()> {
        self.0.set_ttl(ttl)
    }

    pub fn ttl(&self) -> io::Result<u32> {
        self.0.ttl()
    }

    pub fn set_only_v6(&self, _only_v6: bool) -> io::Result<()> {
        unreachable!();
    }

    pub fn only_v6(&self) -> io::Result<bool> {
        unreachable!();
    }

    /// Sets the value for the `SO_LINGER` option on this socket.
    pub fn set_linger(&self, dur: Option<Duration>) -> io::Result<()> {
        self.0.set_linger(dur)
    }

    /// Gets the value of the `SO_LINGER` option on this socket.
    ///
    /// For more information about this option, see [`set_linger`][link].
    ///
    /// [link]: #method.set_linger
    pub fn linger(&self) -> io::Result<Option<Duration>> {
        self.0.linger()
    }

    #[deprecated(since = "0.6.9", note = "use set_keepalive")]
    #[cfg(feature = "with-deprecated")]
    #[doc(hidden)]
    pub fn set_keepalive_ms(&self, keepalive: Option<u32>) -> io::Result<()> {
        self.set_keepalive(keepalive.map(|v| {
            Duration::from_millis(u64::from(v))
        }))
    }

    #[deprecated(since = "0.6.9", note = "use keepalive")]
    #[cfg(feature = "with-deprecated")]
    #[doc(hidden)]
    pub fn keepalive_ms(&self) -> io::Result<Option<u32>> {
        self.keepalive().map(|v| {
            v.map(|v| {
                crate::convert::millis(v) as u32
            })
        })
    }

    /// Get the value of the `SO_ERROR` option on this socket.
    ///
    /// This will retrieve the stored error in the underlying socket, clearing
    /// the field in the process. This can be useful for checking errors between
    /// calls.
    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        self.0.take_error()
    }

    /// Receives data on the socket from the remote address to which it is
    /// connected, without removing that data from the queue. On success,
    /// returns the number of bytes peeked.
    ///
    /// Successive calls return the same data. This is accomplished by passing
    /// `MSG_PEEK` as a flag to the underlying recv system call.
    pub fn peek(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.peek(buf)
    }

    pub fn read_bufs(&self, _bufs: &mut [&mut IoVec]) -> io::Result<usize> {
        unreachable!();
    }

    pub fn write_bufs(&self, _bufs: &[&IoVec]) -> io::Result<usize> {
        unreachable!();
    }
}

impl Read for TcpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (&self.0).read(buf)
    }
}

impl<'a> Read for &'a TcpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (&self.0).read(buf)
    }
}

impl Write for TcpStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        (&self.0).write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        (&self.0).flush()
    }
}

impl<'a> Write for &'a TcpStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        (&self.0).write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        (&self.0).flush()
    }
}

impl crate::Evented for TcpStream {
    fn register(&self, poll: &crate::Poll, token: crate::Token, interest: crate::Ready, _opts: crate::PollOpt) -> io::Result<()> {
        poll.use_registry(|r| self.0.register(r, mio::Token(token.0), convert_ready_to_interests(interest).unwrap()))
    }

    fn reregister(&self, poll: &crate::Poll, token: crate::Token, interest: crate::Ready, _opts: crate::PollOpt) -> io::Result<()> {
        poll.use_registry(|r| self.0.reregister(r, mio::Token(token.0), convert_ready_to_interests(interest).unwrap()))
    }

    fn deregister(&self, poll: &crate::Poll) -> io::Result<()> {
        poll.use_registry(|r| self.0.deregister(r))
    }
}

impl fmt::Debug for TcpStream {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

pub struct TcpListener(mio::net::TcpListener);

impl TcpListener {
    pub fn bind(addr: &SocketAddr) -> io::Result<TcpListener> {
        Ok(TcpListener(mio::net::TcpListener::bind(*addr)?))
    }

    #[deprecated(since = "0.6.13", note = "use from_std instead")]
    #[cfg(feature = "with-deprecated")]
    #[doc(hidden)]
    pub fn from_listener(listener: net::TcpListener, _: &SocketAddr)
                         -> io::Result<TcpListener> {
        TcpListener::from_std(listener)
    }

    pub fn from_std(listener: net::TcpListener) -> io::Result<TcpListener> {
        Ok(TcpListener(mio::net::TcpListener::from_std(listener)?))
    }

    pub fn accept(&self) -> io::Result<(TcpStream, SocketAddr)> {
        self.0.accept().map(|(s, addr)| (TcpStream(s), addr))
    }

    pub fn accept_std(&self) -> io::Result<(net::TcpStream, SocketAddr)> {
        self.0.accept_std()
    }

    /// Returns the local socket address of this listener.
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.0.local_addr()
    }

    pub fn try_clone(&self) -> io::Result<TcpListener> {
        Ok(TcpListener(self.0.try_clone()?))
    }

    pub fn set_ttl(&self, ttl: u32) -> io::Result<()> {
        self.0.set_ttl(ttl)
    }

    pub fn ttl(&self) -> io::Result<u32> {
        self.0.ttl()
    }

    pub fn set_only_v6(&self, _only_v6: bool) -> io::Result<()> {
        unreachable!();
    }

    pub fn only_v6(&self) -> io::Result<bool> {
        unreachable!();
    }

    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        self.0.take_error()
    }
}

impl crate::Evented for TcpListener {
    fn register(&self, poll: &crate::Poll, token: crate::Token, interest: crate::Ready, _opts: crate::PollOpt) -> io::Result<()> {
        poll.use_registry(|r| self.0.register(r, mio::Token(token.0), convert_ready_to_interests(interest).unwrap()))
    }

    fn reregister(&self, poll: &crate::Poll, token: crate::Token, interest: crate::Ready, _opts: crate::PollOpt) -> io::Result<()> {
        poll.use_registry(|r| self.0.reregister(r, mio::Token(token.0), convert_ready_to_interests(interest).unwrap()))
    }

    fn deregister(&self, poll: &crate::Poll) -> io::Result<()> {
        poll.use_registry(|r| self.0.deregister(r))
    }
}

impl fmt::Debug for TcpListener {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

#[cfg(all(unix, not(target_os = "fuchsia")))]
use std::os::unix::io::{IntoRawFd, AsRawFd, FromRawFd, RawFd};

#[cfg(all(unix, not(target_os = "fuchsia")))]
impl IntoRawFd for TcpStream {
    fn into_raw_fd(self) -> RawFd {
        self.0.into_raw_fd()
    }
}

#[cfg(all(unix, not(target_os = "fuchsia")))]
impl AsRawFd for TcpStream {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

#[cfg(all(unix, not(target_os = "fuchsia")))]
impl FromRawFd for TcpStream {
    unsafe fn from_raw_fd(fd: RawFd) -> TcpStream {
        TcpStream(mio::net::TcpStream::from_raw_fd(fd))
    }
}

#[cfg(all(unix, not(target_os = "fuchsia")))]
impl IntoRawFd for TcpListener {
    fn into_raw_fd(self) -> RawFd {
        self.0.into_raw_fd()
    }
}

#[cfg(all(unix, not(target_os = "fuchsia")))]
impl AsRawFd for TcpListener {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

#[cfg(all(unix, not(target_os = "fuchsia")))]
impl FromRawFd for TcpListener {
    unsafe fn from_raw_fd(fd: RawFd) -> TcpListener {
        TcpListener(mio::net::TcpListener::from_raw_fd(fd))
    }
}