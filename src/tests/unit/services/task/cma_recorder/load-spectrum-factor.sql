select SUM (value * pow(load_value, 3))
	from (
        select
			om.name as name,
			cast(om.value as decimal) as value,
			om1.name as load_name,
			cast(om1.value as decimal) as load_value
		FROM operating_metric om
		    INNER JOIN operating_metric om1 ON (substring (om.name, 'cycles-(\d_\d\d-\d_\d\d)-load-range') || '-load') = om1.name
	) as cycles;


select name, cast(value as decimal) from operating_metric om where name like 'cycles-_\___%load-range';

SELECT  u.username
        , u.name
        , u.email
        , m.access_level
FROM users u
    JOIN members m ON (u.id = m.user_id)
;
select
	om.name as name,
	cast(om.value as decimal) as value,
	om1.name as load_name,
	cast(om1.value as decimal) as load_value
FROM operating_metric om
    INNER JOIN operating_metric om1 ON (substring (om.name, 'cycles-(\d_\d\d-\d_\d\d)-load-range') || '-load') = om1.name;

select regexp_replace('crane-index', 'cycles-(\d_\d\d-\d_\d\d)-load-range', '\1_load');
select regexp_replace('cycles-0_05-0_15-load-range', 'cycles-(\d_\d\d-\d_\d\d)-load-range', '\1_load', 'g');
select regexp_match('cycles-0_05-0_15-load-range', 'cycles-(\d_\d\d-\d_\d\d)-load-range');
select substring ('cycles-0_05-0_15-load-range', 'cycles-(\d_\d\d-\d_\d\d)-load-range');
select substring ('crane-index', 'cycles-(\d_\d\d-\d_\d\d)-load-range');

select name, cast(value as decimal) from operating_metric om where name like 'cycles-_\___%load-range';
select (substring (om.name, 'cycles-(\d_\d\d-\d_\d\d)-load-range') || '-load') as name, value from operating_metric om;


