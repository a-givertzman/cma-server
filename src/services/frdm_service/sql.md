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
    value               text not null
    unit                text null
);
```

---

## frdm_defect

id  |  defect  |  timestamp  | count | acknowledged | deleted

```sql
-- FRDM | Defects type
-- Enum of geometry defect type`s
-- containing the position of defect withing a frame';
create type public.frdm_defect_enum as enum (
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
    id                  bigint not null,
    defect              frdm_defect_enum not null,
    camera              int2 not null,
    first               timestamp not null,
    last                timestamp not null,
    score               int8 default 0 not null,
    acknowledged        timestamp null,
    deleted             timestamp null,
    PRIMARY KEY (id, defect, camera)
);
```

---

## frdm_defect_image

id | frdm_defect_id | camera_id | path

```sql
-- FRDM | Images of the rope defects
create table public.frdm_defect_image (
    id                  bigserial not null,
    frdm_defect_id      frdm_defect_enum not null,
    camera_id           int2 not null,
    path                text not null,
    created             timestamp default current_timestamp not null,
    PRIMARY KEY (id, frdm_defect_id, camera_id)
);

-- FRDM | Insert or update defect and associated image
do $$
begin
	insert into public.frdm_defect (id, defect, first, last, score)
	    values (3, 'expansion', current_timestamp, current_timestamp, 1)
	on conflict (id, defect) do update 
	    set (last, score) = (current_timestamp, frdm_defect.score + 1);
    -- Image camera 1
    insert into public.frdm_defect_image (frdm_defect_id, camera, path)
        values (1, 1, 'assets/frdm/defect_image/1.jpeg');
    -- Image camera 1
    insert into public.frdm_defect_image (frdm_defect_id, camera, path)
        values (1, 2, 'assets/frdm/defect_image/1.jpeg');
    -- Image camera 1
    insert into public.frdm_defect_image (frdm_defect_id, camera, path)
        values (1, 3, 'assets/frdm/defect_image/1.jpeg');
    -- Image camera 1
    insert into public.frdm_defect_image (frdm_defect_id, camera, path)
        values (1, 4, 'assets/frdm/defect_image/1.jpeg');
	EXCEPTION
		WHEN others then
			rollback;
end; $$
language plpgsql;

-- FRDM | Function cleaning the old images keeping 10 imeges per rope slice for each defect tipe
CREATE OR REPLACE FUNCTION public.clean_frdm_defect_image(defect_id_ frdm_defect_enum, camera_id_ bigint)
 RETURNS TABLE(path text)
AS $function$
declare
	images numeric;
	cam record;
	err text;
	deleted_row record;
begin
	-- FRDM | Function cleaning the old images keeping 10 imeges per rope slice for each defect tipe
	for cam in select camera from public.frdm_defect_image group by camera
	loop
		select count(public.frdm_defect_image.id) into images from public.frdm_defect_image
		 	where frdm_defect_id = defect_id_ and camera = camera_id_;
		raise notice '%', format('clean_frdm_defect_image | Defect ' || defect_id_ || ' Camera[' || camera_id_ || '] images: ' || images);
		if images > 10 then
			raise notice '%', format('clean_frdm_defect_image | Cleaning Defect ' || defect_id_ || ' Camera[' || camera_id_ || '] images: ' || images);
		    for deleted_row in
			    delete from public.frdm_defect_image
			        where (id) in (
			            select id from public.frdm_defect_image fdi
						where fdi.frdm_defect_id = defect_id_ and fdi.camera = camera_id_
			            order by last
			            limit images - 10
						offset 1
			        )
		    	returning public.frdm_defect_image.id, public.frdm_defect_image.path
			loop
				raise notice '%', format('clean_frdm_defect_image | Deleted: ' || deleted_row);
				path := deleted_row.path;
				return next;
			end loop;
		end if;
	end loop;
	exception
		when others then
			GET STACKED DIAGNOSTICS err = PG_EXCEPTION_CONTEXT;
			raise warning '%', format('Frdm | clean_frdm_defect_image | error: ' || err);
end; $function$
language plpgsql;

-- FRDM | Trigger for `frdm_defect_image` table to call cleaning after each insert
create or replace trigger clean_frdm_defect_image
    after insert on public.frdm_defect_image
    for each row
    execute procedure clean_frdm_defect_image();
```

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
