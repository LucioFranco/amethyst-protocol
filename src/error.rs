use failure;
use std::result;

pub type Error = failure::Error;
pub type Result<T> = result::Result<T, Error>;

#[derive(Fail, Debug)]
pub enum NetworkError {
    // TODO: write more informative error
    #[fail(display = "Lock posioned")]
    AddConnectionToManagerFailed,
    #[fail(display = "TcpStream clone failed")]
    TcpStreamCloneFailed,
    #[fail(display = "TcpStream failed to take the rx channel in outgoing loop")]
    TcpSteamFailedTakeRx,
    #[fail(display = "TCP client connections hash was poisoned")]
    TcpClientConnectionsHashPoisoned,
    #[fail(display = "The lock for a specific TCP client was poisoned")]
    TcpClientLockFailed,
    #[fail(display = "The fragment of an packet is invalid")]
    FragmentInvalid,
    #[fail(display = "The parsing of the header went wrong")]
    HeaderParsingFailed,
    #[fail(display = "Something went wrong when sending")]
    SendFailed
}
