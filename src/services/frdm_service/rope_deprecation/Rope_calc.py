import matplotlib.pyplot as plt
import math
import logging as log
import matplotlib.patches as patches

log.basicConfig(level = log.DEBUG, force = True)

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
    boom: int
    coord: Offset
    def __init__(self, lF: Offset, D: float, scheme: int, boom: int):
        """
        :lF: Растояние от корня стрелы до оси блока, mm
        :D: Диаметры блоков, мм
        :schemes: Схема схода каната на блоке
        :boom: К какой стреле относится блок (нумерация с 0)
        """
        self.lF = lF
        self.D = D
        self.scheme = scheme
        self.boom = boom
        self.coord = Offset(0.0, 0.0)

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
    # alpha = [0, 23.783]            # Углы в градусах как в расчете у Вани
    # alpha = [74, 128]               # Углы наклона стрел (относительно предыдыдущей) в градусах
    # L_boom = [11200, 7984]          # Длины стрел (мм)
    # l1 = [0, 0]                     # вертикальное смещение точки D
    # l2 = [0, 0]                     # горизонтальное смещение точки D
    # l3 = [0, 0]                     # вертикальное смещение начала стрелы
    # l4 = [10330, 0]                 # горизонтальное смещение начала стрелы
    #
    # Блоков
    blocks = [
        #     lF                          D        scheme    boom    
        Block(lF=Offset(308.0, 1090.0),   D=816.2, scheme=1, boom=0),
        Block(lF=Offset(1433.0, 1743.0),  D=816.2, scheme=1, boom=1),
        Block(lF=Offset(-1120.0, 1005.0), D=816.2, scheme=2, boom=1),
        Block(lF=Offset(268.0, 895.0),    D=816.2, scheme=3, boom=1),
        Block(lF=Offset(140.0, 0.0),      D=816.2, scheme=3, boom=1),
    ]
    # D = [816.2, 816.2, 816.2, 816.2, 816.2] # диаметры блоков, мм ???
    # schemes = [1, 1, 2, 3, 3]       # схема схода каната на блоке 

    # Размеры расположения блоков на стрелах, mm
    # lF = [Offset(308.0, 1090.0), Offset(1433.0, 1743.0), Offset(-1120.0, 1005.0), Offset(268.0, 895.0), Offset(140.0, 0.0)]
    # lFx = [308, 1433, -1120, 268, 140]  # мм 
    # lFy = [1090, 1743, 1005, 895, 0]   # мм

    #lFx = [308, 1435, -1121, 267, 136]  # мм как в расчете у Вани
    #lFy = [1100, 1730, 973, 860, -35]   # мм как в расчете у Вани
    # boom_index = [0, 1, 1, 1, 1]        # к какой стреле относится блок (нумерация с 1)

    # ---------------------------
    # 2. Угол наклона к горизонту каждой стрелы (alpha_boom)
    # ---------------------------
    alpha_sum = 0.0
    for i, boom in enumerate(booms):
        alpha_sum += boom.alpha_rel
        # log.debug(f"i: {i},  alpha sum_ {alpha_sum}")
        boom.alpha = alpha_sum - i * 180

    # ---------------------------
    # 3. D и G для каждой стрелы
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



    # ---------------------------
    # 4. Координаты блоков x, y относительно ГСК, мм
    # ---------------------------
    for i, block in enumerate(blocks):
        base_point = booms[block.boom].D
        dx, dy = XY_rotate(block.lF.x, block.lF.y, booms[block.boom].alpha)
        block.coord.x = base_point.x + dx
        block.coord.y = base_point.y + dy

    # ---------------------------
    # 5. Расчёт параметров каната
    # ---------------------------
    rope_data = []
    # Перебираем элементы без последнего
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
        params["block_pair"] = (i, i + 1)
        params["scheme"] = scheme
        rope_data.append(params)

    # ---------------------------
    # Логи
    # ---------------------------
    log.debug(f"Число стрел: {len(booms)}")
    # log.debug(f"alpha_boom: {[round(a, 3) for a in alpha_boom]}")
    for idx, boom in enumerate(booms, start=1):
        log.debug(f"Стрела {idx}: D={boom.D}, G={boom.G}")

    for idx, boom in enumerate(booms, start=1):
        log.debug(f"Стрела {idx}: alpha={boom.alpha}")

    for i, block in enumerate(blocks, start=1):
        log.debug(f"Блок {i}: {block.coord}")
    for r in rope_data:
        log.debug(f"Блоки {r['block_pair']} | Схема {r['scheme']} | "
                    f"L_block={r['l_block']:.2f} | Alpha_rope={r['alpha_rope']:.2f}° | "
                    f"L_rope={r['l_rope']:.2f}")

    # ---------------------------
    # Тесты
    # ---------------------------

    # Абсолютные углы стрел
    target_boom_alpha = [74, 22]
    for i, boom in enumerate(booms):
        target = target_boom_alpha[i]
        assert boom.alpha == target, f'Boom[{i}].alpha \n\t result: {boom.alpha} \n\t target: {target}'

    # Начало и конец стрел
    target_T = [
        {'D': Offset(6.32530071759608e-13, 10330.0),        'G': Offset(3087.138385150391, 21096.13099450917)},
        {'D': Offset(3087.138385150391, 21096.13099450917), 'G': Offset(10489.774280011621, 24086.990036341813)},
    ]
    for i, boom in enumerate(booms):
        target_D = target_T[i]['D']
        target_G = target_T[i]['G']
        assert boom.D.x == target_D.x, f'T[{i}].D.x \n\t result: {boom.D.x} \n\t target: {target_D.x}'
        assert boom.D.y == target_D.y, f'T[{i}].D.y \n\t result: {boom.D.y} \n\t target: {target_D.y}'
        assert boom.G.x == target_G.x, f'T[{i}].G.x \n\t result: {boom.G.x} \n\t target: {target_G.x}'
        assert boom.G.y == target_G.y, f'T[{i}].G.y \n\t result: {boom.G.y} \n\t target: {target_G.y}'

    # Блоки
    target_coords = [
        (2124.2594421692593, 21692.6443146987),
        (3762.8535564206627, 23249.02370138408),
        (9074.848736513828, 24599.250425555612),
        (10402.986651928279, 25017.21415321455),
        (10619.580019650972, 24139.43495942004),        
    ]
    for i, block in enumerate(blocks):
        target_x = target_coords[i][0]
        target_y = target_coords[i][1]
        assert block.coord.x == target_x, f'Coord[{i}].x \n\t result: {block.coord.x} \n\t target: {target_x}'
        assert block.coord.y == target_y, f'Coord[{i}].y \n\t result: {block.coord.y} \n\t target: {target_y}'

    # # ---------------------------
    # # Построение графика
    # # ---------------------------
    # plt.figure(figsize=(10, 8))
    # plt.title("Схема расположения стрел и блоков")
    # plt.xlabel("X координата (мм)")
    # plt.ylabel("Y координата (мм)")
    # plt.grid(True)
    # plt.axis('equal')

    # # Стрелы
    # colors = ['green', 'blue']
    # for i, (D_pt, G_pt) in enumerate(T):
    #     plt.plot([D_pt[0], G_pt[0]], [D_pt[1], G_pt[1]], color=colors[i], linewidth=1.5, label=f'Стрела {i+1}')
    #     plt.scatter([D_pt[0], G_pt[0]], [D_pt[1], G_pt[1]], color=colors[i], s=20, marker='s')

    # # Блоки с диаметрами
    # for i, (x, y) in enumerate(XY_block):
    #     radius = D[i] / 2  # из мм, D — это диаметр
    #     circle = patches.Circle((x, y), radius, fill=False, color='deepskyblue', linewidth=1)
    #     plt.gca().add_patch(circle)
    #     plt.text(x + radius, y + radius, f'Блок {i+1}', fontsize=8, color='black')

    # # Канаты между точками схода
    # for r in rope_data:
    #     plt.plot([r["X1_block"], r["X2_block"]], [r["Y1_block"], r["Y2_block"]],
    #              color='blue', linestyle='--')
    #     plt.scatter([r["X1_block"], r["X2_block"]], [r["Y1_block"], r["Y2_block"]],
    #                 color='orange', s=15)

    # plt.legend()
    # plt.show()
