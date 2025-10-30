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
///
/// Representation of the csv data single row
#[allow(unused)]
#[derive(Debug, Clone, serde::Deserialize)]
pub struct CsvRecord {
    pub step: usize,
    pub a21: f64,               // a21, град, угол стрелы относительно предыдущей
    pub a22: f64,               // a22, град, угол стрелы относительно предыдущей
    pub x_nok: f64,             // Xнок, мм,
    pub y_nok: f64,             // Yнок, мм,
    pub xg: f64,                // XG, мм,
    pub yg: f64,                // YG, мм,
    pub x_hook: f64,            // Xкп, мм,
    pub y_hook: f64,            // Yкп, мм,
    pub lrope_hook_min: f64,    // lподв_min, мм,
    pub lrope_straight: f64,    // Lкан_прям, мм,
    pub lrope_ark: f64,         // Lкан_дуг, мм,
    pub lrope_winch: f64,       // Lкан_леб,мм,
    pub x2: f64,                // X2, мм,
    pub y2: f64,                // Y2, мм,
    pub x3: f64,                // X3, мм,
    pub y3: f64,                // Y3, мм,
    pub x4: f64,                // X4, мм,
    pub y4: f64,                // Y4, мм,
    pub x5: f64,                // X5, мм,
    pub y5: f64,                // Y5, мм,
    pub x6: f64,                // X6, мм,
    pub y6: f64,                // Y6, мм
    pub wrap_l1: f64,              // Lдуг 1,мм
    pub wrap_l2: f64,              // Lдуг 2,мм
    pub wrap_l3: f64,              // Lдуг 3,мм
    pub wrap_l4: f64,              // Lдуг 4,мм
    pub wrap_l5: f64,              // Lдуг 5,мм
    pub wrap_l6: f64,              // Lдуг 6,мм
    pub Lпрям1: f64,            // Lпрям1, мм
    pub Lпрям2: f64,            // Lпрям2, мм
    pub Lпрям3: f64,            // Lпрям3, мм
    pub Lпрям4: f64,            // Lпрям4, мм
    pub Lпрям5: f64,            // Lпрям5, мм
    pub Lпрям6: f64,            // Lпрям6, мм
    pub Угол_кан1: f64,         // Угол_кан1, град
    pub Угол_кан2: f64,         // Угол_кан2, град
    pub Угол_кан3: f64,         // Угол_кан3, град
    pub Угол_кан4: f64,         // Угол_кан4, град
    pub Угол_кан5: f64,         // Угол_кан5, град
    pub Угол_кан6: f64,         // Угол_кан6, град
    pub Xсход: f64,             // Xсход
    pub Xсход2: f64,            // Xсход2
    pub Xсход3: f64,            // Xсход3
    pub Xсход4: f64,            // Xсход4
    pub Xсход5: f64,            // Xсход5
    pub Xсход6: f64,            // Xсход6
    pub Yсход: f64,             // Yсход
    pub Yсход2: f64,            // Yсход2
    pub Yсход3: f64,            // Yсход3
    pub Yсход4: f64,            // Yсход4
    pub Yсход5: f64,            // Yсход5
    pub Yсход6: f64,            // Yсход6
    pub Xвход: f64,             // Xвход
    pub Xвход2: f64,            // Xвход2
    pub Xвход3: f64,            // Xвход3
    pub Xвход4: f64,            // Xвход4
    pub Xвход5: f64,            // Xвход5
    pub Xвход6: f64,            // Xвход6
    pub Yвход: f64,             // Yвход
    pub Yвход2: f64,            // Yвход2
    pub Yвход3: f64,            // Yвход3
    pub Yвход4: f64,            // Yвход4
    pub Yвход5: f64,            // Yвход5
    pub Yвход6: f64,            // Yвход6
    pub wrap_alpha1: f64,              // угол обхв
    pub wrap_alpha2: f64,              // угол обхв2
    pub wrap_alpha3: f64,              // угол обхв3
    pub wrap_alpha4: f64,              // угол обхв4
    pub wrap_alpha5: f64,              // угол обхв5
    pub wrap_alpha6: f64,              // угол обхв6
}
