use std::{sync::Once, time::Duration};
use sal_core::dbg::Dbg;
use testing::stuff::max_test_duration::TestDuration;
use debugging::session::{DebugSession, LogLevel};

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
/// Testing [CsvTable::load]
#[test]
fn test_load() {
    DebugSession::new().filter(LogLevel::Debug).init().unwrap();
    init_once();
    init_each();
    log::debug!("");
    let dbg = Dbg::own("CsvTable-test-load");
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(20));
    test_duration.run().unwrap();
    let test_data = [
        (01, "src/tests/unit/services/frdm_service/csv_table/csv_table_test.csv"),
    ];
    for (step, path) in test_data {
        log::debug!("{dbg} | Step {step} | Reading file '{path}'...");
        let csv = super::CsvTable::load(&dbg, path).unwrap();
        log::debug!("{dbg} | Step {step} | Header: {:?}", csv.header);
        log::debug!("{dbg} | Step {step} | Rows [{}]:", csv.len());
        let header: Vec<&String> = csv.header.iter().map(|f| &f.key).collect();
        for (i, row) in csv.iter().enumerate() {
            let values: Vec<f64> = header.iter().map(|field| {
                let cell: f64 = row.cell(field).unwrap_or(f64::NAN);
                cell
            }).collect();
            log::debug!("{dbg} | [{i}]: {:?}", values);
        }
    }
    test_duration.exit();
}
