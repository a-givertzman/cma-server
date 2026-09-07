# -*- coding: utf-8 -*-
"""
Генерация целевых значений «износа» (Deprecation) для теста
src/tests/unit/services/frdm_service/deprecation_test.rs

Геометрические формулы скопированы ПОСИМВОЛЬНО из образцового расчёта
design/frdm-service/spu-tnpa-optimist/spu-tnpa-optimist.py (конфиг КСШ = конфиг теста).
Логика Deprecation воспроизводит Rust: Blocks -> RopeSections -> BlockArcs -> Bendings -> Deprecation.
"""
import math
from copy import deepcopy

# ==================== формулы из spu-tnpa-optimist.py ====================
class Offset:
    def __init__(self, x, y):
        self.x = x
        self.y = y
class Boom:
    def __init__(self, alpha_rel, len, l1, l2, l3, l4):
        self.alpha_rel = alpha_rel
        self.alpha = 0.0
        self.len = len
        self.l1 = l1
        self.l2 = l2
        self.l3 = l3
        self.l4 = l4
        self.D = Offset(0.0, 0.0)
        self.G = Offset(0.0, 0.0)
class Block:
    def __init__(self, lF, D, scheme, bind):
        self.lF = lF
        self.D = D
        self.scheme = scheme
        self.bind = bind
        self.coord = Offset(0.0, 0.0)
        self.skipped = False
class BlockBindFixed:
    pass
class BlockBindBoom:
    def __init__(self, boom):
        self.boom = boom
class BlockBindHook:
    pass

def XY_rotate(lx, ly, alpha):
    angle_rad = math.radians(alpha)
    x = lx * math.cos(angle_rad) - ly * math.sin(angle_rad)
    y = lx * math.sin(angle_rad) + ly * math.cos(angle_rad)
    return x, y
def alpha_horiz(Y1, Y2, X1, X2):
    length = math.sqrt((X2 - X1) ** 2 + (Y2 - Y1) ** 2)
    if length == 0:
        return 0
    a = math.degrees(math.asin((Y1 - Y2) / length))
    if X1 <= X2:
        return a
    else:
        return 180 - a
def l_section(Y1, Y2, X1, X2):
    return math.sqrt((X2 - X1) ** 2 + (Y2 - Y1) ** 2)
def normalize_angle_deg(angle):
    return (angle + 180) % 360 - 180
def get_scheme_params(scheme):
    if scheme == 1:
        return -1, 1
    elif scheme == 2:
        return 1, 1
    elif scheme == 3:
        return 1, -1
    elif scheme == 4:
        return -1, -1
    else:
        raise ValueError(f"Некорректная схема: {scheme}")
def rope_parameters(X1, Y1, X2, Y2, D1, D2, k, j, block_pair=None):
    l_block = l_section(Y1, Y2, X1, X2)
    if l_block == 0:
        raise ValueError(f"Блоки {block_pair}: расстояние между центрами равно 0")
    alpha_block = alpha_horiz(Y1, Y2, X1, X2)
    asin_arg = 0.5 * (D1 + k * D2) / l_block
    if abs(asin_arg) > 1:
        raise ValueError(f"Невозможная геометрия каната для пары блоков {block_pair}")
    alpha_rope = normalize_angle_deg(
        alpha_block + j * math.degrees(math.asin(asin_arg)))
    X1_block = X1 + j * 0.5 * D1 * math.sin(math.radians(alpha_rope))
    Y1_block = Y1 + j * 0.5 * D1 * math.cos(math.radians(alpha_rope))
    X2_block = X2 - j * k * 0.5 * D2 * math.sin(math.radians(alpha_rope))
    Y2_block = Y2 - j * k * 0.5 * D2 * math.cos(math.radians(alpha_rope))
    l_rope = l_section(Y2_block, Y1_block, X2_block, X1_block)
    return {
        "l_block": l_block,
        "alpha_block": alpha_block,
        "alpha_rope": alpha_rope,
        "X1_block": X1_block,
        "Y1_block": Y1_block,
        "X2_block": X2_block,
        "Y2_block": Y2_block,
        "l_rope": l_rope,
    }
def calc_rope_data(blocks):
    rope_data = []
    for i, block in enumerate(blocks[:-1]):
        X1 = block.coord.x
        Y1 = block.coord.y
        X2 = blocks[i + 1].coord.x
        Y2 = blocks[i + 1].coord.y
        D1 = block.D
        D2 = blocks[i + 1].D
        scheme = block.scheme
        k, j = get_scheme_params(scheme)
        params = rope_parameters(X1, Y1, X2, Y2, D1, D2, k, j, block_pair=(i + 1, i + 2))
        params["block_pair"] = (i + 1, i + 2)
        params["scheme"] = scheme
        rope_data.append(params)
    return rope_data

# ==================== конфигурация КСШ (как в тесте) ====================
cfg_booms = [
    Boom(alpha_rel=0, len=11200.0, l1=0.0, l2=0.0, l3=0.0, l4=10330.0),
    Boom(alpha_rel=23.78, len=7984.0, l1=0.0, l2=0.0, l3=0.0, l4=0.0),
]
cfg_blocks = [
    Block(lF=Offset(1830.0,   710.0), D=845.000, scheme=1, bind=BlockBindFixed()),
    Block(lF=Offset(308.0,   1100.0), D=816.000, scheme=1, bind=BlockBindBoom(0)),
    Block(lF=Offset(-6549.0, 1730.0), D=816.000, scheme=1, bind=BlockBindBoom(1)),
    Block(lF=Offset(-1121.0,  973.0), D=816.000, scheme=2, bind=BlockBindBoom(1)),
    Block(lF=Offset(267.0,    860.0), D=816.000, scheme=3, bind=BlockBindBoom(1)),
    Block(lF=Offset(136.0,    -35.0), D=816.000, scheme=1, bind=BlockBindBoom(1)),
    Block(lF=Offset(0.0,        0.0), D=0.0,     scheme=0, bind=BlockBindHook()),
]
RUST_BIND = ["Drum", "Fixed", "Boom", "Boom", "Boom", "Boom", "Hook"]
# индекс блока с deflector-angle 90 в конфиге (блок 6, 0-based = 5)
DEFLECTOR_IX = 5
DEFLECTOR_DEG = 90.0

rope_len_mm = 85045.0
aux_length = 1200.0
seg_deg = 100.0
parking_angles = [0.0, 23.78]

booms = deepcopy(cfg_booms)
blocks = deepcopy(cfg_blocks)

# ==================== 1. Углы стрел ====================
alpha_sum = 0.0
for i, boom in enumerate(booms):
    alpha_sum += parking_angles[i]
    boom.alpha = alpha_sum - i * 180

# ==================== 2. D и G ====================
for i, boom in enumerate(booms):
    if i == 0:
        x0, y0 = 0, 0
        alpha_prime = 90
    else:
        x0, y0 = booms[i - 1].G.x, booms[i - 1].G.y
        alpha_prime = booms[i - 1].alpha
    wx, wy = XY_rotate(boom.l4, boom.l3, alpha_prime)
    XY_start = Offset(x0 + wx, y0 + wy)
    Dx, Dy = XY_rotate(-boom.l2, boom.l1, boom.alpha)
    boom.D = Offset(XY_start.x + Dx, XY_start.y + Dy)
    Gx, Gy = XY_rotate(boom.len - boom.l2, boom.l1, boom.alpha)
    boom.G = Offset(XY_start.x + Gx, XY_start.y + Gy)

# ==================== 3. Координаты блоков ====================
for idx, block in enumerate(blocks):
    if isinstance(block.bind, BlockBindFixed):
        dx1, dy1 = XY_rotate(-block.lF.x, block.lF.y, 0)
        dx2, dy2 = XY_rotate(booms[0].l4, booms[0].l3, 90)
        block.coord.x = dx1 + dx2
        block.coord.y = dy1 + dy2
    elif isinstance(block.bind, BlockBindBoom):
        boom = booms[block.bind.boom]
        base_point = boom.G
        dx, dy = XY_rotate(block.lF.x, block.lF.y, boom.alpha)
        block.coord.x = base_point.x + dx
        block.coord.y = base_point.y + dy
    elif isinstance(block.bind, BlockBindHook):
        block.coord.x = float("nan")
        block.coord.y = float("nan")

# ==================== 4. Крюк (первично - под блоком 6) ====================
hook_ix = 6
blocks[hook_ix].coord.x = blocks[5].coord.x + 0.5 * blocks[5].D
blocks[hook_ix].coord.y = blocks[5].coord.y - aux_length

# ==================== 5. Прямые участки (до проверки перекидывания) ====================
rope_data = calc_rope_data(blocks)
alpha_56 = rope_data[4]["alpha_rope"]     # участок блок5 -> блок6 (0-based 4->5)
print(f"alpha каната (блок5 -> блок6): {alpha_56:.6f} deg, |alpha| > 90: {abs(alpha_56) > DEFLECTOR_DEG}")

# ==================== 5.1 Перекидывание (Rust: блок 6 skipped) ====================
throwover = abs(alpha_56) > DEFLECTOR_DEG
if throwover:
    # Rust Blocks::blocks_pos для Hook при наличии skipped:
    #   x = prev.pos.x - 0.5 * prev.diameter,  y = prev.pos.y - aux_length
    blocks[hook_ix].coord.x = blocks[4].coord.x - 0.5 * blocks[4].D
    blocks[hook_ix].coord.y = blocks[4].coord.y - aux_length
    blocks[DEFLECTOR_IX].skipped = True

# Участки каната - только по НЕскипнутым блокам (как Rust RopeSections):
# после перекидывания участки (4,5) и (5,hook), без скипнутого блока 6
active = [b for b in blocks if not b.skipped]
rope_data = calc_rope_data(active)
rope_data_pairs = []
j = 1
for b in blocks:
    if not b.skipped:
        rope_data_pairs.append(j)
        j += 1
for rd, jj in zip(rope_data, rope_data_pairs):
    rd["block_pair"] = (jj, jj + 1)

# ==================== 6. Rust-структура Block / RopeSections / BlockArcs ====================
# Порядок блоков в result Rust (Blocks::eval):
#   [блок0..блок4, блок5, блок6(skipped), блок7(hook)]
# rope_len_fwd[i] - прямой участок от блока i к следующему НЕСКИПНУТОМУ (по RopeSections)
rope_len_fwd = [0.0] * 7
rope_alpha_fwd = [0.0] * 7
# Участки: пары по очереди в rope_data (5 участков при перекидывании: (1,2)..(5,hook))
pair_count = len(rope_data)
for i in range(pair_count):
    # RopeSections: блок i (не скипнутый) получает fwd = l_rope(i -> next)
    rope_len_fwd[i] = rope_data[i]["l_rope"]
    rope_alpha_fwd[i] = rope_data[i]["alpha_rope"]
# если блок 6 скипнут: rope_len_fwd[4] = участок блок5->крюк, rope_len_fwd[5]=0

rope_alpha_bck = [0.0] * 7
for i in range(1, 7):
    if blocks[i].skipped:
        continue
    rope_alpha_bck[i] = rope_alpha_fwd[i - 1]

# BlockArcs
wrap_alpha = [0.0] * 7
wrap_length = [0.0] * 7
for i in range(7):
    if blocks[i].skipped:
        wrap_alpha[i] = 0.0
        wrap_length[i] = 0.0
    elif RUST_BIND[i] == "Drum":
        wrap_alpha[i] = 0.0
        wrap_length[i] = seg_deg
    elif RUST_BIND[i] == "Hook":
        wrap_alpha[i] = 0.0
        wrap_length[i] = 0.0
    else:
        wa = abs(normalize_angle_deg(rope_alpha_fwd[i] - rope_alpha_bck[i]))
        wrap_alpha[i] = wa
        wrap_length[i] = math.pi * blocks[i].D * 0.5 * wa / 180.0

print("alpha_boom  :", [round(b.alpha, 6) for b in booms])
for i, blk in enumerate(blocks):
    print(f"block[{i}] ({RUST_BIND[i]}{' SKIPPED' if blk.skipped else ''}) pos=({blk.coord.x:.3f}, {blk.coord.y:.3f}) D={blk.D:.3f}")
for i, r in enumerate(rope_data):
    print(f"rope_data[{i}] pair={r['block_pair']} alpha={r['alpha_rope']:.6f} l_rope={r['l_rope']:.6f}   (по активным блокам)")
print("wrap_alpha  :", [round(w, 6) for w in wrap_alpha])
print("wrap_length :", [round(w, 6) for w in wrap_length])

# ==================== Bendings::new (winch_len) ====================
winch_len = rope_len_mm
for i in range(7):
    if blocks[i].skipped:
        continue
    if RUST_BIND[i] == "Drum":
        winch_len -= (0.0 + rope_len_fwd[i])
    else:
        winch_len -= (wrap_length[i] + rope_len_fwd[i])
print(f"\nwinch_len (Rust) = {winch_len:.6f} мм    (ожидание: t01 = 58330.0)")

# ==================== Bendings::eval ====================
def bendings_eval(pos_mm):
    wrap_delta = 0.0
    prev_bend_end = 0.0
    bending = [None] * 7
    for i in range(7):
        if blocks[i].skipped:
            continue
        if RUST_BIND[i] == "Drum":
            start = max(winch_len - wrap_length[i] - pos_mm + wrap_delta, 0.0)
        else:
            start = prev_bend_end
        end = start + wrap_length[i]
        prev_bend_end = end + rope_len_fwd[i]
        if abs(end - start) > 0.0:
            bending[i] = (start, end)
    return bending

# ==================== Deprecation::slices ====================
total_slices = int(math.floor(rope_len_mm / seg_deg))

def slices_of(bend):
    if bend is None:
        return []
    first = int(math.trunc(bend[0] / seg_deg))
    start_point = first * seg_deg
    delta = bend[1] - start_point
    n = int(math.ceil(delta / seg_deg))
    end = min(first + n, total_slices)
    return list(range(first, end))

print("\n========== Бенды при pos = 0 (парковка) ==========")
bend0 = bendings_eval(0.0)
for i, b in enumerate(bend0):
    if b is not None:
        print(f"  block[{i}] ({RUST_BIND[i]}, D={blocks[i].D}): [{b[0]:.6f} .. {b[1]:.6f}]  slices: {slices_of(b)}")

# ==================== прогон шагов теста ====================
test_data = [
    (4,  "Winch.Load", 1.000),
    (5,  "Winch.Pos", 0.020),
    (6,  "Winch.Pos", 0.030),
    (6,  "Winch.Pos", 0.060),
    (6,  "Winch.Pos", 0.080),
    (7,  "Winch.Pos", 0.100),
    (8,  "Winch.Pos", 0.120),
    (8,  "Winch.Pos", 0.140),
    (8,  "Winch.Pos", 0.160),
    (8,  "Winch.Pos", 0.180),
    (9,  "Winch.Pos", 0.200),
    (10, "Winch.Pos", 0.220),
    (10, "Winch.Pos", 0.240),
    (10, "Winch.Pos", 0.260),
    (10, "Winch.Pos", 0.280),
    (11, "Winch.Pos", 0.300),
    (12, "Winch.Pos", 0.320),
    (12, "Winch.Pos", 0.340),
    (12, "Winch.Pos", 0.360),
    (12, "Winch.Pos", 0.380),
    (13, "Winch.Pos", 0.400),
    (14, "Winch.Pos", 0.420),
    (14, "Winch.Pos", 0.440),
    (14, "Winch.Pos", 0.460),
    (14, "Winch.Pos", 0.480),
    (15, "Winch.Pos", 0.500),
    (16, "Winch.Pos", 0.520),
    (16, "Winch.Pos", 0.540),
    (16, "Winch.Pos", 0.560),
    (16, "Winch.Pos", 0.580),
    (17, "Winch.Pos", 0.600),
    (18, "Winch.Pos", 0.620),
    (18, "Winch.Pos", 0.640),
    (18, "Winch.Pos", 0.660),
    (18, "Winch.Pos", 0.680),
    (19, "Winch.Pos", 0.700),
]

result = [0.0] * total_slices
prev = [set() for _ in range(7)]
is_first_time = True
total_count = 0
pos_mm = 0.0
load = 0.0

step_touched = []  # список начислений на текущем шаге: (slice, dep, kind)
def eval_deprecation():
    global is_first_time, total_count, step_touched
    step_touched = []
    bend = bendings_eval(pos_mm)
    for i in range(7):
        if blocks[i].D <= 0.0:
            continue
        dep = load / (blocks[i].D * 0.001)
        cur = [] if blocks[i].skipped else slices_of(bend[i])
        for s in prev[i]:
            if s not in cur and not is_first_time:
                result[s] += dep
                total_count += 1
                step_touched.append((s, dep, "exit"))
        for s in cur:
            if s not in prev[i] and not is_first_time:
                result[s] += dep
                total_count += 1
                step_touched.append((s, dep, "enter"))
        prev[i] = set(cur)
    is_first_time = False

# Первый вызов (инициализация): углы парковки, pos=0, load=0 - износ не начисляется
eval_deprecation()

print("\n========== Шаги теста ==========")
print("step | event        | value  | count | первые 3 слайса (0..2) | затронутые слайсы (slice=значение)")
for step, event, value in test_data:
    if event == "Winch.Pos":
        pos_mm = value * 1000.0
    elif event == "Winch.Load":
        load = value
    eval_deprecation()
    delta = len(step_touched)
    touches = " ".join(f"{s}({k[0]}:{d:.3f})" for s, d, k in step_touched)
    print(f"{step:>4} | {event:<12} | {value:>6.3f} | {total_count:>5} | {delta:>2} | {touches}")

print("\nИтоговый count:", total_count)
