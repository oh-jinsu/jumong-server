use std::io;

use tokio::net::TcpStream;

pub trait Reader {
    fn try_read_packet_buf(&self, buf: &mut [u8]) -> io::Result<usize>;
}

impl Reader for TcpStream {
    fn try_read_packet_buf(&self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.try_read(&mut buf[..2])?;

        if n == 0 {
            return Err(io::Error::from(io::ErrorKind::UnexpectedEof));
        }

        if n < 2 {
            return Err(io::Error::new(io::ErrorKind::Other, "buffer too short"));
        }

        let len = usize::from(u16::from_le_bytes([buf[0], buf[1]]));

        if len > buf.len() {
            return Err(io::Error::new(io::ErrorKind::Other, "packet too big"));
        }

        let n = self.try_read(&mut buf[..len])?;

        if n == 0 {
            return Err(io::Error::new(io::ErrorKind::Other, "unexpected eof"));
        }

        if n != len {
            return Err(io::Error::new(io::ErrorKind::Other, "length not matched"));
        }

        Ok(n)
    }
}

pub fn wrap_tcp_packet(buf: &[u8]) -> Vec<u8> {
    [
        &u16::try_from(buf.len()).unwrap().to_le_bytes() as &[u8],
        buf,
    ]
    .concat()
}
