use serde::{Deserialize, Serialize};
use super::UdpClientConf;

/// Параметры датчика виброаналитики цифровой обработки
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SensorConf {
    /// Уникальный идентификатор целевого механизма.
    pub target: String,
    /// Номер канала в АЦП (0..255). 0 - первый канал.
    pub channel: usize,
    // /// Общее количество каналов в АЦП
    // pub channels: usize,
    /// Параметры связи с датчиком.
    pub connection: UdpClientConf,
    /// Параметры сбора данных с АЦП и цифровой обработки и виброаналитики.
    pub dsp: vibro_core::Conf,
}
