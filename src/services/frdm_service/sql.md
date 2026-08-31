# FRDM (Rope Defects Monitoring)

## frdm_settings

 key                           |  value  |  unit
------------------------------ | ------- | ------
 winch1-rope-length            |  3000.0 |  m
 winch1-defect-slices          |  30000  |  
 winch1-deprecation-slices     |  60000  |  

```sql
-- FRDM | Setting parameters
create table public.frdm_settings (
    id                  varchar primary key not null,
    value               text not null,
    unit                text null
);
```

---

## frdm_defect

slice | defect | camera | first | last | score | acknowledged

```sql
-- FRDM | Defects type
-- Enum of geometry defect type`s
-- containing the position of defect withing a frame';
create type public.frdm_defect_kind as enum (
    -- Detecting both sides width growing
    'expansion',
    -- Detecting both sides width reduction
    'compressing',
    -- Detecting one side raising
    'hill',
    -- Detecting one side drooping
    'pit'
);

-- FRDM | Defects
create table public.frdm_defect (
    -- Номер сегмента каната
    slice               bigint not null,
    -- Вид дефекта
    defect              frdm_defect_kind not null,
    -- Номер камеры
    camera              int2 not null,
    -- Момент первой регистрации дефекта
    first               timestamp not null,
    -- Момент последней регистрации дефекта
    last                timestamp not null,
    -- UI покажет дефект при score >= 3
    score               int8 default 0 not null,
    -- Момент сдроса (score = 0) дефекта пользователем
    acknowledged        timestamp null,
    
    PRIMARY KEY (slice, defect, camera)
);
```

---

## frdm_defect_image

image_id | slice | defect | camera | path | created

```sql
-- FRDM | Images of the rope defects
create table public.frdm_defect_image (
    -- Локальный уникальный идентификатор изображения
    image_id            bigint generated always as identity primary key,
    -- Составной внешний ключ
    slice               bigint not null,
    defect              frdm_defect_kind not null,
    camera              int2 not null,
    -- Путь к файлу изображения
    path                text not null,
    -- Дата создания
    created             timestamp default current_timestamp not null,

    CONSTRAINT fk_frdm_defect 
            FOREIGN KEY (slice, defect, camera) 
            REFERENCES public.frdm_defect (slice, defect, camera)
            ON DELETE RESTRICT, -- для удаления записи дефекта сначала удалить все изображения
);
create index frdm_defect_image_defect_idx 
    on public.frdm_defect_image (slice, defect, camera, created);
```

### Пример вставки
```sql
-- FRDM | Insert or update defect and associated image
do $$
begin
    insert into public.frdm_defect (slice, defect, camera, first, last, score)
        values (3, 'expansion', 1, current_timestamp, current_timestamp, 1)
    on conflict (slice, defect, camera) do update
        set (last, score) = (current_timestamp, public.frdm_defect.score + 1);
    -- Image camera 1
    insert into public.frdm_defect_image (slice, defect, camera, path)
        values (3, 'expansion', 1, 'assets/frdm/defect_image/1.jpeg');
    -- Image camera 2
    insert into public.frdm_defect_image (slice, defect, camera, path)
        values (3, 'expansion', 2, 'assets/frdm/defect_image/1.jpeg');
end; $$
language plpgsql;
```

### Поиск устаревших изображений

```sql
-- FRDM | Function cleaning the old imeges keeping 10 imeges per rope slice for each defect kind
CREATE OR REPLACE FUNCTION public.clean_frdm_defect_image(
    slice_      bigint,             -- номер сегмента
    defect_     frdm_defect_kind,   -- вид дефекта
    camera_     integer,            -- номер камеры
    keep_       int default 10      -- сколько изображений оставить
)
RETURNS TABLE(path text)
AS $function$
begin
    return query
    with to_delete as (
        -- Сначала выбираем строго те ID, которые подлежат удалению
        select fdi.image_id
        from public.frdm_defect_image fdi
        where fdi.slice = slice_ and fdi.defect = defect_ and fdi.camera = camera_
        order by fdi.created asc, fdi.image_id asc -- самые старые в начале
        offset keep_ -- пропускаем первые keep_ (свежих) и берем все, что дальше
    )
    delete from public.frdm_defect_image
    where image_id in (select image_id from to_delete)
    returning public.frdm_defect_image.path;
end;
$function$
language plpgsql;
```

---


## frdm_deprecation

id  |  deprecation

```sql
-- FRDM | Rope deprecation values
-- Rope devided for slices, deprecation value calculated for each slice
-- sliceLingth = ropeLength / slices'
create table public.frdm_deprecation (
    id                  bigserial primary key not null,
    deprecation         numeric(24, 8) default 0.0 not null	
);
