use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use function_name::named;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{
    services::{entity::{Name, Object}, Service, ServiceWaiting, RECV_TIMEOUT},
    sync::{channel::RecvTimeoutError, Handles},
    thread_pool::Scheduler,
};
use crate::{domain::{Sender, unbounded}, err, infra::ApiClient, services::frdm_service::{Bendings, BlockArcs, Blocks, Booms, Deprecation, Inputs, RopeDeprecationConf, RopeSections}};

///
/// ## Rope deprecation rate
/// - Counting passes rope via cargo block
/// - Including:
///     - Rope width
///     - Block sizes
///     - Current rope load
pub struct RopeDeprecation {
    name: Name,
    conf: RopeDeprecationConf,
    inputs: Arc<Inputs>,
    api_client: Arc<ApiClient>,
    scheduler: Scheduler,
    handles: Arc<Handles<()>>,
    exit: Arc<AtomicBool>,
    dbg: Dbg,
}
//
impl RopeDeprecation {
    ///
    /// - `updates` - callback for share rope position
    pub fn new(
        parent: impl Into<String>,
        conf: RopeDeprecationConf,
        inputs: Arc<Inputs>,
        api_client: Arc<ApiClient>,
        scheduler: Scheduler,
    ) -> Self {
        let name = Name::new(parent, "RopeDeprecation");
        let dbg = Dbg::new(name.parent(), name.me());
        Self {
            name,
            conf,
            inputs,
            api_client,
            scheduler,
            handles: Arc::new(Handles::new(&dbg)),
            exit: Arc::new(AtomicBool::new(false)),
            dbg,
        }
    }
    /// Собирает запрос для добавления пар `(id, deprecation)`
    #[named]
    fn build_sql(aggregated: & Vec<(usize, f64)>, table: &str, sql: &mut String) -> Result<(), Error> {
        use std::fmt::Write;
        sql.clear();
        if aggregated.is_empty() { return Err(err!(Self, "No deprecation pairs found to build sql"))}
        let additional = (120 + aggregated.len() * 25) as isize - sql.len() as isize;
        if additional > 0 {
            sql.reserve(additional as usize);
        }
        let _ = write!(sql, "INSERT INTO {table} (id, deprecation) VALUES ");
        for (slice_ix, deprecation) in aggregated {
            let _ = write!(sql, "({}, {}), ", slice_ix, deprecation);
        }
        if sql.len() > 2 {
            sql.truncate(sql.len() - 2);
        }
        let _ = write!(sql, " ON CONFLICT (id) DO UPDATE \
                SET deprecation = {table}.deprecation + EXCLUDED.deprecation;");
        Ok(())
    }
    /// Копирование из `src` в `result` с агрегацией элементов с одинаковым ix
    fn aggregate_sparse(src: &mut Vec<(usize, f64)>, result: &mut Vec<(usize, f64)>) {
        src.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        result.clear();
        if src.is_empty() { return; }
        // 2. Берем первый элемент как стартовый
        let mut current_ix = src[0].0;
        let mut current_sum = src[0].1;
        // 3. Бежим по остальным элементам (линейный проход за O(N))
        for &(ix, val) in &src[1..] {
            if ix == current_ix {
                // Если индекс тот же — просто суммируем в переменную
                current_sum += val;
            } else {
                // Индекс изменился — сохраняем накопленный результат в буфер
                result.push((current_ix, current_sum));
                // Переключаемся на новый индекс
                current_ix = ix;
                current_sum = val;
            }
        }
        // Запись последней группы
        result.push((current_ix, current_sum));
    }
    /// Собирает алгоритм расчета
    fn build_math(dbg: Dbg, conf: RopeDeprecationConf, inputs: Arc<Inputs>,  tx: Sender<(usize, f64)>) ->  Deprecation<impl Fn(usize, f64)> {
        Deprecation::new(
            &dbg,
            &conf.crane,
            inputs.clone(),
            Bendings::new(
                &dbg,
                &conf.crane.rope,
                BlockArcs::new(
                    &dbg,
                    &conf.crane.rope.segment,
                    RopeSections::new(
                        &dbg,
                        Blocks::new(
                            &dbg,
                            conf.crane.rope.aux_length,
                            &conf.crane.blocks,
                            true,
                            Booms::new(&dbg, &conf.crane.booms, inputs, true),
                        ),
                    ),
                ),
            ),
            move |slice_ix, deprecation| {
                let _ = tx.send((slice_ix, deprecation));
            },
        )
    }
}
//
impl Object for RopeDeprecation {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
impl std::fmt::Debug for RopeDeprecation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RopeDeprecation")
            .field("dbg", &self.dbg)
            .finish()
    }
}
// ///
// /// Used for logging
// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
// enum NotifyState {
//     Start,
//     Exit,
//     SendError,
// }
//
//
impl Service for RopeDeprecation {
    //
    fn run(&self) -> Result<(), Error> {
        log::info!("{}.run | Starting...", self.dbg);
        let dbg = self.dbg.clone();
        let name = self.name.clone();
        let conf = self.conf.clone();
        let service_waiting = ServiceWaiting::new(&name, conf.wait_started);
        let service_release = service_waiting.release();
        let inputs = self.inputs.clone();
        let exit = self.exit.clone();
        let api_client = self.api_client.clone();
        log::debug!("{}.run | Preparing thread...", dbg);
        let (pairs_tx, pairs_rx) = unbounded();
        let mut deprecation = Self::build_math(dbg.clone(), conf.clone(), inputs.clone(), pairs_tx);
        let handle = self.scheduler.spawn(move || {
            let dbg = &dbg;
            let inputs_stream = inputs.listen();
            let conf_table = conf.table.clone();
            service_release.add(Ok(()));
            let mut sql = String::with_capacity(120 + (conf.crane.blocks.len() * 32));
            let mut received = Vec::with_capacity(pairs_rx.len());
            let mut aggregated = Vec::with_capacity(pairs_rx.len());
            while !exit.load(Ordering::Acquire) {
                // log::trace!("{dbg}.run | Receiving events...");
                match inputs_stream.recv_timeout(RECV_TIMEOUT) {
                    Ok(_point) => {
                        // log::trace!("{dbg}.run | Received event: {}: {}", point.name(), point.to_string().as_string().value);
                        deprecation.eval();
                        received.clear();
                        if let Err(err) = pairs_rx.drain_into(&mut received) {
                            log::warn!("{dbg}.run | Can't access accumulated Deprecation results: {:?}", err);
                        }
                        if !received.is_empty() {
                            Self::aggregate_sparse(&mut received, &mut aggregated);
                            log::trace!("{dbg}.run | Deprecation aggregated: {:?}", aggregated.len());
                            if Self::build_sql(&aggregated, &conf_table, &mut sql).is_ok() {
                                // log::trace!("{dbg}.run | Fetching sql: {:?}", sql);
                                api_client.fetch(&sql).then(
                                    |_reply| {
                                        // log::trace!("{dbg}.run | Sql reply: {:?}", reply);
                                    },
                                    |err| log::warn!("{dbg}.run | Fetch error: {:?}", err),
                                );
                            }
                        }
                    }
                    Err(RecvTimeoutError::Timeout) => {}
                    Err(_) => {
                        log::warn!("{dbg}.run | Channel is closed, exiting...");
                        break;
                    }
                }
            }
            log::info!("{dbg}.run | Exit");
        });
        match handle {
            Ok(handle) => {
                self.handles.push(handle);
            }
            Err(err) => {
                let err = Error::new(&self.dbg, "run").pass_with("Start failed", err.to_string());
                log::warn!("{}", err);
                return Err(err);
            }
        }
        let r = match conf.wait_started {
            Some(_) => {
                log::info!("{}.run | Waiting while starting...", self.dbg);
                service_waiting.wait()
            }
            None => Ok(()),
        };
        log::info!("{}.run | Starting - ok", self.dbg);
        r
    }
    //
    fn wait(&self) -> Result<(), Error> {
        self.handles.wait()
    }
    //
    fn is_finished(&self) -> bool {
        self.handles.is_finished()
    }
    //
    fn exit(&self) {
        self.exit.store(true, Ordering::Release);
    }
}

///
/// Basic Tests
#[cfg(test)]
mod test_aggregate_sparse {
    use super::*;
    use debugging::session::{DebugSession, LogLevel};

    #[test]
    fn test_build_sql() {
        DebugSession::new().filter(LogLevel::Debug).init().unwrap();
        let dbg = "test_str_write";
        let mut aggregated = Vec::with_capacity(10);
        let mut sql = String::with_capacity(4096);
        let test_data = [
            (01, vec![], ""),
            (02, vec![(1212, 1212.1313)],
            "INSERT INTO my_table (id, deprecation) VALUES (1212, 1212.1313) ON CONFLICT (id) DO UPDATE SET deprecation = my_table.deprecation + EXCLUDED.deprecation;"),
            (03, vec![(111, 111.111), (222, 222.222)],
            "INSERT INTO my_table (id, deprecation) VALUES (111, 111.111), (222, 222.222) ON CONFLICT (id) DO UPDATE SET deprecation = my_table.deprecation + EXCLUDED.deprecation;"),
            (04, vec![(1212, 1212.1313), (2121, 2121.2121), (3131, 3131.3131)],
            "INSERT INTO my_table (id, deprecation) VALUES (1212, 1212.1313), (2121, 2121.2121), (3131, 3131.3131) ON CONFLICT (id) DO UPDATE SET deprecation = my_table.deprecation + EXCLUDED.deprecation;"),
        ];
        for (step, values, target) in test_data {
            aggregated.clear();
            for (id, dep) in values {
                aggregated.push((id, dep));
            }
            if RopeDeprecation::build_sql(&mut aggregated, "my_table", &mut sql).is_ok() {
                log::debug!("{dbg} | Sql: {:?}", sql);
                assert!(sql == target, "{dbg} | Step {step} \n result: {:?} \n target: {:?}", sql, target);
            } else {
                assert!(sql.is_empty(), "{dbg} | Step {step} SQL must be empty");
            }
        }
    }

    ///
    /// Tests for `aggregate_sparse`
    #[test]
    fn test_aggregate_sparse_empty() {
        DebugSession::new().filter(LogLevel::Debug).init().unwrap();
        let dbg = "test_aggregate_sparse_empty";
        let mut src: Vec<(usize, f64)> = vec![];
        let mut result: Vec<(usize, f64)> = vec![(999, 1.0)]; // непустой result — должен очиститься
        RopeDeprecation::aggregate_sparse(&mut src, &mut result);
        assert!(result.is_empty(), "{dbg} | result must be empty for empty src");
    }

    #[test]
    fn test_aggregate_sparse_single() {
        DebugSession::new().filter(LogLevel::Debug).init().unwrap();
        let dbg = "test_aggregate_sparse_single";
        let mut src = vec![(42, 3.5)];
        let mut result = Vec::new();
        RopeDeprecation::aggregate_sparse(&mut src, &mut result);
        assert_eq!(result, vec![(42, 3.5)], "{dbg} | single element must pass through");
    }

    #[test]
    fn test_aggregate_sparse_all_unique() {
        DebugSession::new().filter(LogLevel::Debug).init().unwrap();
        let dbg = "test_aggregate_sparse_all_unique";
        let mut src = vec![(1, 10.0), (2, 20.0), (3, 30.0)];
        let mut result = Vec::new();
        RopeDeprecation::aggregate_sparse(&mut src, &mut result);
        assert_eq!(result, vec![(1, 10.0), (2, 20.0), (3, 30.0)], "{dbg} | unique indices must not be aggregated");
    }

    #[test]
    fn test_aggregate_sparse_all_same_index() {
        DebugSession::new().filter(LogLevel::Debug).init().unwrap();
        let dbg = "test_aggregate_sparse_all_same_index";
        let mut src = vec![(5, 1.0), (5, 2.0), (5, 3.0)];
        let mut result = Vec::new();
        RopeDeprecation::aggregate_sparse(&mut src, &mut result);
        assert_eq!(result, vec![(5, 6.0)], "{dbg} | same indices must be summed");
    }

    #[test]
    fn test_aggregate_sparse_unsorted_duplicates() {
        DebugSession::new().filter(LogLevel::Debug).init().unwrap();
        let dbg = "test_aggregate_sparse_unsorted_duplicates";
        // Дубликаты разбросаны, индексы не отсортированы
        let mut src = vec![(3, 30.0), (1, 10.0), (3, 5.0), (1, 2.0), (2, 20.0)];
        let mut result = Vec::new();
        RopeDeprecation::aggregate_sparse(&mut src, &mut result);
        assert_eq!(result, vec![(1, 12.0), (2, 20.0), (3, 35.0)], "{dbg} | unsorted duplicates must be sorted and summed");
    }

    #[test]
    fn test_aggregate_sparse_reverse_order() {
        DebugSession::new().filter(LogLevel::Debug).init().unwrap();
        let dbg = "test_aggregate_sparse_reverse_order";
        let mut src = vec![(10, 1.0), (8, 2.0), (8, 3.0), (5, 4.0), (5, 5.0), (5, 6.0)];
        let mut result = Vec::new();
        RopeDeprecation::aggregate_sparse(&mut src, &mut result);
        assert_eq!(result, vec![(5, 15.0), (8, 5.0), (10, 1.0)], "{dbg} | reverse-order input must be sorted and aggregated");
    }

    #[test]
    fn test_aggregate_sparse_negative_values() {
        DebugSession::new().filter(LogLevel::Debug).init().unwrap();
        let dbg = "test_aggregate_sparse_negative_values";
        let mut src = vec![(1, -10.0), (1, 5.0), (2, -3.0), (2, -7.0)];
        let mut result = Vec::new();
        RopeDeprecation::aggregate_sparse(&mut src, &mut result);
        assert_eq!(result, vec![(1, -5.0), (2, -10.0)], "{dbg} | negative values must be summed correctly");
    }

    #[test]
    fn test_aggregate_sparse_zero_values() {
        DebugSession::new().filter(LogLevel::Debug).init().unwrap();
        let dbg = "test_aggregate_sparse_zero_values";
        let mut src = vec![(0, 0.0), (0, 0.0), (1, 0.0)];
        let mut result = Vec::new();
        RopeDeprecation::aggregate_sparse(&mut src, &mut result);
        assert_eq!(result, vec![(0, 0.0), (1, 0.0)], "{dbg} | zero values must be handled");
    }

    #[test]
    fn test_aggregate_sparse_result_cleared() {
        DebugSession::new().filter(LogLevel::Debug).init().unwrap();
        let dbg = "test_aggregate_sparse_result_cleared";
        // result содержит мусор — метод должен его очистить перед записью
        let mut src = vec![(1, 1.0)];
        let mut result = vec![(100, 100.0), (200, 200.0), (300, 300.0)];
        RopeDeprecation::aggregate_sparse(&mut src, &mut result);
        assert_eq!(result, vec![(1, 1.0)], "{dbg} | result must be cleared before writing");
    }

    #[test]
    fn test_aggregate_sparse_large_input() {
        DebugSession::new().filter(LogLevel::Debug).init().unwrap();
        let dbg = "test_aggregate_sparse_large_input";
        // 1000 элементов, каждый 3-й индекс дублируется 3 раза
        let mut src = Vec::with_capacity(3000);
        for i in 0..1000usize {
            src.push((i, i as f64));
            src.push((i, i as f64));
            src.push((i, i as f64));
        }
        let mut result = Vec::new();
        let t = std::time::Instant::now();
        RopeDeprecation::aggregate_sparse(&mut src, &mut result);
        log::debug!("{dbg} | 1000 done in {:?}", t.elapsed());
        assert_eq!(result.len(), 1000, "{dbg} | result must have 1000 unique indices");
        for (i, &(ix, val)) in result.iter().enumerate() {
            assert_eq!(ix, i, "{dbg} | index mismatch at position {i}");
            assert_eq!(val, (i as f64) * 3.0, "{dbg} | sum mismatch at index {i}");
        }
    }

    #[test]
    fn test_aggregate_sparse_groups_of_duplicates() {
        DebugSession::new().filter(LogLevel::Debug).init().unwrap();
        let dbg = "test_aggregate_sparse_groups_of_duplicates";
        // Несколько групп дубликатов, перемешанных
        let mut src = vec![
            (7, 1.0), (3, 1.0), (7, 2.0), (3, 2.0), (7, 3.0),
            (9, 100.0), (3, 3.0), (9, 200.0),
        ];
        let mut result = Vec::new();
        RopeDeprecation::aggregate_sparse(&mut src, &mut result);
        assert_eq!(result, vec![(3, 6.0), (7, 6.0), (9, 300.0)], "{dbg} | mixed groups must be sorted and summed");
    }
}
