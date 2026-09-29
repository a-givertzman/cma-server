#!/bin/bash

echo "=================================="
echo "Сканер Vibro-ADC устройств в сети"
echo "=================================="

# Проверяем, запущен ли скрипт от root (для tcpdump)
if [ "$EUID" -ne 0 ]; then
  echo "Пожалуйста, запустите скрипт от имени sudo / root!"
  exit 1
fi

PORT=15180
LOG_FILE="scan-adc.log"
TMP_RAW_LOG="scan-adc.log.tmp"
> "$LOG_FILE"
> "$TMP_RAW_LOG"

# 1. ЗАПУСКАЕМ ТРАФИК-МАСТЕР В ФОНЕ
# Он слушает любой входящий UDP-трафик с порта 15180
echo "Включаем прослушивание сети..."
tcpdump -l -n -i any "udp src port $PORT" > "$TMP_RAW_LOG" 2>/dev/null &
TCPDUMP_PID=$!

# Даем tcpdump полсекунды, чтобы инициализировать интерфейс
sleep 0.5

echo "Генерируем список IP-адресов 192.168.X.Y..."
IPS=()
for x in {0..255}; do
    for y in {1..254}; do
        IPS+=("192.168.$x.$y")
    done
done

echo "Отправляем пакеты \x22\x04..."
PAYLOAD=$(printf '\x22\x04')
BATCH_SIZE=800
COUNTER=0

for ip in "${IPS[@]}"; do
    # Отправляем пакет асинхронно
    (echo -n "$PAYLOAD" > /dev/udp/$ip/$PORT) 2>/dev/null &
    
    ((COUNTER++))
    if (( COUNTER % BATCH_SIZE == 0 )); then
        # Пауза, чтобы не забить ARP-таблицу роутера
        sleep 0.04
    fi
done

echo "Все пакеты отправлены. Ожидаем оставшиеся ответы (3 секунды)..."
sleep 3

# 2. ОСТАНАВЛИВАЕМ TCPDUMP
kill $TCPDUMP_PID 2>/dev/null
wait $TCPDUMP_PID 2>/dev/null

echo "Готовим результаты сканирования..."

# 3. ОБРАБАТЫВАЕМ СОБРАННЫЙ ЛОГ
# Извлекаем IP-адреса отправителей из лога tcpdump
if [ -s "$TMP_RAW_LOG" ]; then
    # echo "Обработка результатов..."
    # Строка tcpdump выглядит так: 12:34:56.789 IP 192.168.1.50.15180 > ...
    # Вырезаем IP адрес устройства
    awk '{
        for(i=1;i<=NF;i++) {
            if($i=="IP") {
                split($(i+1), a, ".");
                print a[1]"."a[2]"."a[3]"."a[4];
            }
        }
    }' "$TMP_RAW_LOG" | sort -u > "$LOG_FILE"
fi

rm -f "$TMP_RAW_LOG"

echo "====== Найденные устройства ======"
cat "$LOG_FILE"
echo "=================================="
