# -*- coding: utf-8 -*-
# Формирует компактную сводку по шагам теста (переиспользует расчёт из gen_deprecation_target.py)
import importlib.util, sys
spec = importlib.util.spec_from_file_location("gen", "/tmp/depr/gen_deprecation_target.py")
# не выполняем повторно - выполняем только нужные вычисления: скопируем прогон без печати
exec(open("/tmp/depr/gen_deprecation_target.py").read().split("# ==================== прогон шагов теста")[0])
