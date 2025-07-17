# FRDM (Rope Defects Monitoring)

**frdm_defect**
id  |  defect  |  timestamp  | count | acknowledged | deleted

```sql
comment on type public.frdm_defect_enum is E''
    'Enum of geometry defect type`s'
    'containing the position of defect withing a frame';

create type public.frdm_defect_enum as enum (
    -- Detecting both sides width growing
    'expansion',
    -- Detecting both sides width reduction
    'compressing',
    -- Detecting one side raising
    'hill',
    -- Detecting one side drooping
    'pit',
);

comment on type public.frdm_defect is E''
    'FRDM (Fiber Rope Defects Monitoring) defects'
create table public.frdm_defect (
    id                  bigserial primary key not null,
    defect              frdm_defect_enum not null,
    timestamp           timestamp not null,
    count               int8 default 0 not null,
    acknowledged        timestamp null,
    deleted             timestamp null,
);
```

**frdm_defect_image**
id | frdm_defect_id | camera_id | path

```sql
comment on type public.frdm_defect is E''
    'FRDM images '
create table public.frdm_defect_image (
    id                  bigserial primary key not null,
    frdm_defect_id      int8 not null,
    camera_id           int2 not null,
    path                text not null,
);
```

**frdm_deprecation**
id  |  deprecation

```sql
comment on type public.frdm_defect is E''
    'FRDM (Fiber Rope Defects Monitoring) deprecation values'
    'Rope devided for slices, deprecation value calculated for each slice'
    'sliceLingth = ropeLength / slices'
create table public.frdm_deprecation (
    id                  bigserial primary key not null,
    deprecation         double default 0.0 not null,
);
```
