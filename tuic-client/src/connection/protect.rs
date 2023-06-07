use std::io;
use std::net::{ToSocketAddrs, UdpSocket};

pub fn my_bind_udp<A: ToSocketAddrs>(addr: A) -> io::Result<UdpSocket> {
    let socket = UdpSocket::bind(addr)?;
    #[cfg(target_os = "android")]
    {
        use std::os::fd::AsRawFd;
        use passfd::FdPassingExt;
        let try_protect = || -> Result<(), io::Error> {
            let unix = std::os::unix::net::UnixStream::connect("protect_path")?;
            unix.send_fd(socket.as_raw_fd())?;
            unix.shutdown(std::net::Shutdown::Both)?;
            Ok(())
        };
        if let Err(err) = try_protect() {
            log::debug!("try protect failed: {err}")
        }
    }
    Ok(socket)
}
