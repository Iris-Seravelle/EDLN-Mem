use nix::sys::socket::{ControlMessageOwned, MsgFlags, recvmsg};
use serde::{Deserialize, Serialize};
use std::io::IoSliceMut;
use std::os::unix::io::RawFd;
use std::os::unix::net::UnixListener;

#[derive(Serialize, Deserialize, Debug)]
pub struct Registration {
    pub addr: usize,
    pub len: usize,
    pub page_size: usize,
}

pub struct IpcServer {
    listener: UnixListener,
}

impl IpcServer {
    pub fn new(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let _ = std::fs::remove_file(path);
        let listener = UnixListener::bind(path)?;
        listener.set_nonblocking(true)?;
        Ok(IpcServer { listener })
    }

    pub fn listener_fd(&self) -> RawFd {
        use std::os::unix::io::AsRawFd;
        self.listener.as_raw_fd()
    }

    pub fn accept_registration(
        &self,
    ) -> Result<Option<(Registration, RawFd)>, Box<dyn std::error::Error>> {
        let (stream, _) = match self.listener.accept() {
            Ok(s) => s,
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => return Ok(None),
            Err(e) => return Err(e.into()),
        };

        use std::os::unix::io::AsRawFd;
        let fd = stream.as_raw_fd();

        let mut buf = [0u8; 1024];
        let mut iov = [IoSliceMut::new(&mut buf)];
        let mut cmsg_buf = nix::cmsg_space!(RawFd);

        let msg = recvmsg::<()>(fd, &mut iov, Some(&mut cmsg_buf), MsgFlags::empty())?;

        let mut received_fd = None;
        for cmsg in msg.cmsgs()? {
            if let ControlMessageOwned::ScmRights(fds) = cmsg {
                received_fd = fds.first().copied();
            }
        }

        let uffd = received_fd.ok_or("No FD received via SCM_RIGHTS")?;

        // Deserialize the registration metadata from the beginning of the buffer
        let registration: Registration = bincode::deserialize(&buf)?;

        Ok(Some((registration, uffd)))
    }
}
