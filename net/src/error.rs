use protocol::SerdeError;
use quinn::crypto::rustls::NoInitialCipherSuite;

#[derive(Debug)]
pub enum NetError {
    Connect(quinn::ConnectError),
    Connection(quinn::ConnectionError),
    Write(quinn::WriteError),
    Read(quinn::ReadToEndError),
    ClosedStream(quinn::ClosedStream),
    Rustls(rustls::Error),
    Cipher(NoInitialCipherSuite),
    #[cfg(feature = "dev")]
    RcGen(rcgen::Error),
    Io(std::io::Error),
    Serde(SerdeError),
}

impl From<quinn::ConnectError> for NetError {
    fn from(value: quinn::ConnectError) -> Self {
        NetError::Connect(value)
    }
}

impl From<quinn::ConnectionError> for NetError {
    fn from(value: quinn::ConnectionError) -> Self {
        NetError::Connection(value)
    }
}

impl From<quinn::WriteError> for NetError {
    fn from(value: quinn::WriteError) -> Self {
        NetError::Write(value)
    }
}

impl From<quinn::ReadToEndError> for NetError {
    fn from(value: quinn::ReadToEndError) -> Self {
        NetError::Read(value)
    }
}

impl From<quinn::ClosedStream> for NetError {
    fn from(value: quinn::ClosedStream) -> Self {
        NetError::ClosedStream(value)
    }
}

impl From<rustls::Error> for NetError {
    fn from(value: rustls::Error) -> Self {
        NetError::Rustls(value)
    }
}

impl From<NoInitialCipherSuite> for NetError {
    fn from(value: NoInitialCipherSuite) -> Self {
        NetError::Cipher(value)
    }
}

#[cfg(feature = "dev")]
impl From<rcgen::Error> for NetError {
    fn from(value: rcgen::Error) -> Self {
        NetError::RcGen(value)
    }
}

impl From<std::io::Error> for NetError {
    fn from(value: std::io::Error) -> Self {
        NetError::Io(value)
    }
}

impl From<SerdeError> for NetError {
    fn from(value: SerdeError) -> Self {
        NetError::Serde(value)
    }
}

impl std::fmt::Display for NetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetError::Connect(e) => write!(f, "failed to initiate connection: {e}"),
            NetError::Connection(e) => write!(f, "connection error: {e}"),
            NetError::Write(e) => write!(f, "failed to write to stream: {e}"),
            NetError::Read(e) => write!(f, "failed to read from stream: {e}"),
            NetError::ClosedStream(e) => write!(f, "stream closed: {e}"),
            NetError::Rustls(e) => write!(f, "TLS error: {e}"),
            NetError::Cipher(e) => write!(f, "invalid initial cipher suite: {e}"),
            #[cfg(feature = "dev")]
            NetError::RcGen(e) => write!(f, "certificate generation error: {e}"),
            NetError::Io(e) => write!(f, "I/O error: {e}"),
            NetError::Serde(e) => write!(f, "encode/decode error: {e}"),
        }
    }
}

impl std::error::Error for NetError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            NetError::Connect(e) => Some(e),
            NetError::Connection(e) => Some(e),
            NetError::Write(e) => Some(e),
            NetError::Read(e) => Some(e),
            NetError::ClosedStream(e) => Some(e),
            NetError::Rustls(e) => Some(e),
            NetError::Cipher(e) => Some(e),
            #[cfg(feature = "dev")]
            NetError::RcGen(e) => Some(e),
            NetError::Io(e) => Some(e),
            NetError::Serde(e) => Some(e),
        }
    }
}
