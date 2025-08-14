import matplotlib.pyplot as plt
import math
import logging 
import matplotlib.patches as patches
from dataclasses import dataclass

plt.set_loglevel(level="info")
# logging.getLogger('PIL.PngImagePlugin').setLevel(level="info")
logging.basicConfig(level = logging.DEBUG, force = True)

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
        :bind: К какой стреле относится блок (нумерация с 0)
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
        Boom(alpha_rel= 74.0, len=11200.0, l1=0.0, l2=0.0, l3=0.0, l4=10330.0),
        Boom(alpha_rel=128.0, len= 7984.0, l1=0.0, l2=0.0, l3=0.0, l4=    0.0),
    ]
    #
    # Блоки
    blocks = [
        Block(lF=Offset( 1830.0,  710.0), D=844.0, scheme=1, bind=BlockBindFixed()),
        Block(lF=Offset(  308.0, 1090.0), D=816.2, scheme=1, bind=BlockBindBoom(0)),
        Block(lF=Offset(-6550.0, 1743.0), D=816.2, scheme=1, bind=BlockBindBoom(1)),
        Block(lF=Offset(-1120.0, 1005.0), D=816.2, scheme=2, bind=BlockBindBoom(1)),
        Block(lF=Offset(  268.0,  895.0), D=816.2, scheme=3, bind=BlockBindBoom(1)),
        Block(lF=Offset(  140.0,    0.0), D=816.2, scheme=1, bind=BlockBindBoom(1)),
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
                logging.debug(f"Стрела {idx}")
                base_point = boom.G  # точка G
                dx, dy = XY_rotate(block.lF.x, block.lF.y, boom.alpha)
                block.coord.x = base_point.x + dx
                block.coord.y = base_point.y + dy
            case BlockBindHook():
                block.coord.x = float('nan')
                block.coord.y = float('nan')
            # case _:
            #     raise ValueError(f"Неизвестный тип блока [{idx}]: {bind}")

    # ---------------------------
    # 5. Расчёт параметров каната
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

    plt.legend()
    plt.show()