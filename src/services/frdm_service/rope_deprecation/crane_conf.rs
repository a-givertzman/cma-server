use sal_core::dbg::Dbg;
use sal_sync::services::{conf::{ConfTree, ConfTreeGet}, entity::Name};
use crate::services::frdm_service::{BlockConf, BoomConf, RopeConf};
///
/// ## The configuration parameters for the rope
/// 
/// ### Example:
/// ```yaml
/// crane:
///     rope:
///         width: 35 mm            # Diameter of the rome
///         length: 3000 m          # Total working length of the rope
///         winch-length: 2985 m    # Length of the rope on the winch drum in the parking position, when rope pos is zero
///         segment: 100 mm         # Whole rope will divided by the segments for the Depreciation Rate calculation, use less to incrise accuracy
///         pos: point real 'Winch.EncoderBR2'     # meters, current rope position (длина каната размотанного с барабана считая от парковочного)
///         load: point real 'Winch.Load'          # tonn, current rope load 
///     booms:
///         - Main-Boom:
///             l1: 0.0 mm                  # Растояние от продольной оси стрелы до точки A (оси ее поворота), константа
///             l2: 0.0 mm                  # Растояние по продольной оси стрелы от точки D (корня стрелы) до точки A (оси ее поворота), константа
///             l3: 0.0 mm                  # Расстояние от точки A (ось поворота) стрелы до продольной оси предыдущей стрелы (до ГСК для первой срелы), константа
///             l4: 10330.0 mm              # Расстояние от точки A (ось поворота) стрелы до перпендикуляра к продольной оси через точку G предыдущей стрелы (до ГСК для первой срелы), константа
///             len: 11200.0 mm                                         # length of the boom
///             angle: point real 'App/MultiQueue/Load.MainBoomAngle'   # degrees, current angle of the boom (relative axis)
///         - Rotary-Boom:
///             l1: 0.0 mm                  # Растояние от продольной оси стрелы до точки A (оси ее поворота), константа
///             l2: 0.0 mm                  # Растояние по продольной оси стрелы от точки D (корня стрелы) до точки A (оси ее поворота), константа
///             l3: 0.0 mm                  # Расстояние от точки A (ось поворота) стрелы до продольной оси предыдущей стрелы (до ГСК для первой срелы), константа
///             l4: 0.0 mm                  # Расстояние от точки A (ось поворота) стрелы до перпендикуляра к продольной оси через точку G предыдущей стрелы (до ГСК для первой срелы), константа
///             len: 7984.1 mm                                          # length of the rotary boom
///             angle: point real 'App/MultiQueue/Load.RotaryBoomAngle' # degrees, current angle of the boom (relative axis)
///     blocks:
///         - 1:
///             lf: 1830.0 mm,  710.0 mm    # Растояние (x, y) от **конца** стрелы до оси блока, мм
///             d: 844.0 mm                 # Диаметры блоков, мм
///             scheme: TopTop             # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
///             bind: Fixed                 # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
///         - 2:
///             lf: 308.0 mm, 1090.0 mm     # Растояние (x, y) от **конца** стрелы до оси блока, мм
///             d: 816.0 mm                 # Диаметры блоков, мм
///             scheme: TopTop             # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
///             bind: Boom 0                # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
///         - 3:
///             lf: -6550.0 mm, 1730.0 mm   # Растояние (x, y) от **конца** стрелы до оси блока, мм
///             d: 816.0 mm                 # Диаметры блоков, мм
///             scheme: TopTop             # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
///             bind: Boom 1                # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
///         - 4:
///             lf: -1121.0 mm, 973.0 mm    # Растояние (x, y) от **конца** стрелы до оси блока, мм
///             d: 816.0 mm                 # Диаметры блоков, мм
///             scheme: TopBottom          # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
///             bind: Boom 2                # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
///         - 5:
///             lf: 267.0 mm, 860.0 mm      # Растояние (x, y) от **конца** стрелы до оси блока, мм
///             d: 816.0 mm                 # Диаметры блоков, мм
///             scheme: BottomTop          # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
///             bind: Boom 3                # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
///         - 6:
///             lf: 136.0 mm, -35.0 mm      # Растояние (x, y) от **конца** стрелы до оси блока, мм
///             d: 816.0 mm                 # Диаметры блоков, мм
///             scheme: TopTop             # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
///             bind: Boom 4                # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
///         - 7:
///             lf: 0.0 mm, 0.0 mm          # Растояние (x, y) от **конца** стрелы до оси блока, мм
///             d: 0.0 mm                   # Диаметры блоков, мм
///             scheme: TopTop             # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
///             bind: Hook                  # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct CraneConf {
    pub booms: Vec<(String, BoomConf)>,
    pub blocks: Vec<(String, BlockConf)>,
    pub rope: RopeConf,
}
//
// 
impl CraneConf {
    ///
    /// Returns [CraneConf] built from `ConfTree`:
    pub fn new(parent: impl Into<String>, conf: ConfTree) -> Self {
        let parent = parent.into();
        let me = "CraneConf";
        let dbg = Dbg::new(&parent, me);
        log::trace!("{}.new | conf: {:?}", dbg, conf);
        let name = Name::new(parent, me);
        log::trace!("{}.new | name: {:?}", dbg, name);
        let booms: &Vec<serde_yaml::Value> = conf.get("booms").expect(&format!("{dbg}.new | 'booms' - not found or wrong config"));
        let booms = booms.iter().map(|boom| {
            let (key, boom) = boom.as_mapping()
                .expect(&format!("{dbg}.new | boom's config have to be a Map, but found: {:#?}", boom))
                .iter()
                .next()
                .expect(&format!("{dbg}.new | 'boom' config can't be empty, but found: {:#?}", boom));
            let boom = ConfTree::new(key.as_str().unwrap(), boom.to_owned());
            (boom.key.clone(), BoomConf::new(&name, boom))
        }).collect();
        log::trace!("{dbg}.new | booms: {:#?}", booms);

        let blocks: &Vec<serde_yaml::Value> = conf.get("blocks").expect(&format!("{dbg}.new | 'blocks' - not found or wrong config"));
        let blocks = blocks.iter().map(|block| {
            let (key, block) = block.as_mapping()
                .expect(&format!("{dbg}.new | block's config have to be a Map, but found: {:#?}", block))
                .iter()
                .next()
                .expect(&format!("{dbg}.new | 'block' config can't be empty, but found: {:#?}", block));
            let key = if key.is_number() {
                format!("{}", key.as_u64().expect(&format!("{dbg}.new | Block's key expected positive number or string")))
            } else if key.is_string() {
                format!("{}", key.as_str().expect(&format!("{dbg}.new | Block's key expected positive number or string")))
            } else {
                panic!("{dbg}.new | Block's key expected positive number or string");
            };
            let block = ConfTree::new(key, block.to_owned());
            (block.key.clone(), BlockConf::new(&name, block))
        }).collect();
        log::trace!("{dbg}.new | blocks: {:#?}", blocks);
        let rope = conf.get("rope").expect(&format!("{dbg}.new | 'rope' - not found or wrong config"));
        let rope = RopeConf::new(&name, rope);
        log::trace!("{dbg}.new | rope: {:#?}", rope);
        Self {
            booms,
            blocks,
            rope,
        }
    }
}
