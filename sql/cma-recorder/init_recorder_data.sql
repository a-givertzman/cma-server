

INSERT INTO public.rec_basic_metric VALUES ('1.1                             ', 'string', 'crane-type', 'тип крана', 'unknown')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('1.2                             ', 'string', 'crane-index', 'индекс крана', 'unknown')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('1.3                             ', 'string', 'crane-vendor', 'наименование предприятия - изготовителя крана', 'ООО "ТКЗ"')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('1.4                             ', 'string', 'crane-serial-number', 'заводской номер крана', '000-000')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('1.5                             ', 'date', 'crane-manufacturing-date', 'год изготовления крана', '2023-03-03')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('1.6                             ', 'real', 'crane-capacity', 'грузоподъемность крана, тонны', '0.0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('1.7                             ', 'string', 'crane-classification-group', 'группа классификации (режима) крана', 'M8')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('1.7.1                           ', 'string', 'winch1-classification-group', 'группа классификации (режима) лебедки 1 крана', 'M8')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('1.7.2                           ', 'string', 'winch2-classification-group', 'группа классификации (режима) лебедки 2 крана', 'M8')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('1.7.3                           ', 'string', 'winch3-classification-group', 'группа классификации (режима) лебедки 3 крана', 'M8')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('1.8                             ', 'real', 'crane-nominal-characteristic-number', 'нор­мативное характеристическое число крана', '0.0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('1.8.1                           ', 'real', 'winch1-nominal-characteristic-number', 'нор­мативное характеристическое число лебедки 1 крана', '0.0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('1.8.2                           ', 'real', 'winch2-nominal-characteristic-number', 'нор­мативное характеристическое число лебедки 2 крана', '0.0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('1.8.3                           ', 'real', 'winch3-nominal-characteristic-number', 'нор­мативное характеристическое число лебедки 3 крана', '0.0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('1.9                             ', 'date', 'crane-commissioning-date', 'дата ввода крана в эксплуатацию', '2023-03-03')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('1.10                            ', 'int', 'crane-standard-service-life ', 'нормативный срок службы крана', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('2.1                             ', 'string', 'fault-recorder-type', 'тип Регистратора параметров', 'Fault Recorder')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('2.2                             ', 'string', 'fault-recorder-model', 'модификация Регистратора параметров', 'v0.0.1')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('2.3                             ', 'string', 'fault-recorder-vendor', 'наименование предприятия - изготовителя Регистратора параметров', 'ООО "ТКЗ"')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('2.4                             ', 'string', 'fault-recorder-serial-number', 'заводской номер Регистратора параметров', '000-000')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('2.5                             ', 'date', 'fault-recorder-manufacturing-date', 'год изготовления Регистратора параметров', '2023-03-03')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('2.6                             ', 'date', 'fault-recorder-installation-date', 'дата установки РП на кран', '2023-03-03')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('2.7                             ', 'string', 'fault-recorder-installation-company', 'наименование организации, установившей Регистратора параметров на кран', 'ООО "ТКЗ"')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.1                             ', 'real', 'crane-total-operating-hours', 'общее количество часов работы крана', '0.0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.2.0                           ', 'real', 'pump-total-operating-hours', 'общее количество часов работы насосной станции (моточасы)', '0.0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.2.1                           ', 'real', 'winch1-total-operating-hours', 'общее количество часов работы лебедки 1 (моточасы)', '0.0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.2.2                           ', 'real', 'winch2-total-operating-hours', 'общее количество часов работы лебедки 2 (моточасы)', '0.0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.2.3                           ', 'real', 'winch3-total-operating-hours', 'общее количество часов работы лебедки 3 (моточасы)', '0.0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.7.1                           ', 'real', 'winch1-characteristic-number', 'текущее характеристическое число лебедка 1', '0.0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.7.2                           ', 'real', 'winch2-characteristic-number', 'текущее характеристическое число лебедка 2', '0.0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.5                             ', 'real', 'crane-total-lifted-mass', 'суммарная масса поднятых грузов. тонн', '0.0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.5.1                           ', 'real', 'winch1-total-lifted-mass', 'суммарная масса поднятых грузов лебедка 1', '0.0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.5.2                           ', 'real', 'winch2-total-lifted-mass', 'суммарная масса поднятых грузов лебедка 2', '0.0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.5.3                           ', 'real', 'winch3-total-lifted-mass', 'суммарная масса поднятых грузов лебедка 3', '0.0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.6.2                           ', 'int', 'winch2-load-limiter-trip-count', 'количество срабатываний ограничителя грузоподъемности лебедка 2', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.6.3                           ', 'int', 'winch3-load-limiter-trip-count', 'количество срабатываний ограничителя грузоподъемности лебедка 3', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.7                             ', 'real', 'crane-characteristic-number', 'текущее характеристическое число для крана', '0.0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.3                             ', 'int', 'total-operating-cycles-count', 'суммарное число рабочих циклов', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.1.12                        ', 'real', 'cycles-1_15-1_25-load-range', 'циклов в диапазоне загрузки 1,15 - 1,25', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.1.13                        ', 'real', 'cycles-1_25-load-range', 'циклов в диапазоне загрузки 1,25 -', '0.0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.1.04                        ', 'real', 'cycles-0_35-0_45-load-range', 'циклов в диапазоне загрузки 0,35 - 0,45', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.1.06                        ', 'real', 'cycles-0_55-0_65-load-range', 'циклов в диапазоне загрузки 0,55 - 0,65', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.1.09                        ', 'real', 'cycles-0_85-0_95-load-range', 'циклов в диапазоне загрузки 0,85 - 0,95', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.1.03                        ', 'real', 'cycles-0_25-0_35-load-range', 'циклов в диапазоне загрузки 0,25 - 0,35', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.7.3                           ', 'real', 'winch3-characteristic-number', 'текущее характеристическое число лебедка 3', '0.0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.8                             ', 'real', 'crane-load-spectrum-factor', 'коэффициент распределения нагрузок для крана', '0.0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.8.1                           ', 'real', 'winch1-load-spectrum-factor', 'коэффициент распределения нагрузок лебедка 1', '0.0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.8.2                           ', 'real', 'winch2-load-spectrum-factor', 'коэффициент распределения нагрузок лебедка 2', '0.0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.8.3                           ', 'real', 'winch3-load-spectrum-factor', 'коэффициент распределения нагрузок лебедка 3', '0.0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.1.1                           ', 'real', 'crane-total-operating-secs', 'общее количество часов работы крана', '0.0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.6.1                           ', 'int', 'winch1-load-limiter-trip-count', 'количество срабатываний ограничителя грузоподъемности лебедка 1', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4                             ', 'real', 'cycles-distribution-by-load-ranges', 'распределение циклов по диапазонам нагрузки', '')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.1.10.L                      ', 'real', '0_95-1_05-load', 'нагрузка в диапазоне загрузки 0,95 - 1,05', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.1.10                        ', 'real', 'cycles-0_95-1_05-load-range', 'циклов в диапазоне загрузки 0,95 - 1,05', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.1.11                        ', 'real', 'cycles-1_05-1_15-load-range', 'циклов в диапазоне загрузки 1,05 - 1,15', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.1.11.L                      ', 'real', '1_05-1_15-load', 'нагрузка в диапазоне загрузки 1,05 - 1,15', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.1.12.L                      ', 'real', '1_15-1_25-load', 'нагрузка в диапазоне загрузки 1,15 - 1,25', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.1.13.L                      ', 'real', '1_25-load', 'нагрузка в диапазоне загрузки 1,25 -', '0.0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.1.03.L                      ', 'real', '0_25-0_35-load', 'нагрузка в диапазоне загрузки 0,25 - 0,35', '0.0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.1.04.L                      ', 'real', '0_35-0_45-load', 'нагрузка в диапазоне загрузки 0,35 - 0,45', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.1.05                        ', 'real', 'cycles-0_45-0_55-load-range', 'циклов в диапазоне загрузки 0,45 - 0,55', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.1.05.L                      ', 'real', '0_45-0_55-load', 'нагрузка в диапазоне загрузки 0,45 - 0,55', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.1.06.L                      ', 'real', '0_55-0_65-load', 'нагрузка в диапазоне загрузки 0,55 - 0,65', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.1.07                        ', 'real', 'cycles-0_65-0_75-load-range', 'циклов в диапазоне загрузки 0,65 - 0,75', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.1.07.L                      ', 'real', '0_65-0_75-load', 'нагрузка в диапазоне загрузки 0,65 - 0,75', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.1.08                        ', 'real', 'cycles-0_75-0_85-load-range', 'циклов в диапазоне загрузки 0,75 - 0,85', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.1.08.L                      ', 'real', '0_75-0_85-load', 'нагрузка в диапазоне загрузки 0,75 - 0,85', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.1.09.L                      ', 'real', '0_85-0_95-load', 'нагрузка в диапазоне загрузки 0,85 - 0,95', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.2.01                        ', 'real', 'winch1-cycles-0_05-0_15-load-range', 'циклов в диапазоне загрузки 0,05 - 0,15', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.2.02                        ', 'real', 'winch1-cycles-0_15-0_25-load-range', 'циклов в диапазоне загрузки 0,15 - 0,25', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.2.01.L                      ', 'real', 'winch1-0_05-0_15-load', 'нагрузка в диапазоне загрузки 0,05 - 0,15', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.2.02.L                      ', 'real', 'winch1-0_15-0_25-load', 'нагрузка в диапазоне загрузки 0,15 - 0,25', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.1.01                        ', 'real', 'cycles-0_05-0_15-load-range', 'циклов в диапазоне загрузки 0,05 - 0,15', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.1.01.L                      ', 'real', '0_05-0_15-load', 'нагрузка в диапазоне загрузки 0,05 - 0,15', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.1.02                        ', 'real', 'cycles-0_15-0_25-load-range', 'циклов в диапазоне загрузки 0,15 - 0,25', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.1.02.L                      ', 'real', '0_15-0_25-load', 'нагрузка в диапазоне загрузки 0,15 - 0,25', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.2.03                        ', 'real', 'winch1-cycles-0_25-0_35-load-range', 'циклов в диапазоне загрузки 0,25 - 0,35', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.2.03.L                      ', 'real', 'winch1-0_25-0_35-load', 'нагрузка в диапазоне загрузки 0,25 - 0,35', '0.28')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.2.04                        ', 'real', 'winch1-cycles-0_35-0_45-load-range', 'циклов в диапазоне загрузки 0,35 - 0,45', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.2.04.L                      ', 'real', 'winch1-0_35-0_45-load', 'нагрузка в диапазоне загрузки 0,35 - 0,45', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.2.05                        ', 'real', 'winch1-cycles-0_45-0_55-load-range', 'циклов в диапазоне загрузки 0,45 - 0,55', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.2.05.L                      ', 'real', 'winch1-0_45-0_55-load', 'нагрузка в диапазоне загрузки 0,45 - 0,55', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.2.06                        ', 'real', 'winch1-cycles-0_55-0_65-load-range', 'циклов в диапазоне загрузки 0,55 - 0,65', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.2.06.L                      ', 'real', 'winch1-0_55-0_65-load', 'нагрузка в диапазоне загрузки 0,55 - 0,65', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.2.07                        ', 'real', 'winch1-cycles-0_65-0_75-load-range', 'циклов в диапазоне загрузки 0,65 - 0,75', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.2.07.L                      ', 'real', 'winch1-0_65-0_75-load', 'нагрузка в диапазоне загрузки 0,65 - 0,75', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.2.08                        ', 'real', 'winch1-cycles-0_75-0_85-load-range', 'циклов в диапазоне загрузки 0,75 - 0,85', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.2.08.L                      ', 'real', 'winch1-0_75-0_85-load', 'нагрузка в диапазоне загрузки 0,75 - 0,85', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.2.09                        ', 'real', 'winch1-cycles-0_85-0_95-load-range', 'циклов в диапазоне загрузки 0,85 - 0,95', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.2.09.L                      ', 'real', 'winch1-0_85-0_95-load', 'нагрузка в диапазоне загрузки 0,85 - 0,95', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.2.10                        ', 'real', 'winch1-cycles-0_95-1_05-load-range', 'циклов в диапазоне загрузки 0,95 - 1,05', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.2.10.L                      ', 'real', 'winch1-0_95-1_05-load', 'нагрузка в диапазоне загрузки 0,95 - 1,05', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.2.11                        ', 'real', 'winch1-cycles-1_05-1_15-load-range', 'циклов в диапазоне загрузки 1,05 - 1,15', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.2.11.L                      ', 'real', 'winch1-1_05-1_15-load', 'нагрузка в диапазоне загрузки 1,05 - 1,15', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.2.12                        ', 'real', 'winch1-cycles-1_15-1_25-load-range', 'циклов в диапазоне загрузки 1,15 - 1,25', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.2.12.L                      ', 'real', 'winch1-1_15-1_25-load', 'нагрузка в диапазоне загрузки 1,15 - 1,25', '0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.2.13                        ', 'real', 'winch1-cycles-1_25-load-range', 'циклов в диапазоне загрузки 1,25 -', '0.0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;
INSERT INTO public.rec_basic_metric VALUES ('3.4.2.13.L                      ', 'real', 'winch1-1_25-load', 'нагрузка в диапазоне загрузки 1,25 -', '0.0')
	on conflict(name) do update set id = EXCLUDED.id, type = EXCLUDED.type, name = EXCLUDED.name, description = EXCLUDED.description, value = EXCLUDED.value;


INSERT INTO public.rec_name VALUES ('4               ', NULL, 'Trip count', 'Количество срабатываний');
INSERT INTO public.rec_name VALUES ('5               ', NULL, 'Diviation max', 'Максимальное отклонение');
INSERT INTO public.rec_name VALUES ('average_load    ', NULL, 'Average load', 'Средняя нагрузка за цикл');
INSERT INTO public.rec_name VALUES ('average         ', NULL, 'Average', 'Среднее арифметическое');
INSERT INTO public.rec_name VALUES ('max             ', NULL, 'Maximum', 'Максимальное');
INSERT INTO public.rec_name VALUES ('min             ', NULL, 'Minimum', 'Минимальное');
INSERT INTO public.rec_name VALUES ('max_load        ', NULL, 'Maximum load', 'Максимальная нашрузка');
INSERT INTO public.rec_name VALUES ('min_load        ', NULL, 'Minimum load', 'Минимальная нагрузка');