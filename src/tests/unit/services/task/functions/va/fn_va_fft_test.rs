#[cfg(test)]

mod fn_va_fft {
    use plotters::prelude::*;
    use std::{f64::consts::PI, sync::Once, time::Duration};
    use rustfft::{num_complex::{Complex, ComplexFloat}, FftPlanner};
    use testing::stuff::max_test_duration::TestDuration;
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};

    use crate::{core_::aprox_eq::aprox_eq::AproxEq, services::task::nested_function::va::unit_circle::UnitCircle};
    ///
    ///
    static INIT: Once = Once::new();
    ///
    /// once called initialisation
    fn init_once() {
        INIT.call_once(|| {
            // implement your initialisation code to be called only once for current test file
        })
    }
    ///
    /// returns:
    ///  - ...
    fn init_each() -> () {}
    ///
    /// Testing FFT
    #[test]
    fn next() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        log::debug!("");
        let self_id = "test";
        log::debug!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();
        // Sampling freq
        let freq = 10_000;
        let mut sampling_unit_circle = UnitCircle::new(freq);
        let unit_circle_1 = UnitCircle::new(2000);
        let unit_circle_2 = UnitCircle::new(300);
        let mut buf = vec![];
        let mut fft_result;
        let fft_len = 24;
        let fft = FftPlanner::new().plan_fft_forward(fft_len);
        // let pif2 = 2.0 * PI * (freq as f64);
        let mut input = vec![];
        let mut series = vec![];
        // Time of sampling, sec
        let mut t = 0.0;
        let until = 0.5;
        while t < until {
            (t, _, _) = sampling_unit_circle.next();
            let value: Complex<f64> = [
                unit_circle_1.at_with(t, 10.0),
                unit_circle_2.at_with(t, 30.0),
            ].iter().map(|(_angle, complex)| complex).sum();
            buf.push(value);
            // println!("x: {}  |  y: {}", t, round(value.abs(), 3));
            input.push(
                (t, value.abs())
            );
            if buf.len() >= fft_len {
                fft_result = buf;
                fft.process(&mut fft_result);
                let fft_scalar: Vec<f64> = fft_result.iter().map(|complex| {
                    round(complex.abs(), 3)
                }).collect();
                println!("{}  |  {:?}", t, fft_scalar);
                // series.push(
                //     fft_scalar.into_iter().map(|y|, )
                // );
                buf = vec![];
            }
        }
        let input_len = input.len();
        series.push(
            input,
        );
        plot(input_len / 2, series).unwrap();
        let delta = (freq as f64) / (fft_len as f64);
        let mut f = vec![0.0];
        while f.last().unwrap().to_owned() < (freq as f64) {
            f.push(
                round(f.last().unwrap() + delta, 3)
            );
        }
        println!("{:?}", f);
        test_duration.exit();
    }
    ///
    /// Returns float rounded to the specified digits
    fn round(value: f64, digits: usize) -> f64 {
        let factor = 10.0f64.powi(digits as i32);
        (value * factor).round() / factor
    }
    ///
    /// 
    fn plot(x_lables: usize, series: Vec<Vec<(f64, f64)>>) -> Result<(), Box<dyn std::error::Error>> {
        let colors = colors(7);
        let root = BitMapBackend::new("src/tests/unit/services/task/functions/va/plot_1.png", (100000, 1024)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let root = root.margin(10, 10, 10, 10);
        // After this point, we should be able to construct a chart context
        let mut chart = ChartBuilder::on(&root)
            // Set the caption of the chart
            .caption("Plot", ("sans-serif", 40).into_font())
            // Set the size of the label region
            // .x_label_area_size(20)
            // .y_label_area_size(40)
            // Finally attach a coordinate on the drawing area and make a chart context
            .build_cartesian_2d(0f64..3f64, -3f64..50f64)?;
    
        // Then we can draw a mesh
        chart
            .configure_mesh()
            // We can customize the maximum number of labels allowed for each axis
            .x_labels(x_lables)
            .y_labels(10)
            // We can also change the format of the label text
            .y_label_formatter(&|x| format!("{:.3}", x))
            .draw()?;
    
        // And we can draw something in the drawing area
        for (i, ser) in series.into_iter().enumerate() {
            chart.draw_series(LineSeries::new(
                ser.clone(),
                colors[i],
                // vec![(0.0, 0.0), (5.0, 5.0), (8.0, 7.0)],
                // &RED,
            ))?;
            // Similarly, we can draw point series
            chart.draw_series(PointSeries::of_element(
                ser,
                3,
                colors[i],
                &|c, s, st| {
                    return EmptyElement::at(c)    // We want to construct a composed element on-the-fly
                    + Circle::new((0,0),s,st.filled()) // At this point, the new pixel coordinate is established
                    + Text::new(format!("{:?}", c), (10, 0), ("sans-serif", 10).into_font());
                },
            ))?;
        }
        root.present()?;
        Ok(())
    }
    ///
    /// 
    fn colors(sat: usize) -> Vec<RGBColor> {
        let colors = [
            ["FFCCCC", "FFC0C0", "FF9999", "FF8080", "FF6666", "FF4040", "FF3333", "FF0000"],
            ["FFE5CC", "FFE0C0", "FFCC99", "FFC080", "FFB266", "FFA040", "FF9933", "FF8000"],
            ["FFFFCC", "FFFFC0", "FFFF99", "FFFF80", "FFFF66", "FFFF40", "FFFF33", "FFFF00"],
            ["FFFFE5", "FFFFE0", "FFFFCC", "FFFFC0", "FFFFB2", "FFFFA0", "FFFF99", "FFFF80"],
            ["E5FFCC", "E0FFC0", "CCFF99", "C0FFA0", "B2FF66", "A0FF40", "99FF33", "80FF00"],
            ["CCFFCC", "C0FFC0", "99FF99", "80FF80", "66FF66", "40FF40", "33FF33", "00FF00"],
            ["E5FFE5", "E0FFE0", "CCFFCC", "C0FFC0", "B2FFB2", "A0FFA0", "99FF99", "80FF80"],
            ["CCE5CC", "C0E0C0", "99CC99", "80C080", "66B266", "40A040", "339933", "008000"],
            ["CCFFE5", "C0FFE0", "99FFCC", "80FFC0", "66FFB2", "40FFA0", "33FF99", "00FF80"],
            ["CCFFFF", "C0FFFF", "99FFFF", "80FFFF", "66FFFF", "40FFFF", "33FFFF", "00FFFF"],
            ["E5FFFF", "E0FFFF", "CCFFFF", "C0FFFF", "B2FFFF", "A0FFFF", "99FFFF", "80FFFF"],
            ["CCE5E5", "C0E0E0", "99CCCC", "80C0C0", "66B2B2", "40A0A0", "339999", "008080"],
            ["CCE5FF", "C0E0FF", "99CCFF", "80C0FF", "66B2FF", "40A0FF", "3399FF", "0080FF"],
            ["CCCCFF", "C0C0FF", "9999FF", "8080FF", "6666FF", "4040FF", "3333FF", "0000FF"],
            ["CCCCE5", "C0C0E0", "9999CC", "8080C0", "6666B2", "4040A0", "333399", "000080"],
            ["E5E5FF", "E0E0FF", "CCCCFF", "C0C0FF", "B2B2FF", "A0A0FF", "9999FF", "8080FF"],
            ["E5CCFF", "E0C0FF", "CC99FF", "C080FF", "B266FF", "A040FF", "9933FF", "8000FF"],
            ["E5CCE5", "E0C0E0", "CC99CC", "C080C0", "B266B2", "A040A0", "993399", "800080"],
            ["FFCCFF", "FFC0FF", "FF99FF", "FF80FF", "FF66FF", "FF40FF", "FF33FF", "FF00FF"],
            ["FFE5FF", "FFE0FF", "FFCCFF", "FFC0FF", "FFB2FF", "FFA0FF", "FF99FF", "FF80FF"],
            ["FFCCE5", "FFC0E0", "FF99CC", "FF80C0", "FF66B2", "FF40A0", "FF3399", "FF0080"],
            ["FFE5E5", "FFE0E0", "FFCCCC", "FFC0C0", "FFB2B2", "FFA0A0", "FF9999", "FF8080"],
            ["E5CCCC", "E0C0C0", "CC9999", "C08080", "B26666", "A04040", "993333", "800000"],
            ["E5E5CC", "E0E0C0", "CCCC99", "C0C080", "B2B266", "A0A040", "999933", "808000"],
            ["E5E5E5", "E0E0E0", "CCCCCC", "C0C0C0", "B2B2B2", "A0A0A0", "999999", "808080"],
        ];
        colors.map(|colors| {
            decode_hex(colors[sat])
        }).to_vec()
    } 
    pub fn decode_hex(s: &str) -> RGBColor {
        let r = u8::from_str_radix(&s[0..2], 16).unwrap();
        let g = u8::from_str_radix(&s[2..4], 16).unwrap();
        let b = u8::from_str_radix(&s[4..6], 16).unwrap();
        RGBColor(r, g, b)
    }
    // pub fn encode_hex(bytes: &[u8]) -> String {
    //     let mut s = String::with_capacity(bytes.len() * 2);
    //     for &b in bytes {
    //         write!(&mut s, "{:02x}", b).unwrap();
    //     }
    //     s
    // }
}
