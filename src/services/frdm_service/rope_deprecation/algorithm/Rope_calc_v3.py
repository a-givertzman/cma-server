import matplotlib.pyplot as plt
import math
import logging 
import matplotlib.patches as patches
from dataclasses import dataclass

plt.set_loglevel(level="info")
# logging.getLogger('PIL.PngImagePlugin').setLevel(level="info")
logging.basicConfig(level=logging.DEBUG, force=True)

@dataclass
class BlockBindFixed:
    """Блок вне стрелы, барабан"""
    pass
@dataclass
class BlockBindBoom:
    """Блок на стреле"""
    boom: int
    def __init__(self, boom: int):
        self.boom = boom
@dataclass
class BlockBindHook:
    """Блок на подвеске"""
    pass
BlockBind = BlockBindFixed | BlockBindBoom | BlockBindHook

class Offset:
    x: float
    y: float
    def __init__(self, x: float, y: float):
            self.x = x
            self.y = y
    def __str__(self):
        return f'{self.x, self.y}'

class Boom:
    # Углы наклона стрел (относительно предыдыдущей) в градусах
    alpha_rel: float
    # Углы наклона стрел (относительно ГСК) в градусах
    alpha: float
    len: float
    l1: float
    l2: float
    l3: float
    l4: float
    D: Offset
    G: Offset
    def __init__(self, alpha_rel: float, len: float, l1: float, l2: float, l3: float, l4: float):
        """
        :alpha: Относительный угол наклона стрел (относительно предыдыдущей) в градусах
        :len: Длины стрел, мм
        :l1: Вертикальное смещение точки D, мм
        :l2: Горизонтальное смещение точки D, мм
        :l3: Вертикальное смещение начала стрелы относительно..., мм
        :l4: Горизонтальное смещение начала стрелы относительно..., мм
        """
        self.alpha_rel = alpha_rel
        self.alpha = 0.0
        self.len = len
        self.l1 = l1
        self.l2 = l2
        self.l3 = l3
        self.l4 = l4

class Block:
    lF: Offset
    D: float
    scheme: int
    bind: BlockBind
    coord: Offset
    def __init__(self, lF: Offset, D: float, scheme: int, bind: BlockBind):
        """
        :lF: Растояние от **конца** стрелы до оси блока, мм
        :D: Диаметры блоков, мм
        :schemes: Схема схода каната на блоке
        :boom: К какой стреле относится блок (нумерация с 0)
        """
        self.lF = lF
        self.D = D
        self.scheme = scheme
        self.bind = bind
        self.coord = Offset(0.0, 0.0)

# ---------------------------
# Вспомогательные функции
# ---------------------------

def XY_rotate(lx, ly, alpha):
    angle_rad = math.radians(alpha)
    x = lx * math.cos(angle_rad) - ly * math.sin(angle_rad)
    y = lx * math.sin(angle_rad) + ly * math.cos(angle_rad)
    return x, y

def alpha_horiz(Y1, Y2, X1, X2):
    """Угол наклона прямой к горизонту (в градусах)"""
    # Длина отрезка
    length = math.sqrt((X2 - X1)**2 + (Y2 - Y1)**2)
    if length == 0:
        return 0
    a = math.degrees(math.asin((Y1 - Y2) / length))
    if X1 <= X2:
        return a
    else:
        return 180 - a

def l_section(Y1, Y2, X1, X2):
    return math.sqrt((X2 - X1)**2 + (Y2 - Y1)**2)

def distance_point_to_line(Y1, Y2, X1, X2, x, y):
    """Расстояние от точки (x, y) до прямой, проходящей через (X1, Y1) и (X2, Y2)"""
    numerator = abs((Y2 - Y1) * x - (Y2 - Y1) * X1 - (X2 - X1) * y + (X2 - X1) * Y1)
    denominator = math.sqrt((Y2 - Y1)**2 + (X2 - X1)**2)
    if denominator == 0:
        return 0
    return numerator / denominator

def rope_parameters(X1, Y1, X2, Y2, D1, D2, k, j):
    l_block = l_section(Y1, Y2, X1, X2)
    alpha_block = alpha_horiz(Y1, Y2, X1, X2)
    alpha_rope = alpha_block + j * math.degrees(math.asin(0.5 * (D1 + k * D2) / l_block))
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
        "l_rope": l_rope
    }

# ------------------------------------------------
# Алгоритм расчета входа и исхода каната с блоков
# ------------------------------------------------
if __name__ == "__main__":
    # ---------------------------
    # Исходные данные
    # ---------------------------

    #
    # Стрелы
    booms = [
        Boom(alpha_rel= 69.71, len=11200.0, l1=0.0, l2=0.0, l3=0.0, l4=10330.0),
        Boom(alpha_rel= 155.3, len= 7984.0, l1=0.0, l2=0.0, l3=0.0, l4=    0.0),
    ]
    #
    # Блоки
    blocks = [
        Block(lF=Offset( 1830.0,  710.0), D=844.0, scheme=1, bind=BlockBindFixed()),
        Block(lF=Offset(  308.0, 1100.0), D=816.0, scheme=1, bind=BlockBindBoom(0)),
        Block(lF=Offset(-6550.0, 1730.0), D=816.0, scheme=1, bind=BlockBindBoom(1)),
        Block(lF=Offset(-1121.0,  973.0), D=816.0, scheme=2, bind=BlockBindBoom(1)),
        Block(lF=Offset(  267.0,  860.0), D=816.0, scheme=3, bind=BlockBindBoom(1)),
        Block(lF=Offset(  136.0,  -35.0), D=816.0, scheme=1, bind=BlockBindBoom(1)),
        Block(lF=Offset(    0.0,    0.0), D=  0.0, scheme=0, bind=BlockBindHook()),
    ]

    block_bind = [
        BlockBindFixed(),   # Блок 1
        BlockBindBoom(1),   # Блок 2
        BlockBindBoom(2),   # Блок 3
        BlockBindBoom(2),   # Блок 4
        BlockBindBoom(2),   # Блок 5
        BlockBindBoom(2),   # Блок 6
        BlockBindHook(),    # Блок 7
    ]

    
    # Дополнительные параметры крюковой подвески
    hook_params = {
        "hook_block_num": 7,  # номер блока подвеса в списке
        "l_hook": 1000        # длина подвеса (мм)
    }

    # Параметры для расчета длины каната
    rope_calc_params = {
        "D_pitch": 845,  # делительный диаметр барабана (мм)
        "a_n": 1,        # количество канатов на барабане
        "Lstock": 0,     # дополнительный запас каната (мм)
        "Lfact": 88000      # фактическая длина каната (мм)
    }

    # ---------------------------
    # 2. Угол наклона к горизонту каждой стрелы (alpha_boom)
    # ---------------------------
    alpha_sum = 0.0
    for i, boom in enumerate(booms):
        alpha_sum += boom.alpha_rel
        # log.debug(f"i: {i},  alpha sum_ {alpha_sum}")
        boom.alpha = alpha_sum - i * 180

    # ---------------------------
    # 3. Матрица T (D и G для каждой стрелы)
    # ---------------------------
    for i, boom in enumerate(booms):
        # Начало стрелы
        if i == 0:
            x0, y0 = 0, 0
            alpha_prime = 90
        else:
            x0, y0 = booms[i - 1].G.x, booms[i - 1].G.y
            alpha_prime = booms[i - 1].alpha

        wx, wy = XY_rotate(boom.l4, boom.l3, alpha_prime)
        XY_start = Offset(x0 + wx, y0 + wy)
        # log.debug(f"Стрела {i}: XY_start={XY_start}")

        # Точка D
        Dx, Dy = XY_rotate(- boom.l2, boom.l1, boom.alpha)
        D_point = Offset(XY_start.x + Dx, XY_start.y + Dy)
        # log.debug(f"\t D_point={D_point}")

        # Точка G
        Gx, Gy = XY_rotate(boom.len - boom.l2, boom.l1, boom.alpha)
        G_point = Offset(XY_start.x + Gx, XY_start.y + Gy)
        # log.debug(f"\t G_point={G_point}")

        boom.D = D_point
        boom.G = G_point
    logging.debug(f"booms {booms}")

    # ---------------------------
    # 4. Координаты блоков XY_block
    # ---------------------------
    for idx, block in enumerate(blocks):
        # bind = block_bind[idx]
        match block.bind:
            case BlockBindFixed():
                # Формула из алгоритма:
                dx1, dy1 = XY_rotate(-block.lF.x, block.lF.y, 0)
                dx2, dy2 = XY_rotate(booms[0].l4, booms[0].l3, 90)  # от первой стрелы
                x = dx1 + dx2
                y = dy1 + dy2
                block.coord.x = x
                block.coord.y = y
            case BlockBindBoom(boom_index):
                # Определяем номер стрелы
                # boom_num = int(feature.split()[0]) - 1
                boom = booms[boom_index]
                # logging.debug(f"Стрела {idx}")
                base_point = boom.G  # точка G
                dx, dy = XY_rotate(block.lF.x, block.lF.y, boom.alpha)
                block.coord.x = base_point.x + dx
                block.coord.y = base_point.y + dy
            case BlockBindHook():
                block.coord.x = float('nan')
                block.coord.y = float('nan')
            case _:
                raise ValueError(f"Неизвестный тип блока [{idx}]: {bind}")
                

    # ---------------------------
    # 5. Расчёт координат крюковой подвески
    # ---------------------------
    hook_block_num = hook_params["hook_block_num"]
    l_hook = hook_params["l_hook"]
    
    prev_idx = hook_block_num - 2
    x_prev, y_prev = blocks[prev_idx].coord.x, blocks[prev_idx].coord.y
    D_prev = blocks[prev_idx].D
    
    blocks[hook_block_num - 1].coord.x = x_prev + 0.5 * D_prev
    blocks[hook_block_num - 1].coord.y = y_prev - l_hook

    logging.debug(f"Крюковая подвеска: ({blocks[hook_block_num - 1].coord.x}, "
                  f"{blocks[hook_block_num - 1].coord.y})")



    # ---------------------------
    # 6. Расчёт параметров каната
    # ---------------------------
    rope_data = []
    for i, block in enumerate(blocks[:-1]):
        X1, Y1 = block.coord.x, block.coord.y
        X2, Y2 = blocks[i + 1].coord.x, blocks[i + 1].coord.y
        D1 = block.D
        D2 = blocks[i + 1].D
        scheme = block.scheme

        if scheme == 1: k, j = -1, 1
        elif scheme == 2: k, j = 1, 1
        elif scheme == 3: k, j = 1, -1
        elif scheme == 4: k, j = -1, -1
        else: raise ValueError(f"Некорректная схема: {scheme}")

        params = rope_parameters(X1, Y1, X2, Y2, D1, D2, k, j)
        params["block_pair"] = (i + 1, i + 2)
        params["scheme"] = scheme
        rope_data.append(params)
   
    logging.debug("Параметры каната между блоками (rope_data):")
    for r in rope_data:
       logging.debug(f" Пары {r['block_pair']}: l_block={r['l_block']:.2f}, alpha_rope={r['alpha_rope']:.2f}, l_rope={r['l_rope']:.2f}")

    for r in rope_data:
        logging.debug(
            f'Вход X {r["X1_block"]:.2f}, Выход X {r["X2_block"]:.2f} | '
            f'Вход Y {r["Y1_block"]:.2f}, Выход Y {r["Y2_block"]:.2f}'
        )

    # -----------------------------
    # 7. Углы обхвата и длины дуг каждого блока
    # -----------------------------
    def calc_block_angles_and_arcs(rope_data):
        wrap_angles = []
        arc_lengths = []
        L_sys_arc = 0
    
        prev_alpha = None  # предыдущий угол для формирования alpha_rope_list
    
        for block, r in zip(blocks[:-1], rope_data):
            alpha_rope = r.get("alpha_rope", 0)

            # Формируем alpha_rope_list
            if prev_alpha is not None:
                alpha_rope_list = [prev_alpha, alpha_rope]
            else:
                alpha_rope_list = [alpha_rope]  # для первого блока
            r["alpha_rope_list"] = alpha_rope_list
        
            # Расчёт угла обхвата
            if len(alpha_rope_list) > 1:
                alpha_wrap = abs(alpha_rope_list[-1] - alpha_rope_list[0])
            else:
                alpha_wrap = 0
            
            R = block.D / 2
            L_arc = (math.pi * R * alpha_wrap) / 180
            
            L_sys_arc += L_arc
            wrap_angles.append(alpha_wrap)
            arc_lengths.append(L_arc)

            prev_alpha = alpha_rope  # обновляем предыдущий угол
    
        return {
            "wrap_angles": wrap_angles,
            "arc_lengths": arc_lengths,
            "L_sys_arc": L_sys_arc
        }

    # -----------------------------
    # 8. Общая длина каната и остаток на барабане
    # -----------------------------
    def calc_rope_sums(rope_data, D_pitch, a_n, Lstock, Lfact):
        l_section_summ = sum(r["l_rope"] for r in rope_data)
        L_stock_total = 3 * math.pi * D_pitch * a_n + Lstock
        block_results = calc_block_angles_and_arcs(rope_data)
        L_rope_sum = l_section_summ + block_results["L_sys_arc"] + L_stock_total
        L_winch = Lfact - l_section_summ - block_results["L_sys_arc"]

    
        return {
            "l_section_summ": l_section_summ,
            "L_stock_total": L_stock_total,
            "L_rope_sum": L_rope_sum,
            "L_winch": L_winch,
            "block_results": block_results
        }
        
# ---------------------------
# Логи
# ---------------------------
logging.debug(f"Число стрел: {len(booms)}")
logging.debug(f"alpha_boom: {[round(boom.alpha, 3) for boom in booms]}")
for idx, boom in enumerate(booms, start=1):
    logging.debug(f"Стрела {idx}: D={boom.D}, G={boom.G}")
for i, block in enumerate(blocks, start=1):
    logging.debug(f"Блок {i}: {block.coord}")
for r in rope_data:
    logging.debug(f"Блоки {r['block_pair']} | Схема {r['scheme']} | "
                  f"L_block={r['l_block']:.2f} | Alpha_rope={r['alpha_rope']:.2f}° | "
                  f"L_rope={r['l_rope']:.2f}")

block_results = calc_block_angles_and_arcs(rope_data)
logging.debug("Углы обхвата и длины дуг (по каждой паре блоков):")

for i, (alpha_wrap, arc_length) in enumerate(
        zip(block_results["wrap_angles"], block_results["arc_lengths"]), start=1):
    bp = f"{rope_data[i-1]['block_pair'][0]}-{rope_data[i-1]['block_pair'][1]}"
    logging.debug(
        f"Блок {bp}: alpha_wrap={alpha_wrap:.2f} deg, arc_length={arc_length:.2f} mm"
    )

rope_results = calc_rope_sums(
    rope_data,
    D_pitch=rope_calc_params["D_pitch"],
    a_n=rope_calc_params["a_n"],
    Lstock=rope_calc_params["Lstock"],
    Lfact=rope_calc_params["Lfact"]
)



logging.debug("---- Итоги расчёта каната ----")
logging.debug(f"Сумма прямых участков l_section_summ = {rope_results['l_section_summ']:.2f} мм")
logging.debug(f"Сумма дуг L_sys_arc                 = {rope_results['block_results']['L_sys_arc']:.2f} мм")
logging.debug(f"Запас на барабане L_stock_total     = {rope_results['L_stock_total']:.2f} мм")
logging.debug(f"Общая требуемая длина L_rope_sum    = {rope_results['L_rope_sum']:.2f} мм")
logging.debug(f"Остаток на барабане L_winch        = {rope_results['L_winch']:.2f} мм")


# ---------------------------
# Построение графика
# ---------------------------
plt.figure(figsize=(10, 8))
plt.title("Схема расположения стрел и блоков")
plt.xlabel("X координата (мм)")
plt.ylabel("Y координата (мм)")
plt.grid(True)
plt.axis('equal')

# Стрелы
colors = ['green', 'blue']
for i, boom in enumerate(booms):
    plt.plot([boom.D.x, boom.G.x], [boom.D.y, boom.G.y], color=colors[i], linewidth=1.5)
    plt.scatter([boom.D.x, boom.G.x], [boom.D.y, boom.G.y], color=colors[i], s=20, marker='s')

# Блоки
for i, block in enumerate(blocks):
    x, y = block.coord.x, block.coord.y
    if math.isnan(x) or math.isnan(y):
        continue
    radius = block.D / 2
    circle = patches.Circle((x, y), radius, fill=False, color='deepskyblue', linewidth=1)
    plt.gca().add_patch(circle)
    plt.text(x + radius, y + radius, f'Блок {i+1}', fontsize=8, color='black')

# Канаты
for r in rope_data:
    plt.plot([r["X1_block"], r["X2_block"]], [r["Y1_block"], r["Y2_block"]],
            color='blue', linestyle='--')
    plt.scatter([r["X1_block"], r["X2_block"]], [r["Y1_block"], r["Y2_block"]],
                color='orange', s=15)

plt.show()