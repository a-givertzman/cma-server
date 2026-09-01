# RopeDefect — обнаружение дефектов каната по изображениям с камеры

Сервис анализирует кадры с IP-камер, установленных вдоль каната грузоподъёмной машины.
Канат условно нарезается на сегменты, и в моменты, когда камера проходит границу
между сегментами, кадр обрабатывается алгоритмом обнаружения дефектов. Результаты
(дефекты и изображения) сохраняются в БД и в локальное хранилище.

---


## Назначение

На каждый приходящий с камеры кадр сервис:

1. Проверяет, что позиция каната (`segment_ix`) инициализирована — до этого
   детекция не запускается.
2. Пропускает кадры, относящиеся к уже обработанному сегменту (защита от
   повторной обработки одного и того же сегмента).
3. Прогоняет кадр через цепочку нормализации и сканирования:
   `Initial → Cropping → AutoGamma → Gray → FastScan → FineScan`.
4. Для каждого обнаруженного дефекта (`expansion`, `compressing`, `hill`, `pit`)
   формирует SQL `INSERT ... ON CONFLICT` в таблицы дефектов и изображений
   и отправляет его через `ApiClient` (fire-and-forget).
5. Сохраняет кадр-вырезку в локальное хранилище
   `<storage>/rope-defects/<slice>/<имя-файла>.jpg` и запускает очистку
   устаревших изображений через функцию БД `clean_frdm_defect_image`.

### Виды определяемых дефектов

| Дефект | Описание |
|--------|----------|
| `expansion` | Расширение — симметричная деформация с увеличением среднего диаметра |
| `compressing` | Сужение — симметричная деформация с уменьшением среднего диаметра |
| `hill` | Холмик — выпуклость на одной стороне изображения каната |
| `pit` | Канавка — углубление на одной стороне изображения каната |

---


## Основные файлы



| Файл | Назначение |
|------|------------|
| **Исходный код** | |
| [`rope_defect.rs`](rope_defect.rs) | Сам сервис: запуск, цикл приёма кадров, детекция, SQL, сохранение изображений. |
| [`rope_defect_conf.rs`](rope_defect_conf.rs) | Конфиг `RopeDefectConf` (`wait-started`, `tables`, `segment`, `cameras`, ...). |
| [`rope.rs`](rope.rs) | `Rope` — расчёт позиции каната под камерой и индекса сегмента (`segment_index`). |
| [`tables_conf.rs`](tables_conf.rs) | `TablesConf` — имена таблиц БД (`defect`, `defect-image`). |
| [`mod.rs`](mod.rs) | Описание модуля и пример конфигурации в комментариях. |
| [`../frdm_service.rs`](../frdm_service.rs) | `FrdmService` — родитель: создаёт и запускает `RopeDefect` для каждой камеры. |
| [`../inputs.rs`](../inputs.rs) | `Inputs` — подписка на шину MultiQueue, кэш последних значений, `cam_segment_ix`. |
| **База данных** | |
| [`../sql.md`](../sql.md) | Схемы таблиц `frdm_defect`, `frdm_defect_image`, функция `clean_frdm_defect_image`. |

---


## Жизненный цикл

### Запуск (`run`)

1. Из конфига берётся секция камеры по `camera_id` (`RopeDefect-{camera_id}`).
2. `scheduler.spawn` запускает рабочий поток, в котором собирается цепочка
   алгоритмов `FineScan → FastScan → Gray → AutoGamma → Cropping → Initial`
   с параметрами из секции `defect-detection`.
3. Если у камеры задан `from-path` — режим тестирования: кадры читаются из
   файлов директории, `service_release.add` вызывается после успешного старта.
4. В нормальном режиме цикл повторяет попытки подключения к камере
   (`Camera::stream` / `Camera::read`), при обрыве — переподключается
   с экспоненциально растущей паузой (от 64 ms до 3 s).
5. На каждый кадр: если `inputs.cam_segment_ix()` инициализирована —
   `detection()` → `FineScan.eval(frame)`. Колбэк детекции отправляет SQL в БД
   и сохраняет изображение (без ожидания ответа).
6. Если задан `wait-started`, `run` блокируется до подтверждения старта
   рабочего потока (`service_waiting.wait`).

### Останов (`exit` / `wait`)

- `exit()` выставляет `exit: AtomicBool → true`. Рабочий поток выходит из
  внутреннего и внешнего циклов, логирует `Exit`.
- `wait()` ждёт завершения handle через `Handles::wait`.
- `is_finished()` проверяет состояние handle.

> **Важно:** при потере камеры (`camera lost`) внутренний цикл прерывается,
> но внешний продолжает попытки переподключения, пока не выставлен `exit`.

---


## Подписки и поток данных

```
Camera ──stream──▶ RopeDefect (recv_timeout)
                       │
                       ├─ inputs.cam_segment_ix()  → контроль инициализации позиции
                       ├─ Rope.segment_index       → граница сегмента (в Inputs)
                       ├─ FineScan.eval(frame)     → детекция дефектов
                       │
                       ├─ ApiClient.fetch(sql) ──▶ БД (frdm_defect, frdm_defect_image)
                       └─ frame.save(...)      ──▶ локальное хранилище
```

Позиция сегмента под камерой считается в `Inputs` через `Rope` (`rope.rs`):
к позиции каната из шины (`pos`) прибавляется `camera-offset`, и если полученная
позиция совпадает с границей сегмента с точностью `segment-threshold` —
фиксируется индекс сегмента (`cam_segment_ix`). Именно поэтому порядок запуска
в `FrdmService::run` строго: `rope_deprecation.run()` → `rope_defect.run()`
→ `inputs.run()`.

---


## База данных

Дефекты хранятся в таблице БД. Полное описание схем, функций и примеров —
в [`../sql.md`](../sql.md).

### `frdm_defect`

| Поле | Тип | Описание |
|------|-----|----------|
| `slice` | bigint | Номер сегмента каната |
| `defect` | frdm_defect_kind | Вид дефекта: `expansion`, `compressing`, `hill`, `pit` |
| `camera` | int2 | Номер камеры |
| `first` | timestamp | Момент первой регистрации дефекта |
| `last` | timestamp | Момент последней регистрации дефекта |
| `score` | int8 | Счётчик повторных регистраций (UI показывает дефект при `score >= 3`) |
| `acknowledged` | timestamp | Момент сброса (score = 0) дефекта пользователем |

Первичный ключ: `(slice, defect, camera)`.

### `frdm_defect_image`

| Поле | Тип | Описание |
|------|-----|----------|
| `image_id` | bigserial | Локальный уникальный идентификатор изображения |
| `slice`, `defect`, `camera` | — | Составной внешний ключ на `frdm_defect` |
| `path` | text | Путь к файлу изображения |
| `created` | timestamp | Дата создания |

### Очистка устаревших изображений

При каждой регистрации дефекта вызывается
`select * from clean_frdm_defect_image(slice, defect, camera, 10);` —
функция оставляет 10 последних изображений на каждый `(slice, defect, camera)`
и возвращает пути удалённых файлов, которые сервис удаляет с диска.

---


## Конфигурация

Пример конфигурации можно поискать в файле `frdm-config.yaml`.

### Полный пример

Это рабочая секция `rope-defect` внутри `service FrdmService:` из `frdm-config.yaml`.

```yaml
rope-defect:
    tables:
        defect: 'public.frdm_defect'
        defect-image: 'public.frdm_defect_image'
    segment: 100 mm             # Whole rope will divided by the segments for the Camera defect detection, recomended: `segment length = camera.width * 0.10..0.20`
    segment-threshold: 5 mm     # Acceptable camera position error in relation to exact segment position
    camera-offset: 5.5 m        # camera position from the begin of the rope (hook side)
    defect-detection:
        normalize:
            cropping:
                x: 230              # New left edge
                y: 300              # New top edge
                width: 1410         # New image width
                height: 1000        # New image height
            gamma:
                factor: 120.0       # Percent of influence of [AutoGamma] algorythm bigger the value more the effect of [AutoGamma] algorythm, %

        fast-scan:
            fast-contours:
                otsu-tune: 0.40
            temporal-filter:
                gaussian:
                    kernel: [11, 11]    # Gausian blur kernel size, must be odd
                    sigma: [0.0, 0.0]   # Standard deviation in [X, Y] direction, The higher the value, the more pixels are used to count each pixel and the smoother blur will be
                open-kernel: [3, 3]     # Morphology open operation kernel size [w, h], default [5, 5]
                erode-kernel: [3, 3]    # Morphology erode operation kernel size [w, h], default [5, 5]
                threshold: 12.0         # Threshold to detect the pixel whas changed or not in the each next frame
            fast-edges:
                otsu-tune: 1.40         # Multiplier to otsu auto threshold, 1.0 - do nothing, just use otsu auto threshold, default 1.0
                # threshold: 128        # 0...255, used if otsu-tune is not specified
                smooth: 36              # Smoothing of edge line factor. The higher the factor the smoother the line.
            union:
                add-weighted:
                    weight1: 1.0            # Weight of the first array elements.
                    weight2: 1.0            # Weight of the second array elements.
            rope-dimensions:        # Verifaing the rope dimensions
                rope-width: 380               # Standart rope width, px
                width-tolerance: 50.0         # Tolerance for rope width, %
                square-tolerance: 100.0       # Tolerance for rope square, %
            distortion-threshold: 1.2    # 1.1..1.3, absolute threshold to detect the geometry deffects

        fine-scan:
            fine-contours:
                otsu-tune: 0.40         # Auto threshold factor, 1 - no correction, 0..1 - more, 1.. - less sensitive
                merge-distance: 24.0    # Maximum distance between contours to be merged
            temporal-filter:
                gaussian:
                    kernel: [11, 11]    # Gausian blur kernel size, must be odd
                    sigma: [0.0, 0.0]   # Standard deviation in [X, Y] direction, The higher the value, the more pixels are used to count each pixel and the smoother blur will be
                open-kernel: [3, 3]     # Morphology open operation kernel size [w, h], default [5, 5]
                erode-kernel: [3, 3]    # Morphology erode operation kernel size [w, h], default [5, 5]
                threshold: 12.0         # Threshold to detect the pixel whas changed or not in the each next frame
            fine-edges:
                # otsu-tune: 1.40       # Multiplier to otsu auto threshold, 1.0 - do nothing, just use otsu auto threshold, default 1.0
                threshold: 16           # 0...255, used if otsu-tune is not specified
                smooth: 16              # Smoothing of edge line factor. The higher the factor the smoother the line.
            union:
                # add-weighted:
                #     weight1: 1.0            # Weight of the first array elements.
                #     weight2: 1.0            # Weight of the second array elements.
                bitwise-and:
                    no-params: ~
            rope-dimensions:        # Verifaing the rope dimensions
                rope-width: 380               # Standart rope width, px
                width-tolerance: 30.0         # Tolerance for rope width, %
                square-tolerance: 100.0       # Tolerance for rope square, %
            distortion-threshold: 1.4    # 1.1..1.3, absolute threshold to detect the geometry deffects
            defect-threshold: 2.5        # 1.1..1.3, absolute threshold to detect the geometry deffects
    camera Camera1:
        # from-path: src/tests/unit/services/frdm_service/frames
        fps: Max                    # Max / Min / 30.0
        resolution:
            width: 1200
            height: 800
        index: 0
        # address: 192.168.10.12:2020
        pixel-format:  QOI_BayerRG8
        exposure:
            auto: Off                   # Off / Continuous
            time: 26000                 # microseconds
        auto-packet-size: true          # StreamAutoNegotiatePacketSize
        channel-packet-size: Max        # Maximizing packet size increases frame rate
        resend-packet: true             # StreamPacketResendEnable
```

### Параметры секции `rope-defect`

| Параметр | Тип | Обяз. | Описание |
|----------|-----|-------|----------|
| `wait-started` | Duration | нет | Задержка после старта рабочего потока перед возвратом из `run`. Гарантирует готовность сервиса к моменту старта следующих сервисов. Рекомендуется указывать. |
| `tables` | Map | **да** | Имена таблиц БД: `defect`, `defect-image`. См. `TablesConf`. |
| `segment` | Distance | **да** | Длина расчётного сегмента каната. Весь канат делится на сегменты; кадр обрабатывается при прохождении камерой границы сегмента. Рекомендация: `segment = (0.10..0.20) * ширина кадра камеры` (вдоль каната). Число сегментов (`slices`) рассчитывается от полной длины каната из секции `rope-deprecation` (`RopeDefectConf::slices`) и пишется в `frdm_settings` как `winchN-defect_slices`. |
| `segment-threshold` | Distance | **да** | Допустимая погрешность позиции камеры относительно точной границы сегмента. По умолчанию рекомендуется ~5% от `segment`. Если отклонение больше — кадр пропускается (`Rope::segment_index` возвращает `None`). |
| `camera-offset` | Distance | **да** | Позиция камеры от начала каната (со стороны крюка). Используется при расчёте индекса сегмента под камерой. |
| `defect-detection` | Map | **да** | Параметры нормализации и алгоритмов обнаружения дефектов (см. ниже). Реализовано в крейте `frdm-tools` (`frdm_tools::conf::Conf`). |
| `camera <Имя>:` | Map | нет | Секции камер. Ключ — ключевое слово `camera` с необязательным именем. Для каждой камеры создаётся отдельный экземпляр `RopeDefect-{id}`. Если камер нет — сервис не запускается (warn в логах `FrdmService`). |

### Параметры `defect-detection.normalize`

| Параметр | Тип | Обяз. | Описание |
|----------|-----|-------|----------|
| `cropping.x` | int | **да** | Новая левая граница кадра (обрезка). |
| `cropping.y` | int | **да** | Новая верхняя граница кадра. |
| `cropping.width` | int | **да** | Ширина обрезанного кадра. |
| `cropping.height` | int | **да** | Высота обрезанного кадра. |
| `gamma.factor` | f64 | **да** | Процент влияния авто-гаммы (`AutoGamma`). Больше значение — сильнее эффект. |

### Параметры `defect-detection.fast-scan`

Быстрое сканирование на каждом кадре: поиск контуров, временная фильтрация,
поиск границ и проверка геометрии каната.

| Параметр | Тип | Обяз. | Описание |
|----------|-----|-------|----------|
| `fast-contours.otsu-tune` | f64 | **да** | Коэффициент коррекции авто-порога Otsu. `1` — без коррекции; `0..1` — чувствительнее; `>1` — менее чувствительно. |
| `temporal-filter.gaussian.kernel` | [w, h] | **да** | Размер ядра гауссова размытия (нечётный). |
| `temporal-filter.gaussian.sigma` | [x, y] | **да** | СКО по X и Y. Больше значение — сильнее размытие. `0.0` — авто. |
| `temporal-filter.open-kernel` | [w, h] | **да** | Размер ядра морфологической операции open, по умолчанию `[5, 5]`. |
| `temporal-filter.erode-kernel` | [w, h] | **да** | Размер ядра эрозии, по умолчанию `[5, 5]`. |
| `temporal-filter.threshold` | f64 | **да** | Порог определения изменённого пикселя между соседними кадрами. |
| `fast-edges.otsu-tune` | f64 | **да** | Множитель авто-порога Otsu для поиска границ. `1.0` — без изменений. |
| `fast-edges.threshold` | int | нет | Порог `0..255`, используется вместо `otsu-tune`, если задан. |
| `fast-edges.smooth` | f64 | **да** | Коэффициент сглаживания линии границы. Больше — глаже линия. |
| `union` | Map | **да** | Способ объединения двух изображений: `add-weighted` (`weight1`, `weight2`, `gamma`) или `bitwise-and`. |
| `rope-dimensions.rope-width` | int | **да** | Стандартная ширина каната, px. |
| `rope-dimensions.width-tolerance` | f64 | **да** | Допуск ширины каната, %. |
| `rope-dimensions.square-tolerance` | f64 | **да** | Допуск площади каната, %. |
| `distortion-threshold` | f64 | **да** | Абсолютный порог обнаружения геометрических искажений. Рабочий диапазон `1.1..1.3`. |

### Параметры `defect-detection.fine-scan`

Точное сканирование: на его выходе формируются итоговые дефекты
(`expansion`, `compressing`, `hill`, `pit`).

| Параметр | Тип | Обяз. | Описание |
|----------|-----|-------|----------|
| `fine-contours.otsu-tune` | f64 | **да** | Коэффициент коррекции авто-порога Otsu. |
| `fine-contours.merge-distance` | f64 | **да** | Максимальное расстояние между контурами для их объединения. |
| `temporal-filter.*` | Map | **да** | Аналогично `fast-scan.temporal-filter`. |
| `fine-edges.otsu-tune` | f64 | нет | Множитель авто-порога Otsu. |
| `fine-edges.threshold` | int | нет | Порог `0..255`, используется вместо `otsu-tune`, если задан. |
| `fine-edges.smooth` | f64 | **да** | Коэффициент сглаживания линии границы. |
| `union` | Map | **да** | Способ объединения изображений: `add-weighted` или `bitwise-and`. |
| `rope-dimensions.*` | Map | **да** | Аналогично `fast-scan.rope-dimensions`. |
| `distortion-threshold` | f64 | **да** | Порог обнаружения геометрических искажений. |
| `defect-threshold` | f64 | **да** | Порог регистрации дефекта геометрии каната. |

### Параметры `camera <Имя>:`

| Параметр | Тип | Обяз. | Описание |
|----------|-----|-------|----------|
| `from-path` | String | нет | Путь к директории с файлами тестовых кадров. Задаётся только для тестирования. При наличии этого ключа остальные параметры камеры не применяются. |
| `fps` | Max \| Min \| f64 | **да** | Частота кадров. |
| `resolution.width` / `resolution.height` | int | **да** | Разрешение камеры. |
| `index` | int | нет | Индекс камеры, если IP-адрес динамический или неизвестен. |
| `address` | SocketAddr | нет | IP и порт камеры, если адрес задан статически. |
| `pixel-format` | enum | **да** | Формат пикселей: `Mono8/10/12/16`, `Bayer8/10/12/16`, `RGB8`, `BGR8`, `YCbCr8`, `YCbCr411`, `YUV422`, `YUV411`. Также поддерживаются сжатые `QOI_Mono8`, `QOI_BayerRG8`. По умолчанию и быстрее всего — `BayerRG8`. |
| `exposure.auto` | Off \| Continuous | **да** | Режим авто-экспозиции. |
| `exposure.time` | int | **да** | Время экспозиции, микросекунды. |
| `auto-packet-size` | bool | **да** | Автосогласование размера пакета потока (`StreamAutoNegotiatePacketSize`). Обычно увеличивает FPS и снижает нагрузку на CPU. |
| `channel-packet-size` | Max \| Min \| int | **да** | Максимальный размер пакета канала потока. |
| `resend-packet` | bool | **да** | Запрос повторной отправки потерянных UDP-пакетов (`StreamPacketResendEnable`). |

---


## Тесты

### Инлайн-тесты (`rope.rs`)

Модуль `tests` — проверка `Rope::segment_index`: попадание позиции каната
в границу сегмента с учётом `camera-offset` и `segment-threshold`, пограничные
и пропускаемые значения. Запуск:

```bash
cargo test --profile fast-test test_segment_index
```

### Интеграционный тест сервиса

[`frdm_service_defect_test.rs`](../../../src/tests/unit/services/frdm_service/frdm_service_defect_test.rs) —
полный запуск `FrdmService` через `ServiceTestPlanner` с эмуляцией шины
и подачей событий `Winch.RopePos`, `Load.MainBoomAngle`, `Load.RotaryBoomAngle`.
Камера работает в тестовом режиме `from-path` (кадры из директории
[`frames`](../../../src/tests/unit/services/frdm_service/frames)).
Помечен `#[ignore = "Isn't implemented yet."]` (требует API-сервер и БД).

```bash
cargo test --profile fast-test frdm_service_defect_test -- --ignored
```

---


## Эталонные конфиги

| Файл | Описание |
|------|----------|
| [`frdm-config.yaml`](../../../../frdm-config.yaml) | Рабочий конфиг с секцией `rope-defect` (камера `Camera1`, полный набор параметров `defect-detection`). |
| [`deprecation-test.yaml`](../../../src/tests/unit/services/frdm_service/deprecation-test.yaml) | Интеграционный конфиг с секцией `rope-defect` внутри полного `service FrdmService`. |

---


## Замечания

1. **Порядок запуска.** `RopeDefect` зависит от `Inputs` (`cam_segment_ix`) и от
   данных секции `rope-deprecation` (полная длина каната для расчёта `slices`).
   Порядок запуска в `FrdmService::run` строго:
   `rope_deprecation.run()` → `rope_defect.run()` → `inputs.run()`.

2. **`segment` и разрешение камеры** связаны рекомендацией:
   `segment = (0.10..0.20) * ширина кадра` (вдоль каната). При слишком большом
   сегменте кадр покрывает более одного сегмента, при слишком маленьком —
   теряется производительность и часть каната не анализируется.

3. **Детекция до инициализации позиции не запускается.** Пока
   `inputs.cam_segment_ix()` равна `None` (позиция каната ещё не приходила
   с шины), кадры с камеры пропускаются.

4. **`wait-started`** не является обязательным полем конфига, **но очень
   рекомендуется**: он гарантирует, что рабочий поток дошёл до
   `service_release.add` до возврата из `run`.

5. **Хранилище изображений.** Изображения дефектов сохраняются в
   `assets/files/<имя-сервиса>/rope-defects/<slice>/*.jpg`. Имя файла содержит
   дату, вид дефекта, timestamp и порядковый номер. Количество изображений
   на сегмент ограничено функцией БД `clean_frdm_defect_image` (10 штук).
