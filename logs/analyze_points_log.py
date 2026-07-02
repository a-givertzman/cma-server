#
# Анализирует лог евентов на входе MultiQueue
#
# `logs/App/MultiQueue/points.log`
#
#
# Запустить командой для получения статистики по появлению евентов на вхоже MultiQueue
#
#  python3 ./scan_points_log_py ./points-2.log
#
# Event Path                                                                       | Count | Avg Interval
# --------------------------------------------------------------------------------------------------------------
# Inf:/App/ied13/db905_visual_data_fast/Winch1.EncoderBR1                          | 2028  | 10 ms
# Inf:/App/ied13/db905_visual_data_fast/Winch1.EncoderBR2                          | 2028  | 10 ms
# Inf:/App/ied13/db905_visual_data_fast/Winch1.LVDT1                               | 2028  | 10 ms
# Inf:/App/ied13/db905_visual_data_fast/Winch1.LVDT2                               | 2028  | 10 ms
#
#
# Запустить командой для получения фильра по имени евента  на вхоже MultiQueue
#
#  python3 ./scan_points_log_py ./points-2.log Inf:/App/ied13/db905_visual_data_fast/Winch1.EncoderBR1
#
# 'Inf:/App/ied13/db905_visual_data_fast/Winch1.EncoderBR1'  |  Int  |   txid: 10879087742985427551:
# 2026-03-10T13:26:19.144159334Z | Inf   | Invalid  | 0
# 2026-03-10T13:26:19.154182188Z | Inf   | Ok       | 0
# 2026-03-10T13:26:19.164159911Z | Inf   | Ok       | 0
#
#
import re, sys
from collections import defaultdict
from datetime import datetime

# Регулярка для захвата Эвента и Таймстампа
# Группа 1: Путь эвента (в кавычках)
# Группа 2: Таймстамп (ISO формат в конце строки)
pattern = re.compile(r"^'([^']+)'[:\s\S]+timestamp:\s*([\d\-T:\.Z]+)")

def analyze_log (path):
    # Словарь для хранения списка всех таймстампов каждого эвента
    event_times = defaultdict(list)
    
    with open(path, 'r', encoding='utf-8') as f:
        for line in f:
            match = pattern.search(line.strip())
            if match:
                event_name = match.group(1)
                ts_str = match.group(2).replace('Z', '') # Убираем Z для парсинга
                try:
                    # Парсим время (учитываем наносекунды, если они есть)
                    ts_dt = datetime.fromisoformat(ts_str)
                    event_times[event_name].append(ts_dt)
                except ValueError:
                    continue

    print(f"{'Event Path':<80} | {'Count':<5} | {'Avg Interval'}")
    print("-" * 110)

    # Сортируем эвенты по количеству (самые частые вверху)
    sorted_events = sorted(event_times.items(), key=lambda x: len(x[1]), reverse=True)

    for event, times in sorted_events:
        count = len(times)
        if count < 2:
            avg_str = "N/A"
        else:
            # Сортируем времена на случай, если лог не упорядочен
            times.sort()
            total_duration = (times[-1] - times[0]).total_seconds()
            # Интервалов на 1 меньше, чем записей
            avg_ms = (total_duration / (count - 1)) * 1000
            
            if avg_ms >= 1000:
                avg_str = f"{avg_ms/1000:.2f} s"
            else:
                avg_str = f"{int(avg_ms)} ms"

        print(f"{event:<80} | {count:<5} | {avg_str}")

import re

# Регулярка для детального разбора строки
# Группа 1: Тип (Real/Int)
# Группа 2: txid
# Группа 3: value
# Группа 4: status
# Группа 5: cot
# Группа 6: timestamp
LOG_PARSE_RE = re.compile(
    r":\s*(\w+)\(PointHlr\s*\{\s*txid:\s*(\d+),.*value:\s*([\d\.]+),.*status:\s*(\w+),.*cot:\s*(\w+),.*timestamp:\s*([\d\-T:\.Z]+)"
)

def filter_event(file_path, event_name):
    header_printed = False
    
    with open(file_path, 'r', encoding='utf-8') as f:
        for line in f:
            line = line.strip()
            # Проверяем, начинается ли строка с нужного эвента
            if line.startswith(f"'{event_name}'"):
                match = LOG_PARSE_RE.search(line)
                if match:
                    data_type = match.group(1)
                    txid = match.group(2)
                    value = match.group(3)
                    status = match.group(4)
                    cot = match.group(5)
                    ts = match.group(6)

                    # Печатаем заголовок один раз при первом совпадении
                    if not header_printed:
                        print(f"'{event_name}'  |  {data_type}  |   txid: {txid}:")
                        header_printed = True
                    
                    # Печатаем строку данных
                    print(f"{ts:<30} | {cot:<5} | {status:<8} | {value}")

if __name__ == "__main__":
    if len(sys.argv) > 1:
        # Путь к файлу лога
        path = sys.argv[1]
        filter = sys.argv[2]
        print(f"analyzing '{path}'...")
        if filter:
            analyze_log(path)
            print(f"Filtering for: {filter}\n")
            filter_event(path, filter)
        else:
            analyze_log(path)
    else:
        print("File to analize is not specified")
