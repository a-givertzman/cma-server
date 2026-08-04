use std::{sync::Arc, time::Duration};
use dashmap::DashMap;
use function_name::named;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{
    kernel::state::ExitNotify, services::{EventValueAccess, Service, Services, entity::{Name, Object}}, thread_pool::Scheduler
};
use crate::{domain::{RECV_TIMEOUT, RecvTimeoutError}, err, err_pass, infra::ApiClient};
use super::VibroMonitorConf;

///
/// ### VibroMonitor Service | Сервис вибродиагностики
pub struct VibroMonitor {
    name: Name,
    conf: VibroMonitorConf,
    services: Arc<Services>,
    scheduler: Scheduler,
    tasks: Arc<DashMap<String, Arc<dyn Service>>>,
    exit: Arc<ExitNotify>,
    dbg: Dbg,
}
//
impl VibroMonitor {
    ///
    /// Crteates [VibroMonitor] new instance
    pub fn new(conf: VibroMonitorConf, services: Arc<Services>, scheduler: Scheduler) -> Self {
        let dbg = Dbg::new(conf.name.parent(), conf.name.me());
        Self {
            name: conf.name.clone(),
            conf,
            services,
            scheduler,
            tasks: Arc::new(DashMap::new()),
            exit: Arc::new(ExitNotify::new(&dbg, None, None)),
            dbg,
        }
    }
    ///
    /// Configuration of the database
    #[named]
    fn configure_database(&self, api_client: &Arc<ApiClient>, exit: &Arc<ExitNotify>) -> Result<(), Error> {
        let dbg = self.dbg.clone();
        let fetch_timeout = Duration::from_secs(10);
        let vibration_trends = &self.conf.tables.trends;
        let vibration_faults = &self.conf.tables.faults;
        let sql = format!(r#"
do $$
begin
-- 1. Справочник оборудования
CREATE TABLE IF NOT EXISTS equipment (
    id          integer GENERATED ALWAYS AS IDENTITY,
    name        VARCHAR(255) NOT NULL,       -- Наименование (например, 'Насос НП-101')
    model       VARCHAR(100),                -- Модель/Тип агрегата
    created_at  TIMESTAMPTZ DEFAULT clock_timestamp() NOT NULL,
    CONSTRAINT pk_equipment PRIMARY KEY (id),
    CONSTRAINT uq_equipment_name UNIQUE (name)
);

-- 2. Таблица учета наработки и ресурса (Wear & Lifespan)
CREATE TABLE equipment_wear (
    equipment_id       INTEGER PRIMARY KEY REFERENCES equipment(id) ON DELETE CASCADE,
    operating_hours    REAL NOT NULL DEFAULT 0.0,  -- Фактическая наработка (моточасы)
    nominal_resource   REAL NOT NULL,              -- Номинальный ресурс до кап. ремонта (моточасы)
    updated_at         TIMESTAMPTZ NOT NULL        -- Время последнего обновления наработки
);

-- 3. Обновленная таблица трендов вибрации
CREATE TABLE {vibration_trends} (
    timestamp      TIMESTAMPTZ NOT NULL,
    equipment_id   INTEGER NOT NULL REFERENCES equipment(id) ON DELETE CASCADE,
    order_id       VARCHAR(10) NOT NULL,       -- '1x', '2x', '3x', '0.5x' и т.д.
    rms_value      REAL NOT NULL,              -- Амплитуда (RMS)
    phase          REAL,                       -- Фаза в градусах [0..360)
    rpm            REAL NOT NULL,              -- Текущие обороты вала
    
    PRIMARY KEY (timestamp, equipment_id, order_id)
);

-- Индекс для быстрой фильтрации по конкретному агрегату и гармонике
CREATE INDEX idx_equip_order_trends ON {vibration_trends} (equipment_id, order_id, timestamp DESC);

-- Справочник видов дефектов
CREATE TABLE fault_kind (
    id          VARCHAR(64) NOT NULL,
    description TEXT NOT NULL,

    PRIMARY KEY (id)
)
INSERT INTO fault_kind (id, description) VALUES
    ('Imbalance', 'Дисбаланс'),
    ('Misalignment', 'Расцентровка'),
    ('MechanicalLooseness', 'Механический люфт');

-- Степень развития дефекта (Зоны ISO 10816 / 20816)
CREATE TYPE vibration_severity AS ENUM (
    'green',   -- Отличное или новое состояние.
    'yellow',  -- Пригодно для длительной эксплуатации без ограничений.
    'orange',  -- Предупреждение (Warn). Пригодно для ограниченной эксплуатации, требуется планирование ремонта.
    'red'      -- Преждевременный отказ (Alarm). Опасные вибрации, требуется немедленная остановка.
);
COMMENT ON TYPE vibration_severity VALUE 'green' IS 'Отличное или новое состояние.';
COMMENT ON TYPE vibration_severity VALUE 'yellow' IS 'Пригодно для длительной эксплуатации без ограничений.';
COMMENT ON TYPE vibration_severity VALUE 'orange' IS 'Предупреждение (Warn). Пригодно для ограниченной эксплуатации, требуется планирование ремонта.';
COMMENT ON TYPE vibration_severity VALUE 'red' IS 'Преждевременный отказ (Alarm). Опасные вибрации, требуется немедленная остановка.';

-- Оценка состояния оборудования на основании вибрации
CREATE TABLE {vibration_faults} (
    timestamp      TIMESTAMPTZ NOT NULL,
    equipment_id   INTEGER NOT NULL REFERENCES equipment(id) ON DELETE CASCADE,
    -- Вид неисправности
    fault_kind     VARCHAR(64) NOT NULL REFERENCES fault_kind(id),
    -- Метрика сходства с патерном дефекта [0.0, 1.0]
    score          DOUBLE PRECISION NOT NULL,
    -- Степень опасности текущего дефекта
    -- Green, Yellow, Orange, Red
    severity       vibration_severity NOT NULL,
    -- Текущие обороты расчете, для валидации диагноза
    rpm            DOUBLE PRECISION NOT NULL 
    -- Гарантирует, что для каждого оборудования 
    -- хранится ровно ОДНА запись по конкретному дефекту
    PRIMARY KEY (equipment_id, fault_kind)
);


-- SQL-запрос для вывода списка оборудования с худшим статусом
SELECT 
    e.id AS equipment_id,
    e.name AS equipment_name,
    -- MAX() для ENUM в Postgres выберет самое критическое состояние (последнее в списке ENUM)
    MAX(vf.severity) AS overall_severity,
    -- Собираем список всех обнаруженных дефектов, которые вышли из зоны 'green'
    STRING_AGG(
        CASE WHEN vf.severity != 'green' THEN fk.description END, 
        ', '
    ) AS active_faults,
    MAX(vf.timestamp) AS last_update
FROM equipment e
LEFT JOIN vibration_faults vf ON e.id = vf.equipment_id
LEFT JOIN fault_kind fk ON vf.fault_kind = fk.id
GROUP BY e.id, e.name
ORDER BY 
    -- Сначала показываем самое "красное" и "оранжевое" оборудование
    overall_severity DESC NULLS LAST, 
    e.name;
end; $$
language plpgsql;
        "#);
        match api_client.fetch(sql).timeout(fetch_timeout) {
            Ok(Some(Ok(_))) => Ok(()),
            Ok(Some(Err(err))) => Err(err_pass!(self.dbg, err)),
            Ok(None) => Err(err!(self.dbg, "Timeout {:?}", fetch_timeout)),
            Err(err) => Err(err_pass!(self.dbg, err)),
        }
    }
}
//
impl Object for VibroMonitor {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
impl std::fmt::Debug for VibroMonitor {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("VibroMonitor")
            .field("dbg", &self.dbg)
            .finish()
    }
}
//
impl Service for VibroMonitor {
    //
    #[named]
    fn run(&self) -> Result<(), Error> {
        log::info!("{}.run | Starting...", self.dbg);
        let name = self.name.clone();
        let conf = self.conf.clone();
        let services = self.services.clone();
        let scheduler = self.scheduler.clone();
        let fetch_timeout = Duration::from_secs(3);
        let api_client = Arc::new(ApiClient::new(conf.api.clone(), scheduler.clone()));
        self.tasks.insert(api_client.name().join(), api_client.clone());
        api_client.run().map_err(|err| err_pass!(self.dbg, err))?;
        log::info!("{}.run | ApiClient ready", self.dbg);
        // Конфигурация БД
        // self.configure_database(&api_client, &self.exit).map_err(|err| err_pass!(self.dbg, err))?;
        let retain = Arc::new(vibro_core::Retain::mock(&self.dbg, []));
        let event_values = Arc::new(super::EventValues::new(&name, &conf.subscribe, services.clone(), scheduler.clone(), self.exit.clone()));
        event_values.subscribe(conf.);
        self.tasks.insert(event_values.name().join(), event_values.clone());
        let (api_link, api_queue) = crate::domain::bounded(4096);
        // TODO: Переместить в микросервис
        scheduler.spawn({
            let dbg = self.dbg.clone();
            let exit = self.exit.clone();
            move || {
            while !exit.get() {
                match api_queue.recv_timeout(RECV_TIMEOUT) {
                    Ok(sql) => {
                        match api_client.fetch(&sql).timeout(fetch_timeout) {
                            Ok(Some(Err(err))) => log::warn!("{dbg}.run | Error on sql '{sql}': \n{:?}", err),
                            Ok(None) => log::warn!("{dbg}.run | Timeout ({:?}) on sql '{sql}'", fetch_timeout),
                            Err(err) => log::warn!("{dbg}.run | Error on sql '{sql}': \n{:?}", err),
                            _ => {}
                        }
                    }
                    Err(RecvTimeoutError::Timeout) => {}
                    _ => break,
                }
            }
        }}).map_err(|err| err_pass!(self.dbg, err))?;
        // Настройка диагностики для датчиков, датчики сгруппированы по IP адресам
        // Передаем группу датчиков с одним IP в один модуль
        for (_adc_id, sensors_conf) in &conf.sensors {
            let sensor = Arc::new(super::VibroAdc::new(
                &self.dbg,
                sensors_conf.clone(),
                event_values.clone(),
                retain.clone(),
                api_link.clone(),
                scheduler.clone(),
                self.exit.clone(),
            ));
            self.tasks.insert(sensor.name().join(), sensor.clone());
            sensor.run().map_err(|err| err_pass!(self.dbg, err))?;
        }
        event_values.run().map_err(|err| err_pass!(self.dbg, err))?;      // have to be started after all subscription being added, then it will subscribe all them on MultiQueue
        log::info!("{}.run | RopeDefect's ready", self.dbg);
        log::info!("{}.run | Starting - Ok", self.dbg);
        Ok(())
    }
    //
    fn wait(&self) -> Result<(), Error> {
        let mut errors = vec![];
        for task in self.tasks.iter() {
            if let Err(err) = task.value().wait() {
                errors.push(err);
            }
        }
        errors
            .is_empty()
            .then(|| {
                log::info!("{}.run | Exit", self.dbg);
                ()
            })
            .ok_or(
                Error::new(&self.dbg, "wait").pass(errors.iter().fold(String::new(), |acc, err| format!("{}\n{}", acc, err)))
            )
    }
    //
    fn is_finished(&self) -> bool {
        let mut is_finished = false;
        for task in self.tasks.iter() {
            is_finished = is_finished & task.value().is_finished();
        }
        is_finished
    }
    //
    fn exit(&self) {
        self.exit.exit();
        for task in self.tasks.iter() {
            task.value().exit();
        }
    }    
}

