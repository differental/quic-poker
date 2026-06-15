use quinn::crypto::rustls::NoInitialCipherSuite;

pub enum NetError {
    Connect(quinn::ConnectError),
    Connection(quinn::ConnectionError),
    Write(quinn::WriteError),
    Read(quinn::ReadToEndError),
    Rustls(rustls::Error),
    Cipher(NoInitialCipherSuite),
    #[cfg(feature = "dev")]
    RcGen(rcgen::Error),
    Io(std::io::Error),
    Decode(/* protocol's error type */),
    Config(/* rustls / endpoint setup error */),
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
        Neterror::RcError(value)
    }
}

impl From<std::io::Error> for NetError {
    fn from(value: std::io::Error) -> Self {
        NetError::Io(value)
    }
}
