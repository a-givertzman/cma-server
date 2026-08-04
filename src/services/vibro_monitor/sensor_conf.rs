use super::UdpClientConf;

/// Параметры датчика виброаналитики цифровой обработки
#[derive(Debug, Clone, PartialEq)]
pub struct SensorConf {
    /// Уникальный идентификатор целевого механизма.
    pub target: String,
    /// Номер канала в АЦП (0..255). 0 - первый канал.
    pub channel: usize,
    /// Сигнал скорости вращения вала механизма
    pub rpm: super::InputKind<f64>,
    /// Параметры связи с датчиком.
    pub connection: UdpClientConf,
    /// Параметры сбора данных с АЦП и цифровой обработки и виброаналитики.
    pub dsp: vibro_core::Conf,
}
