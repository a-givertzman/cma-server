//!
//! # Определение дефектов грузоподъемного каната по его изображениям
//!
//! Алгоритм основан на анализе изображения каната, а точнее контуров каната
//! Для выявления дефектов изображение каната нормализуется (авто гамма, авто контраст),
//! затем изображение бинаризуется с использованием регулируемого значения порога бинаризации
//!
//! ## Виды определяемых дефектов
//! - Расширение (симметричная деформация с увеличением среднего диаметра)
//! - Сужение (симметричная деформация с уменьшением среднего диаметра)
//! - Холмик (выпуклость на одной стороне изображения каната)
//! - Канавка (углубление на одной стороне изображения каната)
//!
//! ## Процесс
//! - Из конфигурации получаем геометрию стрел, параметры настройки камер и сегментирования каната
//! ```yaml
//!     rope-defect:
//!         tables:
//!             defect: 'public.frdm_defect'
//!             defect-image: 'public.frdm_defect_image'
//!         segment: 100 mm             # Whole rope will divided by the segments for the Camera defect detection, recomended: `segment length = camera.width * 0.10..0.20`
//!         segment-threshold: 5 mm     # Acceptable camera position error in relation to exact segment position
//!         camera-offset: 5.5 m        # camera position from the begin of the rope (hook side)
//!         defect-detection:
//!             ...
//!         camera Camera1:
//!             ...
//!         camera Camera2:
//!             ...
//!     rope-deprecation:
//!         rope:
//!             width: 35 mm            # Diameter of the rome
//!             length: 3000 m          # Total working length of the rope
//!             winch-length: 2985 m    # Length of the rope on the winch drum in the parking position, when rope pos is zero
//!             segment: 100 mm         # Whole rope will divided by the segments for the Depreciation Rate calculation, use less to incrise accuracy
//!             pos: point real 'Winch.EncoderBR2'     # meters, current rope position (длина каната размотанного с барабана считая от парковочного)
//!             load: point real 'Winch.Load'          # tonn, current rope load
//!         booms:
//!             - Main-Boom:
//!                 l1: 0.0 mm                  # Растояние от продольной оси стрелы до точки A (оси ее поворота), константа
//!                 l2: 0.0 mm                  # Растояние по продольной оси стрелы от точки D (корня стрелы) до точки A (оси ее поворота), константа
//!                 l3: 0.0 mm                  # Расстояние от точки A (ось поворота) стрелы до продольной оси предыдущей стрелы (до ГСК для первой срелы), константа
//!                 l4: 10330.0 mm              # Расстояние от точки A (ось поворота) стрелы до перпендикуляра к продольной оси через точку G предыдущей стрелы (до ГСК для первой срелы), константа
//!                 len: 11200.0 mm                                         # length of the boom
//!                 angle: point real 'App/MultiQueue/Load.MainBoomAngle'   # degrees, current angle of the boom (relative axis)
//!             - Rotary-Boom:
//!                 l1: 0.0 mm                  # Растояние от продольной оси стрелы до точки A (оси ее поворота), константа
//!                 l2: 0.0 mm                  # Растояние по продольной оси стрелы от точки D (корня стрелы) до точки A (оси ее поворота), константа
//!                 l3: 0.0 mm                  # Расстояние от точки A (ось поворота) стрелы до продольной оси предыдущей стрелы (до ГСК для первой срелы), константа
//!                 l4: 0.0 mm                  # Расстояние от точки A (ось поворота) стрелы до перпендикуляра к продольной оси через точку G предыдущей стрелы (до ГСК для первой срелы), константа
//!                 len: 7984.1 mm                                          # length of the rotary boom
//!                 angle: point real 'App/MultiQueue/Load.RotaryBoomAngle' # degrees, current angle of the boom (relative axis)
//!         blocks:
//!             - 1:
//!                 lf: 1830.0 mm,  710.0 mm    # Растояние (x, y) от **конца** стрелы до оси блока, мм
//!                 d: 844.0 mm                 # Диаметры блоков, мм
//!                 schemes: TopTop             # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
//!                 bind: Fixed                 # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
//!             - 2:
//!                 lf: 308.0 mm, 1090.0 mm     # Растояние (x, y) от **конца** стрелы до оси блока, мм
//!                 d: 816.0 mm                 # Диаметры блоков, мм
//!                 schemes: TopTop             # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
//!                 bind: Boom 0                # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
//! ```
//!
//! - Канат с учетом заданных настроек условно нарезается на сегменты, размер сегмента должен быть 85..95% от ширины кадра (размер кадра вдоль каната)
//! - В процессе перемещения каната приложение получает изменения длины вытравленной его части,
//!     - Пересчитываем и получаем положение сегмента относительно положения камеры
//! - В моменты когда камера проходит границу между соседними сегментами, берем кадр с камеры и считаем дефекты
//! - Граница между сегментами задается с допустимой погрешность `segment-threshold`
mod rope_defect_conf;
mod rope_defect;
mod rope;
mod tables_conf;

pub use rope_defect_conf::*;
pub use rope_defect::*;
pub use rope::*;
use tables_conf::*;
