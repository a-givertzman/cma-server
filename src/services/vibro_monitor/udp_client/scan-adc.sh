#!/bin/bash

export PORT=15180
export TIMEOUT=1
# Готовим payload в шестнадцатеричном виде для nc
export PAYLOAD=$(printf '\x22\x04')

# Функция проверки одного IP (экспортируем для xargs)
check_ip() {
    local ip=$1
    
    # Отправляем payload и переводим ответ в HEX (вырезаем только первые 4 символа, т.е. 2 байта)
    # Использование xxd или od позволяет безопасно читать бинарный ответ
    HEX_RESP=$(echo -n "$PAYLOAD" | nc -u -w $TIMEOUT $ip $PORT 2>/dev/null | od -t x1 -An | tr -d ' \n' | head -c 4)
    
    # Если ответ вообще есть
    if [ ! -z "$HEX_RESP" ]; then
        if [ "$HEX_RESP" = "2204" ]; then
            echo "[+] $ip -> УСПЕШНО (Ответ: 0x22 0x04)" >> responded_hosts.log
            echo "[+] $ip -> УСПЕШНО"
        else
            echo "[!] $ip -> ДЕВАЙС НАЙДЕН, но неверный ответ (HEX: $HEX_RESP)" >> responded_hosts.log
            echo "[!] $ip -> Неверный ответ от девайса"
        fi
    fi
}
export -f check_ip

echo "Запуск асинхронного сканирования 192.168.X.Y в 200 потоков..."
> responded_hosts.log

# Генерируем список всех IP и передаем в xargs
# -P 200 задает работу в 200 параллельных потоков
for x in {0..255}; do
    for y in {1..254}; do
        echo "192.168.$x.$y"
    done
done | xargs -I {} -P 200 bash -c 'check_ip "{}"'

echo "Сканирование завершено. Результаты в responded_hosts.log"
