use serde::{Deserializer, Deserialize};

///
/// Representation of the csv data header
#[allow(unused)]
#[derive(Debug, Clone, serde::Deserialize)]
pub struct CsvHeader {
    pub step: String,
    pub a21: String,       // a21, град, угол стрелы относительно предыдущей
    pub a22: String,       // a22, град, угол стрелы относительно предыдущей
    pub x_nok: String,     // Xнок, мм, - координата крайнего блока (5 или 6 блок в зависимости от переброса каната)
    pub y_nok: String,     // Yнок, мм, - координата крайнего блока (5 или 6 блок в зависимости от переброса каната)
    pub xg: String,        // XG, мм, - координаты конца стрелы
    pub yg: String,        // YG, мм, - координаты конца стрелы
    pub x_kp: String,       // Xкп, мм, - координаты Крюковой Подвески
    pub y_kp: String,       // Yкп, мм, - координаты Крюковой Подвески
    pub lpodv_min: String, // lподв_min, мм, - длина подвеса
    pub lkan_прям: String, // Lкан_прям, мм, - сумма длин прямолинейных участков каната
    pub lkan_дуг: String,  // Lкан_дуг, мм, - сумма длин дуг каната
    pub lкан_леб: String,  // Lкан_леб,мм, - длина каната на лебедке
    pub x1: String,        // X1, мм, - координаты блока 1
    pub y1: String,        // Y1, мм, - координаты блока 1
    pub x2: String,        // X2, мм, - координаты блока 2
    pub y2: String,        // Y2, мм, - координаты блока 2
    pub x3: String,        // X3, мм, - координаты блока 3
    pub y3: String,        // Y3, мм, - координаты блока 3
    pub x4: String,        // X4, мм, - координаты блока 4
    pub y4: String,        // Y4, мм, - координаты блока 4
    pub x5: String,        // X5, мм, - координаты блока 5
    pub y5: String,        // Y5, мм, - координаты блока 5
    pub x6: String,        // X6, мм, - координаты блока 6
    pub y6: String,        // Y6, мм, - координаты блока 6
}
impl Default for CsvHeader {
    fn default() -> Self {
        Self { step: Default::default(), a21: Default::default(), a22: Default::default(), x_nok: Default::default(), y_nok: Default::default(), xg: Default::default(), yg: Default::default(), x_kp: Default::default(), y_kp: Default::default(), lpodv_min: Default::default(), lkan_прям: Default::default(), lkan_дуг: Default::default(), lкан_леб: Default::default(), x1: Default::default(), y1: Default::default(), x2: Default::default(), y2: Default::default(), x3: Default::default(), y3: Default::default(), x4: Default::default(), y4: Default::default(), x5: Default::default(), y5: Default::default(), x6: Default::default(), y6: Default::default() }
    }
}
///
/// Representation of the csv data single row
#[allow(unused)]
#[derive(Debug, Clone, serde::Deserialize)]
pub struct CsvRecord {
    pub step: usize,
    #[serde(deserialize_with = "replace_f64")]
    pub a21: f64,               // a21, град, угол стрелы относительно предыдущей
    #[serde(deserialize_with = "replace_f64")]
    pub a22: f64,               // a22, град, угол стрелы относительно предыдущей
    #[serde(deserialize_with = "replace_f64")]
    pub x_nok: f64,             // Xнок, мм,
    #[serde(deserialize_with = "replace_f64")]
    pub y_nok: f64,             // Yнок, мм,
    #[serde(deserialize_with = "replace_f64")]
    pub xg: f64,                // XG, мм,
    #[serde(deserialize_with = "replace_f64")]
    pub yg: f64,                // YG, мм,
    #[serde(deserialize_with = "replace_f64")]
    pub x_hook: f64,            // Xкп, мм,
    #[serde(deserialize_with = "replace_f64")]
    pub y_hook: f64,            // Yкп, мм,
    #[serde(deserialize_with = "replace_f64")]
    pub lrope_hook_min: f64,    // lподв_min, мм,
    #[serde(deserialize_with = "replace_f64")]
    pub lrope_straight: f64,    // Lкан_прям, мм,
    #[serde(deserialize_with = "replace_f64")]
    pub lrope_ark: f64,         // Lкан_дуг, мм,
    #[serde(deserialize_with = "replace_f64")]
    pub lrope_winch: f64,       // Lкан_леб,мм,
    #[serde(deserialize_with = "replace_f64")]
    pub x1: f64,                // X1, мм,
    #[serde(deserialize_with = "replace_f64")]
    pub y1: f64,                // Y1, мм,
    #[serde(deserialize_with = "replace_f64")]
    pub x2: f64,                // X2, мм,
    #[serde(deserialize_with = "replace_f64")]
    pub y2: f64,                // Y2, мм,
    #[serde(deserialize_with = "replace_f64")]
    pub x3: f64,                // X3, мм,
    #[serde(deserialize_with = "replace_f64")]
    pub y3: f64,                // Y3, мм,
    #[serde(deserialize_with = "replace_f64")]
    pub x4: f64,                // X4, мм,
    #[serde(deserialize_with = "replace_f64")]
    pub y4: f64,                // Y4, мм,
    #[serde(deserialize_with = "replace_f64")]
    pub x5: f64,                // X5, мм,
    #[serde(deserialize_with = "replace_f64")]
    pub y5: f64,                // Y5, мм,
    #[serde(deserialize_with = "replace_f64")]
    pub x6: f64,                // X6, мм,
    #[serde(deserialize_with = "replace_f64")]
    pub y6: f64,                // Y6, мм
    #[serde(deserialize_with = "replace_f64")]
    pub wrap_l1: f64,              // Lдуг 1,мм
    #[serde(deserialize_with = "replace_f64")]
    pub wrap_l2: f64,              // Lдуг 2,мм
    #[serde(deserialize_with = "replace_f64")]
    pub wrap_l3: f64,              // Lдуг 3,мм
    #[serde(deserialize_with = "replace_f64")]
    pub wrap_l4: f64,              // Lдуг 4,мм
    #[serde(deserialize_with = "replace_f64")]
    pub wrap_l5: f64,              // Lдуг 5,мм
    #[serde(deserialize_with = "replace_f64")]
    pub wrap_l6: f64,              // Lдуг 6,мм
    #[serde(deserialize_with = "replace_f64")]
    pub lrope_straight1: f64,            // Lпрям1, мм - длина прямого участка каната между барабаном и первым блоком
    #[serde(deserialize_with = "replace_f64")]
    pub lrope_straight2: f64,            // Lпрям2, мм - длина прямого участка каната между первым и вторым блоками
    #[serde(deserialize_with = "replace_f64")]
    pub lrope_straight3: f64,            // Lпрям3, мм
    #[serde(deserialize_with = "replace_f64")]
    pub lrope_straight4: f64,            // Lпрям4, мм
    #[serde(deserialize_with = "replace_f64")]
    pub lrope_straight5: f64,            // Lпрям5, мм
    #[serde(deserialize_with = "replace_f64")]
    pub lrope_straight6: f64,            // Lпрям6, мм
    #[serde(deserialize_with = "replace_f64")]
    pub rope_alpha1: f64,         // Угол_кан1, град
    #[serde(deserialize_with = "replace_f64")]
    pub rope_alpha2: f64,         // Угол_кан2, град
    #[serde(deserialize_with = "replace_f64")]
    pub rope_alpha3: f64,         // Угол_кан3, град
    #[serde(deserialize_with = "replace_f64")]
    pub rope_alpha4: f64,         // Угол_кан4, град
    #[serde(deserialize_with = "replace_f64")]
    pub rope_alpha5: f64,         // Угол_кан5, град
    #[serde(deserialize_with = "replace_f64")]
    pub rope_alpha6: f64,         // Угол_кан6, град
    #[serde(deserialize_with = "replace_f64")]
    pub xсход: f64,             // Xсход
    #[serde(deserialize_with = "replace_f64")]
    pub xсход2: f64,            // Xсход2
    #[serde(deserialize_with = "replace_f64")]
    pub xсход3: f64,            // Xсход3
    #[serde(deserialize_with = "replace_f64")]
    pub xсход4: f64,            // Xсход4
    #[serde(deserialize_with = "replace_f64")]
    pub xсход5: f64,            // Xсход5
    #[serde(deserialize_with = "replace_f64")]
    pub xсход6: f64,            // Xсход6
    #[serde(deserialize_with = "replace_f64")]
    pub yсход: f64,             // Yсход
    #[serde(deserialize_with = "replace_f64")]
    pub yсход2: f64,            // Yсход2
    #[serde(deserialize_with = "replace_f64")]
    pub yсход3: f64,            // Yсход3
    #[serde(deserialize_with = "replace_f64")]
    pub yсход4: f64,            // Yсход4
    #[serde(deserialize_with = "replace_f64")]
    pub yсход5: f64,            // Yсход5
    #[serde(deserialize_with = "replace_f64")]
    pub yсход6: f64,            // Yсход6
    #[serde(deserialize_with = "replace_f64")]
    pub xвход: f64,             // Xвход
    #[serde(deserialize_with = "replace_f64")]
    pub xвход2: f64,            // Xвход2
    #[serde(deserialize_with = "replace_f64")]
    pub xвход3: f64,            // Xвход3
    #[serde(deserialize_with = "replace_f64")]
    pub xвход4: f64,            // Xвход4
    #[serde(deserialize_with = "replace_f64")]
    pub xвход5: f64,            // Xвход5
    #[serde(deserialize_with = "replace_f64")]
    pub xвход6: f64,            // Xвход6
    #[serde(deserialize_with = "replace_f64")]
    pub yвход: f64,             // Yвход
    #[serde(deserialize_with = "replace_f64")]
    pub yвход2: f64,            // Yвход2
    #[serde(deserialize_with = "replace_f64")]
    pub yвход3: f64,            // Yвход3
    #[serde(deserialize_with = "replace_f64")]
    pub yвход4: f64,            // Yвход4
    #[serde(deserialize_with = "replace_f64")]
    pub yвход5: f64,            // Yвход5
    #[serde(deserialize_with = "replace_f64")]
    pub yвход6: f64,            // Yвход6
    #[serde(deserialize_with = "replace_f64")]
    pub wrap_alpha1: f64,              // угол обхв
    #[serde(deserialize_with = "replace_f64")]
    pub wrap_alpha2: f64,              // угол обхв2
    #[serde(deserialize_with = "replace_f64")]
    pub wrap_alpha3: f64,              // угол обхв3
    #[serde(deserialize_with = "replace_f64")]
    pub wrap_alpha4: f64,              // угол обхв4
    #[serde(deserialize_with = "replace_f64")]
    pub wrap_alpha5: f64,              // угол обхв5
    #[serde(deserialize_with = "replace_f64")]
    pub wrap_alpha6: f64,              // угол обхв6
    #[serde(deserialize_with = "replace_f64")]
    pub t01: f64,   // от точки схода каната с барабана до крюка
    #[serde(deserialize_with = "replace_f64")]
    pub t02: f64,
    #[serde(deserialize_with = "replace_f64")]
    pub t03: f64,
    #[serde(deserialize_with = "replace_f64")]
    pub t04: f64,
    #[serde(deserialize_with = "replace_f64")]
    pub t05: f64,
    #[serde(deserialize_with = "replace_f64")]
    pub t06: f64,
    #[serde(deserialize_with = "replace_f64")]
    pub t07: f64,
    #[serde(deserialize_with = "replace_f64")]
    pub t08: f64,
    #[serde(deserialize_with = "replace_f64")]
    pub t09: f64,
    #[serde(deserialize_with = "replace_f64")]
    pub t10: f64,
    #[serde(deserialize_with = "replace_f64")]
    pub t11: f64,
    #[serde(deserialize_with = "replace_f64")]
    pub t12: f64,
    #[serde(deserialize_with = "replace_f64")]
    pub t13: f64,   // от крюка до точки схода каната с барабана
    #[serde(deserialize_with = "replace_f64")]
    pub t14: f64,
    #[serde(deserialize_with = "replace_f64")]
    pub t15: f64,
    #[serde(deserialize_with = "replace_f64")]
    pub t16: f64,
    #[serde(deserialize_with = "replace_f64")]
    pub t17: f64,
    #[serde(deserialize_with = "replace_f64")]
    pub t18: f64,
    #[serde(deserialize_with = "replace_f64")]
    pub t19: f64,
    #[serde(deserialize_with = "replace_f64")]
    pub t20: f64,
    #[serde(deserialize_with = "replace_f64")]
    pub t21: f64,
    #[serde(deserialize_with = "replace_f64")]
    pub t22: f64,
    #[serde(deserialize_with = "replace_f64")]
    pub t23: f64,
    #[serde(deserialize_with = "replace_f64")]
    pub t24: f64,
    #[serde(deserialize_with = "replace_f64")]
    pub l_rope: f64,            // Суммарная длина каната
    #[serde(deserialize_with = "replace_f64")]
    pub l_rope_winch_geom: f64, // Длина каната на лебедке из-за геометрии стрел
    #[serde(deserialize_with = "replace_f64")]
    pub l_rope_winch: f64,      // Длина каната на лебедке
    #[serde(deserialize_with = "replace_f64")]
    pub pos: f64,               // Позиция каната, мм (канат вытравленный вращением лебедки)
    #[serde(deserialize_with = "replace_f64")]
    reserv01: f64,
    #[serde(deserialize_with = "replace_f64")]
    reserv02: f64,
}
/// ### Replace `Empty => f64::NAN`
fn replace_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    if value.is_empty() {
        Ok(f64::NAN)
    } else {
        value.parse().map_err(|err| serde::de::Error::custom(err))
    }
}
