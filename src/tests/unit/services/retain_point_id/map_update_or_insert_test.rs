#[cfg(test)]

mod map_update_or_insert {
    use indexmap::IndexMap;
    use log::debug;
    use std::{collections::HashMap, sync::Once, time::Duration};
    use testing::stuff::max_test_duration::TestDuration;
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::tests::unit::services::retain_point_id::map_update_or_insert_test::{PointConf, RetainedPointConfig};
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
    /// Testing such functionality / behavior
    #[test]
    fn test() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "test";
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();
        let mut cache: IndexMap<String, Vec<PointConf>> = IndexMap::new();
        let test_data = HashMap::from([
            ("Service3".to_owned(), [
                PointConf { id: 0, name: "Service3.Point1".into() },
                PointConf { id: 0, name: "Service3.Point2".into() },
                PointConf { id: 0, name: "Service3.Point5".into() },
            ]),
            ("Service2".into(), [
                PointConf { id: 0, name: "Service2.Point1".into() },
                PointConf { id: 0, name: "Service2.Point2".into() },
                PointConf { id: 0, name: "Service2.Point5".into() },
            ]),
        ]);
        let mut retained = HashMap::from([
            ("Service1".into(), HashMap::from([
                ("Service1.Point1".into(), RetainedPointConfig { id: 11 }),
                ("Service1.Point2".into(), RetainedPointConfig { id: 12 }),
                ("Service1.Point3".into(), RetainedPointConfig { id: 13 }),
            ])),
            ("Service2".into(), HashMap::from([
                ("Service2.Point1".into(), RetainedPointConfig { id: 21 }),
                ("Service2.Point2".into(), RetainedPointConfig { id: 22 }),
                ("Service2.Point3".into(), RetainedPointConfig { id: 23 }),
            ])),
        ]);
        let mut update_retained = false;
        for (owner, points) in test_data {
            for mut point in points {
                let retained_clone = retained.clone();
                let retained_point = retained
                    .entry(owner.to_owned())
                    .or_insert(HashMap::new())
                    .entry(point.name.clone())
                    .or_insert_with(|| {
                        let id = retained_clone.values().map(|v| {
                            v.values()
                            .map(|conf| conf.id)
                            .max().unwrap_or(0)
                        })
                        .max()
                        .map_or(0, |id| id + 1);
                        update_retained = true;
                        RetainedPointConfig { id }
                    });
                point.id = retained_point.id;
                cache
                    .entry(owner.to_owned())
                    .or_insert(vec![])
                    .push(point.clone());
            }
        }
        debug!("cache: {:#?}", cache);
        // assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        test_duration.exit();
    }
}
///
/// The Mok to PointConfig
#[derive(Clone, Debug)]
struct PointConf {
    id: usize,
    name: String,
}
///
/// Simple container storing the ID
#[derive(Clone, Debug)]
struct RetainedPointConfig {
    pub id: usize,
    // pub name: String,
}