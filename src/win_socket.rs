use std::{mem::MaybeUninit, net::SocketAddrV4};

use windows_sys::Win32::Networking::WinSock::*;

pub struct WinSocket {
    sock: SOCKET,
}

impl WinSocket {
    pub fn init() {
        unsafe {
            let mut wsa_data: WSADATA = std::mem::zeroed();
            if WSAStartup(u16::from_be_bytes([2, 2]), &mut wsa_data) != 0 {
                panic!("WSAStartup failed");
            }
        }
    }

    pub fn shutdown() {
        unsafe {
            WSACleanup();
        }
    }

    pub fn new() -> Result<Self, String> {
        let sock = unsafe { socket(AF_INET as i32, SOCK_DGRAM, IPPROTO_UDP) };
        if sock == INVALID_SOCKET {
            let err = format!("socket() failed: {}", unsafe { WSAGetLastError() });
            return Err(err);
        }
        Ok(Self { sock })
    }

    pub fn setsockopt_randomize_port(&self, enable: bool) -> Result<(), String> {
        let enable: u32 = if enable { 1 } else { 0 };
        if unsafe {
            setsockopt(
                self.sock,
                SOL_SOCKET,
                SO_RANDOMIZE_PORT,
                &enable as *const _ as *const u8,
                std::mem::size_of::<u32>() as i32,
            )
        } == SOCKET_ERROR
        {
            let err = format!("setsockopt(SO_RANDOMIZE_PORT) failed: {}", unsafe {
                WSAGetLastError()
            });
            return Err(err);
        }
        Ok(())
    }

    pub fn connect(&self, addr: SocketAddrV4) -> Result<(), String> {
        let remote_addr = SOCKADDR_IN {
            sin_family: AF_INET,
            sin_port: addr.port().to_be(),
            sin_addr: IN_ADDR {
                S_un: IN_ADDR_0 {
                    S_addr: u32::from_ne_bytes(addr.ip().octets()),
                },
            },
            sin_zero: [0; 8],
        };

        if unsafe {
            connect(
                self.sock,
                &remote_addr as *const _ as *const SOCKADDR,
                std::mem::size_of::<SOCKADDR_IN>() as i32,
            )
        } == SOCKET_ERROR
        {
            let err = format!("connect() failed: {}", unsafe { WSAGetLastError() });
            return Err(err);
        }
        Ok(())
    }

    pub fn localport(&self) -> Result<u16, String> {
        let mut addr = MaybeUninit::<SOCKADDR_IN>::uninit();
        let mut addr_len = std::mem::size_of::<SOCKADDR_IN>() as i32;

        if unsafe {
            getsockname(
                self.sock,
                &mut addr as *mut _ as *mut SOCKADDR,
                &mut addr_len,
            )
        } != 0
        {
            let err = format!("getsockname() failed: {}", unsafe { WSAGetLastError() });
            return Err(err);
        }

        // getsockname() api succeeded. addr is valid now
        let addr = unsafe { addr.assume_init() };

        let port = u16::from_be(addr.sin_port);
        Ok(port)
    }
}

impl Drop for WinSocket {
    fn drop(&mut self) {
        unsafe { closesocket(self.sock) };
    }
}
