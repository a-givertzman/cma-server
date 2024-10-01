
drop table public.tree;
create table public.tree (
    id       int8,
    parent       int8,
    ename       varchar(255),
    job         varchar(255),
    hiredate    date, 
    sal         numeric(20, 2),
    comm        numeric(20, 2),
    deptno      int2
);

insert into public.tree (
    id,
    parent, 
    ename, 
    job,   
    hiredate, 
    sal,  
    comm,  
    deptno
) values
    (1000 , null , 'KING'   , 'PRESIDENT' , '1981-11-17' , 5000.00 ,    null ,     10),
        (1100 , 1000 , 'JONES'  , 'MANAGER'   , '1981-04-02' , 2975.00 ,    null ,     20),
            (1110 , 1100 , 'SCOTT'  , 'ANALYST'   , '1987-04-19' , 3000.00 ,    null ,     20),
                (1111 , 1110 , 'ADAMS'  , 'CLERK'     , '1987-05-23' , 1100.00 ,    null ,     20),
            (1120 , 1100 , 'FORD'   , 'ANALYST'   , '1981-12-03' , 3000.00 ,    null ,     20),
                (1121 , 1120 , 'SMITH'  , 'CLERK'     , '1980-12-17' ,  800.00 ,    null ,     20),
        (1200 , 1000 , 'BLAKE'  , 'MANAGER'   , '1981-05-01' , 2850.00 ,    null ,     30),
            (1210 , 1200 , 'ALLEN'  , 'SALESMAN'  , '1981-02-20' , 1600.00 ,  300.00 ,     30),
            (1220 , 1200 , 'WARD'   , 'SALESMAN'  , '1981-02-22' , 1250.00 ,  500.00 ,     30),
            (1230 , 1200 , 'MARTIN' , 'SALESMAN'  , '1981-09-28' , 1250.00 , 1400.00 ,     30),
            (1240 , 1200 , 'TURNER' , 'SALESMAN'  , '1981-09-08' , 1500.00 ,    0.00 ,     30),
            (1250 , 1200 , 'JAMES'  , 'CLERK'     , '1981-12-03' ,  950.00 ,    null ,     30),
        (1300 , 1000 , 'CLARK'  , 'MANAGER'   , '1981-06-09' , 2450.00 ,    null ,     10),
            (13010 , 1300 , 'MILLER' , 'CLERK'     , '1982-01-23' , 1300.00 ,    null ,     10),
            (13020 , 1300 , 'MILLER' , 'CLERK'     , '1982-01-23' , 1300.00 ,    null ,     10),
            (13030 , 1300 , 'MILLER' , 'CLERK'     , '1982-01-23' , 1300.00 ,    null ,     10),
            (13040 , 1300 , 'MILLER' , 'CLERK'     , '1982-01-23' , 1300.00 ,    null ,     10),
            (13050 , 1300 , 'MILLER' , 'CLERK'     , '1982-01-23' , 1300.00 ,    null ,     10),
            (13060 , 1300 , 'MILLER' , 'CLERK'     , '1982-01-23' , 1300.00 ,    null ,     10),
            (13070 , 1300 , 'MILLER' , 'CLERK'     , '1982-01-23' , 1300.00 ,    null ,     10),
            (13080 , 1300 , 'MILLER' , 'CLERK'     , '1982-01-23' , 1300.00 ,    null ,     10),
            (13090 , 1300 , 'MILLER' , 'CLERK'     , '1982-01-23' , 1300.00 ,    null ,     10),
            (13100 , 1300 , 'MILLER' , 'CLERK'     , '1982-01-23' , 1300.00 ,    null ,     10),
            (13110 , 1300 , 'MILLER' , 'CLERK'     , '1982-01-23' , 1300.00 ,    null ,     10),
            (13120 , 1300 , 'MILLER' , 'CLERK'     , '1982-01-23' , 1300.00 ,    null ,     10),
            (13130 , 1300 , 'MILLER' , 'CLERK'     , '1982-01-23' , 1300.00 ,    null ,     10),
            (13140 , 1300 , 'MILLER' , 'CLERK'     , '1982-01-23' , 1300.00 ,    null ,     10),
            (13150 , 1300 , 'MILLER' , 'CLERK'     , '1982-01-23' , 1300.00 ,    null ,     10),
            (13160 , 1300 , 'MILLER' , 'CLERK'     , '1982-01-23' , 1300.00 ,    null ,     10),
            (13170 , 1300 , 'MILLER' , 'CLERK'     , '1982-01-23' , 1300.00 ,    null ,     10)
;

select * from public.tree;

WITH RECURSIVE cte as (
    SELECT t.id, t.parent, t.ename, t.job, 0 as level, '/' as path
    FROM public.tree t
    where t.parent is null
  UNION ALL
    SELECT t.id, t.parent, t.ename, t.job, c.level + 1, c.path || t.parent || '/' 
    FROM cte c
    JOIN public.tree t on t.parent = c.id
)
SELECT * FROM cte ORDER BY path;

-- ============================================================================

drop table public.iso_tree;
create table public.iso_tree (
    parent      varchar(24),
    id          varchar(24),
    iso_name    varchar(255) UNIQUE,
    iso_data    varchar(255),
    PRIMARY KEY (parent, id)
);

do
$$
declare
	id0	varchar(24);
	id1	varchar(24);
	id2	varchar(24);
	id3	varchar(24);
	prt0 varchar(24);
	prt1 varchar(24);
	prt2 varchar(24);
	prt3 varchar(24);
begin
	select 1 into id0;
	insert into public.iso_tree (parent, id, iso_name, iso_data) values (0, id0 , 'ROOT'   , 'DATA');
    for i1 in 1..150 loop
		select concat(id0, ':', i1) into id1;
        insert into public.iso_tree (parent, id, iso_name, iso_data) values (id0, id1, id0 || '/Child-' || id1, 'DATA');
	    for i2 in 1..150 loop
			select concat(id1, ':', i2) into id2;
	        insert into public.iso_tree (parent, id, iso_name, iso_data) values (id1, id2, id1 || '/Child-' || id2, 'DATA');
		    for i3 in 1..150 loop
				select concat(id2, ':', i3) into id3;
		        insert into public.iso_tree (parent, id, iso_name, iso_data) values (id2, id3, id2 || '/Child-' || id3, 'DATA');
		    end loop;
	    end loop;
    end loop;
end;
$$ LANGUAGE plpgsql;

select * from public.iso_tree;
select count(*) from public.iso_tree;

WITH RECURSIVE cte as (
    SELECT t.parent, t.id, t.iso_name, t.iso_data, 0 as level, '/' as path
    FROM public.iso_tree t
    where t.parent = '0'
  UNION ALL
    SELECT t.parent, t.id, t.iso_name, t.iso_data, c.level + 1, c.path || t.parent || '/' 
    FROM cte c
    JOIN public.iso_tree t on t.parent = c.id
)
SELECT * FROM cte ORDER BY path;

WITH RECURSIVE cte as (
    SELECT t.parent, t.id, t.iso_name, t.iso_data
    FROM public.iso_tree t
    where t.parent = '0'
  UNION ALL
    SELECT t.parent, t.id, t.iso_name, t.iso_data
    FROM cte c
    JOIN public.iso_tree t on t.parent = c.id
)
SELECT * FROM cte ORDER BY iso_name;



