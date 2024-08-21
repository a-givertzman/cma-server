///
/// Definition of Frame Type
/// - ST - Single transmission
/// - MT - Multiple transmissions
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FrameType {
    BinReqSt = 0x5000,
    BinResSt = 0xD000,
    BinReqMt = 0x5400,
    BinResMt = 0xD400,
    AsciiReqSt = 0x35303030,  // '5000'
    AsciiResSt = 0x44303030,  // 'D000'
    AsciiReqMt = 0x35343030,  // '5400'
    AsciiResMt = 0x44343030,  // 'D400'
}
///
/// Definition of Processor Number or as 
/// documentation refer Destination unit I/O No.
#[allow(dead_code)]
pub enum ProcessorNumber {
    CpuActive = 0x03D0,
    CpuStandby = 0x03D1,
    CpuTypeA = 0x03D2,
    CpuTypeB = 0x03D3,
    Cpu1 = 0x03E0,
    Cpu2 = 0x03E1,
    Cpu3 = 0x03E2,
    Cpu4 = 0x03E3,
    CpuDefault = 0x03FF,
}
///
/// Definition of Monitoring timer value.
/// Can be increased, check documentation.
#[allow(dead_code)]
pub enum TimerValue {
    WaitForever = 0x0000,
}
///
/// List of SLMP Commands.
#[allow(dead_code)]
pub enum SlmpCommand {
    // Device
    DeviceRead = 0x0401,
    DeviceWrite = 0x1401,
    DeviceReadRandom = 0x0403,
    DeviceWriteRandom = 0x1402,
    DeviceEntryMonitorDevice = 0x0801,
    DeviceExecuteMonitor = 0x0802,
    DeviceReadBlock = 0x0406,
    DeviceWriteBlock = 0x1406,
    // Memory
    MemoryRead = 0x0613,
    MemoryWrite = 0x1613,
    // ExtendUnit
    ExtendUnitRead = 0x0601,
    ExtendUnitWrite = 0x1601,
    // RemoteControl
    RemoteRun = 0x1001,
    RemoteStop = 0x1002,
    RemotePause = 0x1003,
    RemoteLatchClear = 0x1005,
    RemoteReset = 0x1006,
    RemoteReadTypeName = 0x0101,
    // Drive
    DriveReadDiskState = 0x0205,
    DriveDefrag = 0x1207,
    // File
    FileReadFileInfo = 0x0201,
    FileReadFileInfoDetail = 0x0202,
    FileReadFileInfoFileNumberUsage = 0x0204,
    FileChangeFileInfo = 0x1204,
    FileSearch = 0x0203,
    FileReadAccessTypeA = 0x0206,
    FileWriteAccessTypeA = 0x1203,
    FileLockControl = 0x0808,
    FileCopyAccessTypeA = 0x1206,
    FileCopyAccessTypeB = 0x1824,
    FileDeleteAccessTypeA = 0x1205,
    FileDeleteAccessTypeB = 0x1822,
    FileReadDirectoryFile = 0x1810,
    FileSearchDirectoryFile = 0x1811,
    FileCreateNewFileAccessTypeA = 0x1202,
    FileCreateNewFileAccessTypeB = 0x1820,
    FileChangeFileState = 0x1825,
    FileChangeFileDate = 0x1826,
    FileOpenFile = 0x1827,
    FileReadAccessTypeB = 0x1828,
    FileWriteAccessTypeB = 0x1829,
    FileCloseFile = 0x182A,
    // Test
    SelfTest = 0x0619,
    // ClearError
    ClearErrorCode = 0x1617,
    // RemotePassword
    PasswordLock = 0x1630,
    PasswordUnlock = 0x1631,
    // OnDemand
    Ondemand = 0x2101,
    // NodeConnect
    NodeSearch = 0x0E30,
    IpAddressSet = 0x0E31,
    // ParameterSetting
    DeviceInfoCompare = 0x0E32,
    ParameterGet = 0x0E33,
    ParameterSet = 0x0E34,
    ParameterSetStart = 0x0E35,
    ParameterSetEnd = 0x0E36,
    ParameterSetCancel = 0x0E3A,
    // NodeMonitoring
    StatusRead = 0x0E44,
    CommunicationSettingGet = 0x0E45,
    StatusRead2 = 0x0E53,
}
///
/// Subcommand telling whether we communicate with registers 
/// with binary/word/double word data or whether
/// we communicate with CPU.
#[allow(dead_code)]
pub enum SlmpSubCommand {
    SubBit1 = 0x0001,
    SubBit3 = 0x0003,
    SubWord0 = 0x0000,
    SubWord2 = 0x0002,
    SubCpu0 = 0x0080,
    SubCpu2 = 0x0082,
}
