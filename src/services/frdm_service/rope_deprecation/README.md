# RopeDeprecation — расчёт скорости износа каната

Сервис считает накопленный износ (deprecation rate) стального/стекловолоконного каната
по мере прохождения через блоки грузоподъёмной машины. Износ накапливается в
таблице БД и доступен клиентам через API-сервер.

---  


## Назначение

На каждый приходящий event (позиция каната, нагрузка, углы стрел) сервис:
1. Пересчитывает геометрию прохождения каната через блоки (bendings).
2. Определяет, какие сегменты каната в данный момент лежат на блоках.
3. Для каждого вошедшего/вышедшего сегмента начисляет прирост износа
   пропорционально нагрузке и обратно пропорционально диаметру блока.
4. УПРОЩЕНИЕ Диаметр самого каната в расчете пока не учавствует
5. Агрегирует пары `(slice_ix, deprecation)`, формирует `INSERT ... ON CONFLICT`
   и отправляет SQL в БД через `ApiClient` (fire-and-forget).

---  


## Основные файлы

| Файл | Назначение |
|------|------------|
| [`rope_deprecation.rs`](rope_deprecation.rs) | Сам сервис: запуск, цикл обработки, агрегация, SQL. |
| [`rope_deprecation_conf.rs`](rope_deprecation_conf.rs) | Конфиг `RopeDeprecationConf` (`wait-started`, `table`, `crane`). |
| [`crane_conf.rs`](crane_conf.rs) | `CraneConf` — сборка `rope` + `booms` + `blocks`. |
| [`rope_conf.rs`](rope_conf.rs) | `RopeConf` — параметры каната (диаметр, длина, сегмент, точки pos/load). |
| [`boom_conf.rs`](boom_conf.rs) | `BoomConf` — геометрия одной стрелы (l1–l4, len, angle, parking). |
| [`block_conf.rs`](block_conf.rs) | `BlockConf` — геометрия блока (lf, d, scheme, bind, deflector). |
| [`algorithm/deprecation.rs`](algorithm/deprecation.rs) | `Deprecation` — ядро расчёта: сопоставление вход/выход сегментов. |
| [`algorithm/bendings.rs`](algorithm/bendings.rs) | `Bendings` — опорные точки каната на блоках. |
| [`algorithm/booms.rs`](algorithm/booms.rs) | `Booms` — углы и точки D/G стрел. |
| [`algorithm/blocks.rs`](algorithm/blocks.rs) | `Blocks` — позиции блоков в СК стрел. |
| [`../frdm_service.rs`](../frdm_service.rs) | `FrdmService` — родитель: создаёт и запускает `RopeDeprecation`. |
| [`../inputs.rs`](../inputs.rs) | `Inputs` — подписка на шину MultiQueue, кэш последних значений. |

---  


## Жизненный цикл

### Запуск (`run`)

1. Синхронно, в потоке вызывающего, вызывается `build_math` — собирает
   цепочку алгоритмов `Deprecation → Bendings → BlockArcs → RopeSections
   → Blocks → Booms`. На этом этапе выполняются `inputs.subscribe(...)` для
   точек `pos`, `load` и углов/длин стрел. **Подписки регистрируются до
   возврата из `run`**, что исключает гонку со стартом `Inputs`.
2. `scheduler.spawn` запускает рабочий поток.
3. Рабочий поток подписывается на `inputs.listen()`, затем входит в цикл
   `recv_timeout(RECV_TIMEOUT)` (100 ms).
4. На каждый event: `deprecation.eval()` → `drain_into` результатов →
   `aggregate_sparse` (сортировка + суммирование дубликатов) → `build_sql`
   → `api_client.fetch(sql).then(...)` (без ожидания ответа).
5. Если задан `wait-started`, `run` блокируется до подтверждения старта
   рабочего потока (`service_release.add`).

### Останов (`exit` / `wait`)

- `exit()` выставляет `exit: AtomicBool → true`. Рабочий поток выходит из
  цикла на следующей итерации (до 100 ms), логирует `Exit`.
- `wait()` ждёт завершения handle через `Handles::wait`.
- `is_finished()` проверяет состояние handle.

> **Важно:** `exit()` не закрывает канал `inputs.listen()` принудительно.
> Поток завершается по флагу `exit` на следующем `recv_timeout`. Если шина
> MultiQueue закрылась — сработает ветка `Err(_) → break`.

---  


## Подписки и поток данных

```
MultiQueue ──subscribe──▶ Inputs (кэш FxDashMap)
                              │
                              ├─ listen() ──▶ RopeDeprecation (recv_timeout)
                              │                  │
                              │                  ├─ inputs.get(pos)  → Bendings.eval
                              │                  ├─ inputs.get(load) → Deprecation.eval
                              │                  └─ pairs_tx.send((slice, dep))
                              │                         │
                              │                  pairs_rx.drain_into
                              │                         │
                              │                  aggregate_sparse + build_sql
                              │                         │
                              └──────────────▶ ApiClient.fetch(sql) ──▶ БД
```

`Inputs` подписывается на MultiQueue один раз при своём старте (после
`RopeDeprecation::run`), делая снимок всех накопленных `subscribe` ключей.
Поэтому порядок запуска в `FrdmService::run` строго:
`rope_deprecation.run()` → `rope_defect.run()` → `inputs.run()`.

---  


## База данных

Накопленный износ каната храниться в таблице БД. Для каждого сегмента в отдельной записи.

Название таблицы `public.frdm_deprecation`

Структура таблицы

 Номер Сегмента  |  Значение накопленного износа
 ---             |  ---
 id              |  deprecation
 bigserial       |  numeric(24, 8)

---  


## Конфигурация

Пример конфигурации можно поискать в файле `frdm-config.yaml`

### Полный пример

Основан на `ysz-deprecation_test.yaml`. Это рабочий минимальный конфиг
секции `rope-deprecation` внутри `service FrdmService:`.

```yaml
rope-deprecation:
    wait-started: 10 ms
    table: 'public.frdm_deprecation'
    crane:
        rope:
            width: 35 mm               # диаметр каната
            length: 85.045 m           # рабочая длина каната (от барабана до крюка)
            aux-length: 1.200 m        # холостой ход от последнего блока до крюка
            segment: 100 mm            # длина расчётного сегмента (точность износа)
            pos: point real 'Winch.Pos'     # позиция каната, м (от парковочного)
            load: point real 'Winch.Load'   # нагрузка на канат, т
        booms:
            - Main-Boom:
                l1: 0.0 mm
                l2: 0.0 mm
                l3: 0.0 mm
                l4: 10330.0 mm
                len: 11200.0 mm
                angle: point real 'MainBoom.Angle'   # угол стрелы, град
                parking: 0.0 deg                     # угол в парковочном положении
            - Rotary-Boom:
                l1: 0.0 mm
                l2: 0.0 mm
                l3: 0.0 mm
                l4: 0.0 mm
                len: 7984.0 mm
                angle: point real 'RotaryBoom.Angle'
                parking: 23.78 deg
        blocks:
            - 1:
                lf: 1830.0 mm,  710.0 mm
                d: 845.000 mm
                scheme: TopTop
                bind: Drum
            - 2:
                lf: 308.0 mm, 1100.0 mm
                d: 816.000 mm
                scheme: TopTop
                bind: Boom 0
            - 3:
                lf: -6549.0 mm, 1730.0 mm
                d: 816.000 mm
                scheme: TopTop
                bind: Boom 1
            - 4:
                lf: -1121.0 mm, 973.0 mm
                d: 816.000 mm
                scheme: TopBottom
                bind: Boom 1
            - 5:
                lf: 267.0 mm, 860.0 mm
                d: 816.000 mm
                scheme: BottomTop
                bind: Boom 1
            - 6:
                lf: 136.0 mm, -35.0 mm
                d: 816.000 mm
                scheme: TopTop
                bind: Boom 1
                deflector-angle: 90 deg   # опционально
            - 7:
                lf: 0.0 mm, 0.0 mm
                d: 0.0 mm
                scheme: TopTop
                bind: Hook
```

### Параметры секции `rope-deprecation`

| Параметр | Тип | Обяз. | Описание |
|----------|-----|-------|----------|
| `wait-started` | Duration | нет | Задержка после старта рабочего потока перед возвратом из `run`. Гарантирует готовность подписок к моменту старта `Inputs`. Рекомендуется указывать. |
| `table` | String | **да** | Имя таблицы БД для накопления износа. Схема SQL: `id` (int), `deprecation` (numeric). |
| `crane` | Map | **да** | Геометрия крана: `rope`, `booms`, `blocks`. |

### Параметры `crane.rope`

| Параметр | Тип | Обяз. | Описание |
|----------|-----|-------|----------|
| `width` | Distance | **да** | Диаметр каната. Используется для валидации `segment`. А в будущем и для расчета износа. |
| `length` | Distance | **да** | Полная рабочая длина каната. Определяет число сегментов `slices = length / segment`. |
| `aux-length` | Distance | **да** | Длина каната от последнего блока (на конце последней стрелы) до крюка. |
| `segment` | Distance | **да** | Длина расчётного сегмента. Весь канат делится на сегменты; износ считается на segment. Ограничение: `0.5·width ≤ segment ≤ 7·width`. Меньше сегмент — выше точность, больше вычислений. |
| `pos` | point real | **да** | Имя точки позиции каната (м). Количество отданного с барабана каната. Значение из шины. |
| `load` | point real | **да** | Имя точки нагрузки (т). Текущая нагрузка на крюке. Значение из шины. |

### Параметры `crane.booms` (список)

Каждый элемент — Map с одним ключом-именем стрелы.

| Параметр | Тип | Обяз. | Описание |
|----------|-----|-------|----------|
| `l1` | Distance | **да** | Расстояние от продольной оси стрелы до оси поворота (точка A). |
| `l2` | Distance | **да** | Расстояние по продольной оси от корня (точка D) до оси поворота (точка A). |
| `l3` | Distance | **да** | Расстояние от оси поворота до продольной оси предыдущей стрелы (для первой — до ГСК). |
| `l4` | Distance | **да** | Расстояние от оси поворота до перпендикуляра через точку G предыдущей стрелы. |
| `len` | Distance \| point real | **да** | Длина стрелы. Может быть константой или точкой (телескоп). |
| `angle` | f64 \| point real | **да** | Угол стрелы (град). Константа или точка шины. |
| `parking` | Angle | **да** | Угол в парковочном положении. При первом вызове расчёт ведётся для парковочной геометрии. |

### Параметры `crane.blocks` (список)

Каждый элемент — Map с ключом-номером (строка или число).

| Параметр | Тип | Обяз. | Описание |
|----------|-----|-------|----------|
| `lf` | "X, Y" | **да** | Расстояние (x, y) от **конца** стрелы до оси блока, через запятую. Единицы — мм. |
| `d` | Distance | **да** | Диаметр блока. `d: 0` — блок-крюк (не участвует в расчёте износа). |
| `scheme` | enum | **да** | Схема схода каната: <br>&emsp;`TopTop (1)`, <br>&emsp;`TopBottom (2)`, <br>&emsp;`BottomTop (3)`, <br>&emsp;`BottomBottom (4)`. |
| `bind` | enum | **да** | Привязка: <br>&emsp;`Drum` (барабан), <br>&emsp;`Fixed` (неподвижный вне стрел), <br>&emsp;`Boom N` (блок на N-ной стреле), <br>&emsp;`Hook` (подвес). |
| `deflector-angle` | Angle | нет | Угол перекидывания. Блок включается в работу только когда стрела проходит это положение. 90 deg - означает вертикально вниз. |

---  


## Тесты

### Инлайн-тесты (`rope_deprecation.rs`)

Модуль `test_aggregate_sparse` — проверка `aggregate_sparse` и `build_sql`:
пустые входы, дубликаты, неотсортированные данные, отрицательные значения,
группы, 1000 элементов. Запуск:

```bash
cargo test --profile fast-test rope_deprecation::test_aggregate_sparse
```

### Тесты алгоритма

| Файл | Что проверяет |
|------|---------------|
| [`deprecation_test.rs`](../../../tests/unit/services/frdm_service/deprecation_test.rs) | `Deprecation::eval` на CSV-данных, сверка с эталоном. |
| [`bendings_test.rs`](../../../tests/unit/services/frdm_service/bendings_test.rs) | `Bendings` — опорные точки. |
| [`block_arcs_test.rs`](../../../tests/unit/services/frdm_service/block_arcs_test.rs) | `BlockArcs` — дуги обхвата. |
| [`booms_test.rs`](../../../tests/unit/services/frdm_service/booms_test.rs) | `Booms` — углы и точки D/G. |
| [`blocks_test.rs`](../../../tests/unit/services/frdm_service/blocks_test.rs) | `Blocks` — позиции блоков. |
| [`rope_sections_test.rs`](../../../tests/unit/services/frdm_service/rope_sections_test.rs) | `RopeSections` — секции каната. |

Эталонные конфиги и CSV для тестов алгоритма:
- [`spu-tnpa-deprecation_test.yaml`](../../../tests/unit/services/frdm_service/spu-tnpa-deprecation_test.yaml)
- [`ysz-deprecation_test.yaml`](../../../tests/unit/services/frdm_service/ysz-deprecation_test.yaml)

### Интеграционный тест сервиса

[`frdm_service_deprecation_test.rs`](../../../tests/unit/services/frdm_service/frdm_service_deprecation_test.rs) —
полный запуск `FrdmService` через `ServiceTestPlanner` с эмуляцией шины.
Помечен `#[ignore]` (требует API-сервер и БД). Конфиг теста приведён к
актуальному формату и парсится без ошибок.

## Эталонные конфиги

| Файл | Описание |
|------|----------|
| [`ysz-deprecation_test.yaml`](../../../tests/unit/services/frdm_service/ysz-deprecation_test.yaml) | Кран YSZ: 2 стрелы, 7 блоков, `deflector-angle` на блоке 6. |
| [`spu-tnpa-deprecation_test.yaml`](../../../tests/unit/services/frdm_service/spu-tnpa-deprecation_test.yaml) | Кран SPU-TNPA: 2 стрелы, 7 блоков. |
| [`deprecation-test.yaml`](../../../tests/unit/services/frdm_service/deprecation-test.yaml) | Интеграционный конфиг с полным `service FrdmService`. |

---  


## Замечания

1. **Имена точек** в `pos`/`load`/`angle` должны точно совпадать с именами
   событий на шине MultiQueue. Несовпадение приводит к молчаливому игнорированию
   (warn в логах `Inputs`).
   - В тестовых конфигах используются короткие имена (`'Winch.Pos'`).
   - В рабочих и интеграционных конфигах — с путём (`'/App/VirtualIed/Winch.Pos'`).

2. **`segment` и `width`** связаны валидацией: `0.5·width ≤ segment ≤ 7·width`.
   Выход за границы — panic при парсинге конфига.

3. **`wait-started`** у `rope-deprecation` не является обязательным полем
   конфига, **но очень рекомендуется**: он гарантирует, что рабочий поток дошёл до
   `service_release.add` до возврата из `run`. Без него гонка исключена
   конструктивно (подписки регистрируются синхронно в `build_math` до
   `spawn`), но `wait-started` даёт запас на инициализацию `Deprecation`.
