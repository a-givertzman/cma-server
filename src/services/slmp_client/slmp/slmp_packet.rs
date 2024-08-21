use log::trace;
use crate::services::slmp_client::slmp::c_slmp_const::{ProcessorNumber, SlmpCommand, SlmpSubCommand, TimerValue};
use super::{c_slmp_const::FrameType, c_slmp_info::CSlmpInfo, device_code::DeviceCode};
//
//
extern "C" {
    ///
    /// A Function for making packet stream from the SLMP_Info structure */
    pub fn SLMP_MakePacketStream (
        ulFrameType : std::ffi::c_ulong,         // unsigned long
        slmp_info   : *const CSlmpInfo,          // const SLMP_INFO * p
        pucStream   : *const std::ffi::c_uchar,  // unsigned char * pucStream
    ) -> std::ffi::c_int;
}
///
///        Initialize and create SLMPPacket structure.
///        Load C functions from .so file.
///        From SLMPInfoC structure is created packet with create_stream method.
///
///        :param ulFrameType: ASCII/BIN + REQUEST/RESPONSE + ST/MT
///        :param usSerialNumber: used only with MT(multiple transmission) frame types,
///                               marks request and response with this number
///        :param usNetNumber: network number(1 to 239), can be fined in parameters of target device (e.g. SLMPNWNO)
///        :param usNodeNumber: station number (1 to 120), can be fined in parameters of target device (e.g. SLMPNDID)
///        :param usProcNumber: destination unit I/O number
///        :param usDataLength: length of data from usTimer to end of pucData, use function get_data_length
///        :param usTimer: monitoring timer
///        :param usCommand: SLMP command (e.g. SLMP_COMMAND_DEVICE_READ, SLMP_COMMAND_SELF_TEST)
///        :param usSubCommand: Sub command -> 0x0001/0x0003 = per bit,
///                                            0x0000/0x0002 = per word,
///                                            0x0080/0x0082 = per word on CPU
///        :param usEndCode: used mostly in response
///        :param pucData: data which obtains device number, number of devices, its like arguments of used command
pub struct SlmpPacket {
    id: String,
    device_code: DeviceCode,
    offset: u32,
    length: u16,
}
//
//
impl SlmpPacket {
    ///
    /// Creates new SLMP packet
    /// - device_code - SLMP Device code (Sm, Sd, X, Y, M, L, F, V, B, D, W)
    /// - offset - address offset
    /// - length - bytes to be writen / requested
    pub fn new(parent: impl Into<String>, device_code: DeviceCode, offset: u32, length: u16) -> Self {
        Self {
            id: format!("{}/SlmpPacket", parent.into()),
            device_code,
            offset,
            length,
        }
    }
    /// Gets usDataLength for PacketCreator.
    /// TODO is there difference with length of serial number ? Check and add +2 to BIN and +4 to ASCII
    fn data_length(ul_frame_type: FrameType, puc_data: &[u8]) -> u16 {
        let length = puc_data.len() as u16;
        match ul_frame_type {
            FrameType::BinReqSt => length + 6,
            FrameType::BinResSt => length + 2,
            FrameType::BinReqMt => length + 6,
            FrameType::BinResMt => length + 2,
            FrameType::AsciiReqSt => length + 12,
            FrameType::AsciiResSt => length + 4,
            FrameType::AsciiReqMt => length + 12,
            FrameType::AsciiResMt => length + 4,
        }
    }
    ///
    /// Returns SLMP packet for Read from device, ready to send over ethrnet 
    pub fn read_packet(&self, frame_type: FrameType) -> Result<Vec<u8>, String> {
        self.build(frame_type, None, SlmpCommand::DeviceRead)
    }
    ///
    /// Returns SLMP packet for Write to device, ready to send over ethrnet 
    pub fn write_packet(&self, frame_type: FrameType, write_data: &[u8]) -> Result<Vec<u8>, String> {
        self.build(frame_type, Some(write_data), SlmpCommand::DeviceWrite)
    }
    ///
    /// Returns SLMP packet (read / write) ready to send over ethrnet 
    fn build(&self, frame_type: FrameType, write_data: Option<&[u8]>, us_command: SlmpCommand) -> Result<Vec<u8>, String> {
        let slmp_packet = SlmpPacketData::new(self.device_code, self.offset, self.length);
        let puc_data = slmp_packet.build(write_data);
        // let frame_type = FrameType::BinReqSt;
        let us_serial_number = 0;
        let us_net_number = 0;
        let us_node_number = 0xFF;
        let us_data_length = Self::data_length(frame_type, &puc_data);
        let us_end_code = 0x0000;
        trace!("{}.build | puc_data: {:?}", self.id, puc_data);
        let slmp_info = CSlmpInfo::new(
            frame_type,
            us_serial_number,
            us_net_number,
            us_node_number,
            ProcessorNumber::CpuDefault,
            us_data_length,
            TimerValue::WaitForever,
            us_command,
            SlmpSubCommand::SubWord0,
            us_end_code,
            &puc_data,
        );
        let packet = &mut [0; 1518];
        let slmp_make_packet_result = unsafe { SLMP_MakePacketStream(
            frame_type as u64, 
            &slmp_info, 
            packet.as_mut_ptr(),
        ) };
        trace!("{}.build | slmp_make_packet_result: {}", self.id, slmp_make_packet_result);
        if slmp_make_packet_result == 0 {
            Ok(self.trim_packet(frame_type, us_data_length, packet))
        } else {
            Err(format!("{}.build | SLMP_MakePacketStream returns error code -1", self.id))
        }
    }
    ///
    /// Trim packet
    fn trim_packet(&self, ul_frame_type: FrameType, us_data_length: u16, packet: &[u8]) -> Vec<u8> {
        // let result = &mut [0; 1518];
        let package = match ul_frame_type {
            FrameType::BinReqSt => &packet[..(9 + us_data_length)  as usize],
            FrameType::BinResSt => panic!("{}.build | Not implemented - SLMP_FTYPE_BIN_RES_ST is not a request type", self.id),
            FrameType::BinReqMt =>  &packet[..(13 + us_data_length) as usize],
            FrameType::BinResMt => panic!("{}.build | Not implemented - SLMP_FTYPE_BIN_RES_MT is not a request type", self.id),
            FrameType::AsciiReqSt => &packet[..(18 + us_data_length) as usize],
            FrameType::AsciiResSt => panic!("{}.build | Not implemented - SLMP_FTYPE_ASCII_RES_ST is not a request type", self.id),
            FrameType::AsciiReqMt => &packet[..(26 + us_data_length) as usize],
            FrameType::AsciiResMt => panic!("{}.build | Not implemented - SLMP_FTYPE_ASCII_RES_MT is not a request type", self.id),
        };
        package.to_owned()
    }
}
///
/// Structure used in the SlpmPacket
/// for preparing request puc data 
struct SlmpPacketData {
    device_code: DeviceCode,
    offset: u32,
    length: u16,
}
//
//
impl SlmpPacketData {
    ///
    /// 
    pub fn new(device_code: DeviceCode, offset: u32, length: u16) -> Self {
        Self { device_code, offset, length }
    }
    // ///
    // /// 
    // fn copy_into(from: &mut [u8], pos: usize, to: &[u8]) {
    //     let buf = &mut from[pos..];
    //     let len = to.len().min(buf.len());
    //     buf[..len].copy_from_slice(&to[..len]);
    // }
    ///
    /// 
    pub fn build(&self, write_data: Option<&[u8]>) -> Vec<u8> {
        let slmp_head_device = &self.offset.to_le_bytes()[..3];
        // Mitsubushi controller word consists of 2 bytes. 
        // So we need to divide package size in bytes by 2 to get words count.
        let word_count = self.length / 2;
        let slmp_word_count = (word_count as u16).to_le_bytes();
        match write_data {
            Some(bytes) => [slmp_head_device, &[self.device_code as u8], &slmp_word_count, bytes].concat(),
            None =>               [slmp_head_device, &[self.device_code as u8], &slmp_word_count].concat(),
        }
    }
}