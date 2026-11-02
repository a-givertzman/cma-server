#!/usr/bin/env bash
# Без set -e — нам важно продолжать при любых ошибках psql/kill

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# === НАСТРОЙКИ ===
DB_USER="crane_data_server"
DB_PORT="5432"
DB_HOST="0.0.0.0"
DB_NAME="crane_data_server"
DB_INIT_SQL="sql/cma-recorder/init_recorder_data.sql"
DB_PASSWORD="00d0-25e4-*&s2-ccds"

CMA_RECORDER_CFG="src/tests/unit/services/task/cma_recorder/cma-recorder.yaml"
PROFILE="fast-test"
BASIC_METRIC_PATH="src/tests/integration/cma_recorder/0.3.2"
GRADIENTS_PATH="src/tests/integration/cma_recorder/0.3.2/gradients"

RUN_TIME="${RUN_TIME:-200}"

CONFIGS=(
    "${BASIC_METRIC_PATH}/rec_basic_metric_test.yaml"
    "${GRADIENTS_PATH}/gradients_w1.yaml"
    "${GRADIENTS_PATH}/gradients_w2.yaml"
    "${GRADIENTS_PATH}/gradients_w3.yaml"
)

# === ФУНКЦИИ ===
log_info()  { echo -e "${GREEN}[INFO]${NC} $*"; }
log_warn()  { echo -e "${YELLOW}[WARN]${NC} $*"; }
log_error() { echo -e "${RED}[ERROR]${NC} $*"; }

clean_db() {
    log_info "Очистка БД через ${DB_INIT_SQL}..."
    PGPASSWORD="${DB_PASSWORD}" psql \
        -U "${DB_USER}" \
        -p "${DB_PORT}" \
        -h "${DB_HOST}" \
        -d "${DB_NAME}" \
        -f "${DB_INIT_SQL}"
    log_info "БД очищена."
}

# Убить всё, что относится к нашему запуску
kill_all() {
    # Убиваем именно cargo run с нашим конфигом
    pkill -TERM -f "cargo run --profile ${PROFILE}" 2>/dev/null || true
    # И сам бинарь cma_server (или как он у тебя называется)
    pkill -TERM -f "cma_server" 2>/dev/null || true
    sleep 2
    pkill -KILL -f "cargo run --profile ${PROFILE}" 2>/dev/null || true
    pkill -KILL -f "cma_server" 2>/dev/null || true
}

run_program() {
    local cfg="$1"

    log_info "Запуск программы с конфигом: ${cfg}"

    # Запускаем в фоне. Вывод идёт в консоль как обычно.
    cargo run --profile "${PROFILE}" -- --config "${cfg}" "${CMA_RECORDER_CFG}" &
    local prog_pid=$!
    log_info "PID процесса: ${prog_pid}"

    # Ждём ровно RUN_TIME секунд
    log_info "Жду ${RUN_TIME} секунд..."
    sleep "${RUN_TIME}"

    log_info "Время вышло. Завершаю программу..."

    # Сначала пробуем мягко убить по PID
    kill -TERM "${prog_pid}" 2>/dev/null || true
    # И всю группу — но через pkill -P (дети)
    pkill -TERM -P "${prog_pid}" 2>/dev/null || true

    sleep 3

    # Если что-то живо — жёстко
    if kill -0 "${prog_pid}" 2>/dev/null; then
        log_warn "PID ${prog_pid} ещё жив, добиваю..."
        kill -KILL "${prog_pid}" 2>/dev/null || true
    fi

    # Финальная зачистка по имени — на случай, если cargo уже умер,
    # а его дочерний процесс остался.
    kill_all

    # Ждём завершения фоновой задачи, чтобы shell не сыпал "[1]+ Terminated"
    wait "${prog_pid}" 2>/dev/null || true

    log_info "Программа остановлена."
}

# === ОСНОВНОЙ ЦИКЛ ===
for cfg in "${CONFIGS[@]}"; do
    echo "=============================================="
    log_info "Обработка конфига: ${cfg}"

    kill_all       # на всякий случай, если что-то осталось от прошлого прогона
    clean_db
    run_program "${cfg}"
    clear
done

log_info "Финальная очистка БД..."
kill_all
clean_db

log_info "Все задачи выполнены."