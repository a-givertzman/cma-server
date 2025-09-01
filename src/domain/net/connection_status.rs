use std::io::ErrorKind;
///
/// Used in the TspStream handling like Result
pub enum ConnectionStatus<T, E> {
    Active(T),
    Closed(E),
}
///
/// 
pub enum SocketState {
    Active,
    Closed,
    /// Connection is Active, but configured timeout exceeded
    Timeout,
}
//
//
impl SocketState {
    ///
    /// Returns Current connection status on error kind
    pub fn match_error_kind(kind: ErrorKind) -> Self {
        match kind {
            std::io::ErrorKind::NotFound => todo!(),
            std::io::ErrorKind::PermissionDenied => SocketState::Closed,
            std::io::ErrorKind::ConnectionRefused => SocketState::Closed,
            std::io::ErrorKind::ConnectionReset => SocketState::Closed,
            // std::io::ErrorKind::HostUnreachable => Status::Closed,
            // std::io::ErrorKind::NetworkUnreachable => Status::Closed,
            std::io::ErrorKind::ConnectionAborted => SocketState::Closed,
            std::io::ErrorKind::NotConnected => SocketState::Closed,
            std::io::ErrorKind::AddrInUse => SocketState::Closed,
            std::io::ErrorKind::AddrNotAvailable => SocketState::Closed,
            // std::io::ErrorKind::NetworkDown => Status::Closed,
            std::io::ErrorKind::BrokenPipe => SocketState::Closed,
            std::io::ErrorKind::AlreadyExists => todo!(),
            std::io::ErrorKind::WouldBlock => SocketState::Timeout,
            // std::io::ErrorKind::NotADirectory => todo!(),
            // std::io::ErrorKind::IsADirectory => todo!(),
            // std::io::ErrorKind::DirectoryNotEmpty => todo!(),
            // std::io::ErrorKind::ReadOnlyFilesystem => todo!(),
            // std::io::ErrorKind::FilesystemLoop => todo!(),
            // std::io::ErrorKind::StaleNetworkFileHandle => todo!(),
            std::io::ErrorKind::InvalidInput => todo!(),
            std::io::ErrorKind::InvalidData => todo!(),
            std::io::ErrorKind::TimedOut => todo!(),
            std::io::ErrorKind::WriteZero => todo!(),
            // std::io::ErrorKind::StorageFull => todo!(),
            // std::io::ErrorKind::NotSeekable => todo!(),
            // std::io::ErrorKind::FilesystemQuotaExceeded => todo!(),
            // std::io::ErrorKind::FileTooLarge => todo!(),
            // std::io::ErrorKind::ResourceBusy => todo!(),
            // std::io::ErrorKind::ExecutableFileBusy => todo!(),
            // std::io::ErrorKind::Deadlock => todo!(),
            // std::io::ErrorKind::CrossesDevices => todo!(),
            // std::io::ErrorKind::TooManyLinks => todo!(),
            // std::io::ErrorKind::InvalidFilename => todo!(),
            // std::io::ErrorKind::ArgumentListTooLong => todo!(),
            std::io::ErrorKind::Interrupted => todo!(),
            std::io::ErrorKind::Unsupported => todo!(),
            std::io::ErrorKind::UnexpectedEof => todo!(),
            std::io::ErrorKind::OutOfMemory => todo!(),
            std::io::ErrorKind::Other => todo!(),
            _ => SocketState::Closed,
        }
    }
}