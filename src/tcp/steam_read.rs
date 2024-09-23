use sal_sync::services::entity::{object::Object, point::point::Point};
use std::{fmt::Debug, io::BufReader, net::TcpStream};
use crate::core_::net::connection_status::ConnectionStatus;
use super::tcp_stream_write::OpResult;
///
/// 
pub trait StreamRead<T: Send, E>: Object + Debug + Send {
    fn read(&mut self) -> Result<T, E>;
}
///
/// 
pub trait TcpStreamRead: Object + Debug + Send {
    fn read(&mut self, tcp_stream: &mut BufReader<TcpStream>) -> ConnectionStatus<OpResult<Point, String>, String>;
}
