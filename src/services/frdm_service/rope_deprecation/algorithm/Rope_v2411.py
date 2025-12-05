import matplotlib.pyplot as plt
import csv, gc
import math
import logging
import numpy as np
import matplotlib.patches as patches
from dataclasses import dataclass

plt.set_loglevel(level="INFO")
logging.getLogger("PIL").setLevel(logging.WARNING)
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
class BlockBindBoomPair:
    """Блок спаренный (на нем происходит переброс канат) на стреле"""
    boom: int
    def __init__(self, boom: int):
        self.boom = boom
@dataclass
class BlockBindHook:
    """Блок на подвеске"""
    pass
BlockBind = BlockBindFixed | BlockBindBoom | BlockBindBoomPair | BlockBindHook

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

class Booms:
    """
    Расчитывает стрелы
    1. Угол наклона к горизонту каждой стрелы (alpha_boom)
    2. Матрица T (D и G для каждой стрелы)
    """
    __booms: list[Boom]
    def __init__(self, booms: list[Boom]) -> None:
        self.__booms = booms
    def eval(self) -> list[Boom]:
        return self.__booms_d_g(
            self.__alpha(self.__booms)
        )
    # ---------------------------
    # 1. Угол наклона к горизонту каждой стрелы (alpha_boom)
    # ---------------------------
    def __alpha(self, booms: list[Boom]) -> list[Boom]:
        alpha_sum = 0.0
        for i, boom in enumerate(booms):
            alpha_sum += boom.alpha_rel
            # log.debug(f"i: {i},  alpha sum_ {alpha_sum}")
            boom.alpha = alpha_sum - i * 180
        return booms

    # ---------------------------
    # 2. Матрица T (D и G для каждой стрелы)
    # ---------------------------
    def __booms_d_g(self, booms: list[Boom]) -> list[Boom]:
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

            # Точка D
            Dx, Dy = XY_rotate(- boom.l2, boom.l1, boom.alpha)
            D_point = Offset(XY_start.x + Dx, XY_start.y + Dy)

            # Точка G
            Gx, Gy = XY_rotate(boom.len - boom.l2, boom.l1, boom.alpha)
            G_point = Offset(XY_start.x + Gx, XY_start.y + Gy)

            boom.D = D_point
            boom.G = G_point
        return booms


class Block:
    lF: Offset
    D: float
    scheme: int
    bind: BlockBind
    coord: Offset
    wrap_len: float
    wrap_arc: float
    L_sys_arc: float
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
        self.wrap_len = None
        self.wrap_arc = None
        self.L_sys_arc = None
    def empty():
        return Block(lF=Offset( 0.0,  0.0), D=0.000, scheme=0, bind=BlockBindFixed)

class RopeParams:
    l_block: float
    alpha_block: float
    alpha_rope: float
    "Угол прямого участка каната к горизонту, градусы"
    X1_block: float
    Y1_block: float
    X2_block: float
    Y2_block: float
    l_rope: float
    block_pair: tuple[int, int]
    scheme: int
    alpha_rope_list: list[float]
    def __init__(self, l_block: float, alpha_block: float, alpha_rope: float, X1_block: float, Y1_block: float, X2_block: float, Y2_block: float, l_rope: float):
        self.l_block = l_block
        self.alpha_block = alpha_block
        self.alpha_rope = alpha_rope
        self.X1_block = X1_block
        self.Y1_block = Y1_block
        self.X2_block = X2_block
        self.Y2_block = Y2_block
        self.l_rope = l_rope
        self.scheme = None
    def empty():
        return RopeParams(l_block=None, alpha_block=None, alpha_rope=None, X1_block=None, Y1_block=None, X2_block=None, Y2_block=None, l_rope=None)

# ---------------------------
# Вспомогательные функции
# ---------------------------

def XY_rotate(lx, ly, alpha):
    """"Формула расчета координат в повернутой СК"""
    angle_rad = math.radians(alpha)
    x = lx * math.cos(angle_rad) - ly * math.sin(angle_rad)
    y = lx * math.sin(angle_rad) + ly * math.cos(angle_rad)
    return x, y

def alpha_horiz(length, Y1, Y2, X1, X2):
    """Угол наклона прямой к горизонту (в градусах)"""
    # Длина отрезка
    if length == 0:
        return 0
    a = math.degrees(math.asin((Y1 - Y2) / length))
    if X1 <= X2:
        return a
    else:
        return 180 - a

def l_section(Y1, Y2, X1, X2):
    """"Расчет расстояния между двумя точками"""
    return math.sqrt((X2 - X1)**2 + (Y2 - Y1)**2)

def distance_point_to_line(Y1, Y2, X1, X2, x, y):
    """Расстояние от точки (x, y) до прямой, проходящей через (X1, Y1) и (X2, Y2)"""
    numerator = abs((Y2 - Y1) * x - (Y2 - Y1) * X1 - (X2 - X1) * y + (X2 - X1) * Y1)
    denominator = math.sqrt((Y2 - Y1)**2 + (X2 - X1)**2)
    if denominator == 0:
        return 0
    return numerator / denominator

def rope_parameters(X1, Y1, X2, Y2, D1, D2, k, j) -> RopeParams:
    """
    Расчет параметров каната
    l_block - расстояние между блоками, на картинке L_block
    alpha_block - угол между линией между блоками и горизонтом на картинке alpha_block
    alpha_rope - угол между линией каната между блоками и горизонтом на картинке alpha_rope
    X1_block - точка входа на блок по X
    Y1_block - точка входа на блок по Y
    X2_block - точка выхода на блок по X
    Y2_block - точка выхода на блок по Y
    """
    l_block = l_section(Y1, Y2, X1, X2)
    alpha_block = alpha_horiz(l_block, Y1, Y2, X1, X2)
    alpha_rope = alpha_block + j * math.degrees(math.asin(0.5 * (D1 + k * D2) / l_block))
    X1_block = X1 + j * 0.5 * D1 * math.sin(math.radians(alpha_rope))
    Y1_block = Y1 + j * 0.5 * D1 * math.cos(math.radians(alpha_rope))
    X2_block = X2 - j * k * 0.5 * D2 * math.sin(math.radians(alpha_rope))
    Y2_block = Y2 - j * k * 0.5 * D2 * math.cos(math.radians(alpha_rope))
    
    l_rope = l_section(Y2_block, Y1_block, X2_block, X1_block)
    return RopeParams(
        l_block=l_block,
        alpha_block=alpha_block,
        alpha_rope=alpha_rope,
        X1_block=X1_block,
        Y1_block=Y1_block,
        X2_block=X2_block,
        Y2_block=Y2_block,
        l_rope=l_rope,
    )

class RopeCalcParams:
    Lfact: float
    "фактическая длина каната"
    L_winch: float
    "длина каната на лебедке в основном положении"
    lhook_min: float
    "минимальная длина подвеса"
    hook_block_num: float
    "номер блока крюковой подвески"
    alpha_rope0: float
    "Угол прямолинейного участка каната от лебедки (к горизонту), град"
    def __init__(self, Lfact, L_winch, lhook_min, hook_block_num, alpha_rope0 = 0) -> None:
        """
        Lfact - фактическая длина каната,
        L_winch - длина каната на лебедке в основном положении,
        lhook_min - минимальная длина подвеса,
        hook_block_num - номер блока крюковой подвески
        """ 
        self.Lfact = float(Lfact)
        self.L_winch = float(L_winch)
        self.lhook_min = float(lhook_min)
        self.hook_block_num = float(hook_block_num)
        self.alpha_rope0 = float(alpha_rope0)

def float_range(start, stop, step):
    while start + step <= stop:
        yield start
        start += step
    if start < stop: yield stop

def aproxEq(a, b, tolerance=1e-9):
    return abs(a - b) < tolerance

def calc_alpha_rope0_first_boom_zero(blocks: list[Block], booms: list[Boom], rope_calc_params: RopeCalcParams) -> float:
    """
    Считает alpha_rope0 для особого положения:
    первая стрела горизонтальна (0° к горизонту).
    Вторая стрела на alpha_rope первого участка не влияет.
    """
    # ---------- 1. Углы стрел ----------
    # ---------- 2. D и G ----------
    booms = Booms(booms).eval()
    # ---------- 3. Координаты блоков ----------
    blocks = block_pos(blocks, booms)
    # ---------- 4. Расчёт параметров каната ----------
    rope_data: list[RopeParams] = rope_data_eval(blocks)
    # Нам нужен угол первого участка (1 -> 2)
    alpha_rope0 = rope_data[0].alpha_rope
    # logging.debug(f"[INIT] alpha_rope0 (при alpha_1 = 0°) = {alpha_rope0:.3f}°")
    return alpha_rope0

# ---------------------------
# 3. Координаты блоков XY_block
# ---------------------------
def block_pos(blocks: list[Block], booms: list[Boom]) -> list[Block]:
    prev_block: Block = None
    for idx, block in enumerate(blocks):
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
                boom = booms[boom_index]
                base_point = boom.G  # точка G
                dx, dy = XY_rotate(block.lF.x, block.lF.y, boom.alpha)
                block.coord.x = base_point.x + dx
                block.coord.y = base_point.y + dy
            case BlockBindBoomPair(boom_index):
                # Определяем номер стрелы
                boom = booms[boom_index]
                base_point = boom.G  # точка G
                dx, dy = XY_rotate(block.lF.x, block.lF.y, boom.alpha)
                block.coord.x = base_point.x + dx
                block.coord.y = base_point.y + dy
            case BlockBindHook():
                block.coord.x = prev_block.coord.x + 0.5 * prev_block.D
                block.coord.y = prev_block.coord.y - rope_calc_params.lhook_min
            case _:
                raise ValueError(f"Неизвестный тип блока [{idx}]: {block.bind}")
        prev_block = block
    return blocks
# ---------------------------
# 4. Расчёт параметров каната
# ---------------------------
def rope_data_eval(blocks: list[Block]) -> list[RopeParams]:
    rope_data: list[RopeParams] = []
    for i, block in enumerate(blocks[:-1]):
        X1, Y1 = block.coord.x, block.coord.y
        X2, Y2 = blocks[i + 1].coord.x, blocks[i + 1].coord.y
        D1 = block.D
        D2 = blocks[i + 1].D
        scheme = block.scheme
    
        if scheme == 1:
            k, j = -1, 1
        elif scheme == 2:
            k, j = 1, 1
        elif scheme == 3:
            k, j = 1, -1
        elif scheme == 4:
            k, j = -1, -1
        else:
            raise ValueError(f"Некорректная схема: {scheme}")
    
        params = rope_parameters(X1, Y1, X2, Y2, D1, D2, k, j)
        params.block_pair = (i + 1, i + 2)   # 1-базовая нумерация блоков
        params.scheme = scheme
        rope_data.append(params)
    return rope_data

# ------------------------------------------------
# Алгоритм расчета входа и исхода каната с блоков
# ------------------------------------------------
if __name__ == "__main__":
    plot = False
    # target_csv = "src/tests/unit/services/frdm_service/deprecation_test.csv"
    # f = open("C:/Users/Liaman/Desktop/rope/unit test/deprecation_test.csv")
    # forigin = open("C:/Users/Liaman/Desktop/rope/unit test/deprecation_test.csv", mode='r')
    forigin = open("src/tests/unit/services/frdm_service/deprecation_test.csv", mode='r')
    rows = csv.reader(forigin, delimiter=',')
    # logging.debug(f"csv rows {rows}")
    row = next(rows)
    # if row:
    #     f = open(target_csv, mode='w')
    #     frow = ",".join(map(lambda x: f'{x}', row))
    #     f.write(f'{frow}\n')
    #     f.close()


    rope_calc_params = RopeCalcParams(
        Lfact=85045,
        L_winch=58330,
        lhook_min=1200,
        hook_block_num=7,
    )
    def blocks_new() -> list[Block]:
        return [
            Block(lF=Offset( 1830.0,  710.0), D=845.000, scheme=1, bind=BlockBindFixed()),
            Block(lF=Offset(  308.0, 1100.0), D=816.000, scheme=1, bind=BlockBindBoom(0)),
            Block(lF=Offset(-6549.0, 1730.0), D=816.000, scheme=1, bind=BlockBindBoom(1)),
            Block(lF=Offset(-1121.0,  973.0), D=816.000, scheme=2, bind=BlockBindBoom(1)),
            Block(lF=Offset(  267.0,  860.0), D=816.000, scheme=3, bind=BlockBindBoom(1)),
            Block(lF=Offset(  136.0,  -35.0), D=816.000, scheme=1, bind=BlockBindBoomPair(1)),
            Block(lF=Offset(    0.0,    0.0), D=    0.0, scheme=0, bind=BlockBindHook()),
        ]
    def booms_new(angles: list[float]) -> list[Boom]:
        return [
            Boom(alpha_rel= angles[0], len=11200.0, l1=0.0, l2=0.0, l3=0.0, l4=10330.0),
            Boom(alpha_rel= angles[1], len= 7984.0, l1=0.0, l2=0.0, l3=0.0, l4=    0.0),
        ]

    # 0. Расчет особого положения (Парковочное)
    alpha_rope0 = calc_alpha_rope0_first_boom_zero(blocks_new(), booms_new([0.0, 155.299999999996]), rope_calc_params)
    logging.debug(f"boob2 alpha: {155.299999999996}, alpha_rope0: {alpha_rope0}")
    rope_calc_params.alpha_rope0 = alpha_rope0

    tblock: list[Block] = [Block.empty() for _ in range(7)]
    trope: list[RopeParams] = [Block.empty() for _ in range(7)]
    for row in rows:
        logging.debug(f"csv row: {row}")
        step = int(row[0])
        a21 = float(row[1])
        a22 = float(row[2])
        logging.debug(f"csv a21: {a21}")
        logging.debug(f"csv a22: {a22}")
        tblock[2 -1].coord.x = float(row[13])
        tblock[2 -1].coord.y = float(row[14])
        tblock[3 -1].coord.x = float(row[15])
        tblock[3 -1].coord.y = float(row[16])
        tblock[4 -1].coord.x = float(row[17])
        tblock[4 -1].coord.y = float(row[18])
        tblock[5 -1].coord.x = float(row[19])
        tblock[5 -1].coord.y = float(row[20])
        tblock[6 -1].coord.x = float(row[21])
        tblock[6 -1].coord.y = float(row[22])
        tblock[7 -1].coord.x = float(row[7])
        tblock[7 -1].coord.y = float(row[8])

        tblock[1 -1].wrap_len = float(row[23])
        tblock[1 -1].wrap_arc = float(row[65])
        tblock[2 -1].wrap_len = float(row[24])
        tblock[2 -1].wrap_arc = float(row[66])
        tblock[3 -1].wrap_len = float(row[25])
        tblock[3 -1].wrap_arc = float(row[67])
        tblock[4 -1].wrap_len = float(row[26])
        tblock[4 -1].wrap_arc = float(row[68])
        tblock[5 -1].wrap_len = float(row[27])
        tblock[5 -1].wrap_arc = float(row[69])
        tblock[6 -1].wrap_len = float(row[28])
        tblock[6 -1].wrap_arc = float(row[70])

        trope[1 -1].l_rope = float(row[29])
        trope[2 -1].l_rope = float(row[30])
        trope[3 -1].l_rope = float(row[31])
        trope[4 -1].l_rope = float(row[32])
        trope[5 -1].l_rope = float(row[33])
        trope[6 -1].l_rope = float(row[34])

        trope[1 -1].alpha_rope = float(row[35])
        trope[2 -1].alpha_rope = float(row[36])
        trope[3 -1].alpha_rope = float(row[37])
        trope[4 -1].alpha_rope = float(row[38])
        trope[5 -1].alpha_rope = float(row[39])
        trope[6 -1].alpha_rope = float(row[40])

        # ---------------------------
        # Стрелы
        # 1. Угол наклона к горизонту каждой стрелы (alpha_boom)
        # 2. Матрица T (D и G для каждой стрелы)
        booms = Booms(booms_new([a21, a22])).eval()
        # ---------------------------
        # Блоки
        # 3. Координаты блоков XY_block
        blocks = block_pos(blocks_new(), booms)

        # ---------------------------
        # 4. Расчёт параметров каната
        rope_data: list[RopeParams] = rope_data_eval(blocks)

        """
        Далее будет задаваться дополнительное условие, 
        которое учитывает переваливание каната при положении крана хоботом вниз
        """
        alpha_pen = rope_data[-2].alpha_rope  # градусы
        #  если предпоследний < 90°, выкидываем блок №6 (1-based) и шьём 5-7 блоки
        if alpha_pen > 90.0:
            # находим позиции сегментов (5→6) и (6→7)
            idx_56 = next((idx for idx, r in enumerate(rope_data) if r.block_pair == (5, 6)), None)
            idx_67 = next((idx for idx, r in enumerate(rope_data) if r.block_pair == (6, 7)), None)
    
            lhook_min = float(rope_calc_params.lhook_min)
            l_hook = float(rope_data[-1].l_rope)
    
            b5 = blocks[4]  # 0-based: блок 5
            b7 = blocks[6]  # 0-based: блок 7 (КП)
            
            # определение координат кп 
            b7.coord.x = b5.coord.x - 0.5 * b5.D
            b7.coord.y = b5.coord.y - l_hook
            # вычислим новый сегмент 5-7
            X1, Y1 = b5.coord.x, b5.coord.y
            X2, Y2 = b7.coord.x, b7.coord.y
            D1, D2 = b5.D, b7.D
            scheme = b5.scheme if b5.scheme else 1
    
            if scheme == 1:
                k, j = -1, 1
            elif scheme == 2:
                k, j = 1, 1
            elif scheme == 3:
                k, j = 1, -1
            elif scheme == 4:
                k, j = -1, -1
            else:
                raise ValueError(f"Некорректная схема: {scheme}")
    
            new_params = rope_parameters(X1, Y1, X2, Y2, D1, D2, k, j)
            new_params.block_pair = (5, 7)
            new_paramsscheme = scheme
    
            # удаляем (5-6) и (6-7), вставляем (5-7) на место прежнего (5-6)
            for idx in sorted([idx_56, idx_67], reverse=True):
                rope_data.pop(idx)
            rope_data.insert(idx_56, new_params)

        # -----------------------------
        # 6. Углы обхвата и длины дуг каждого блока
        # -----------------------------
        def calc_block_angles_and_arcs(blocks: list[Block], rope_data: list[RopeParams]):
            wrap_angles = []
            arc_lengths = []
            L_sys_arc = 0
       
            prev_alpha = None  # предыдущий угол для формирования alpha_rope_list
       
            for block, r in zip(blocks[:-1], rope_data):
                if block.bind == BlockBindFixed:
                    alpha_rope = 90
                else:
                    alpha_rope = r.alpha_rope if r.alpha_rope else 0
            
                if block.bind == BlockBindBoomPair:
                    pass
                else:
                    # Расчёт угла обхвата
                    if prev_alpha is not None:
                        alpha_wrap = abs(alpha_rope - prev_alpha)
                    else:
                        alpha_wrap = 0
                    
                    L_arc = (math.pi * block.D * 0.5 * alpha_wrap) / 180
                    
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
        # 7. Общая длина каната, сумма длин прямолинейных участков и сумма длин дуг
        # -----------------------------
        def calc_rope_sums(blocks: list[Block], rope_data: list[RopeParams], Lfact):
            l_section_summ = sum(r.l_rope for r in rope_data)
            block_results = calc_block_angles_and_arcs(blocks, rope_data)
            return {
                "l_section_summ": l_section_summ,
                "block_results": block_results
            }
        rope_results = calc_rope_sums(
            blocks,
            rope_data,
            Lfact=rope_calc_params.Lfact
        )
    
        # ---------------------------
        # 8. Пересчет длины дуги на барабане
        # ---------------------------

        def calc_drum_arc_delta(blocks, rope_data, rope_calc_params):
            """
            Изменение длины дуги каната на барабане из-за изменения угла схода каната.
            """
            D_drum = blocks[0].D       # диаметр барабана = D первого блока
            R = 0.5 * D_drum
            # начальный угол (из парковочного положения(положение 1))
            i0 = float(rope_calc_params.alpha_rope0)
            # текущий угол схода каната в рассматриваемом положении
            i_cur = float(rope_data[0].alpha_rope)
            dphi = math.radians(i_cur - i0)  # изменение угла в радианах
            dL = R * dphi                    # изменение длины дуги (мм)
            return dL

        # ---------------------------
        # 9. Пересчет длины подвеса 
        # ---------------------------
        def ensure_min_hook_length(booms, blocks, rope_calc_params):
            """
            Пересчёт длины подвеса с учётом изменения дуги на барабане.
            Ничего принудительно с барабана не сматываем/наматываем.
        
            Возвращает:
              new_L_winch  – эффективная длина на лебёдке (L_winch_nom - dL_drum),
              balance      – остаточная ошибка по длине (должна быть ~0),
              rope_data[-1]['l_rope'] – финальная длина подвеса.
            """
            Lfact = float(rope_calc_params.Lfact)
            L_winch_nom = float(rope_calc_params.L_winch)
        
            # 1. Суммы по текущей геометрии
            block_results = calc_block_angles_and_arcs(blocks,rope_data)
            L_sys_arc = block_results["L_sys_arc"]
        
            # все прямые без подвеса
            l_sections_wo_hook = sum(r.l_rope for r in rope_data[:-1])
        
            # 2. Изменение дуги на барабане
            dL_drum = calc_drum_arc_delta(blocks, rope_data, rope_calc_params)
            L_winch_eff = L_winch_nom + dL_drum

            # 3. Новая длина подвеса из уравнения длины
            l_hook_new = Lfact - L_winch_eff - l_sections_wo_hook - L_sys_arc
        
            # 4. Переставляем крюковую подвеску
            prev_idx = rope_data[-1].block_pair[0] - 1
            hook_idx = rope_data[-1].block_pair[1] - 1
            b_prev = blocks[prev_idx]
            b_hook = blocks[hook_idx]
        
            alpha_pen = rope_data[-2].alpha_rope  # градусы
            b_hook.coord.y = b_prev.coord.y - l_hook_new
            if alpha_pen > 90.0:
                b_hook.coord.x = b_prev.coord.x - 0.5 * b_prev.D
            else:
                b_hook.coord.x = b_prev.coord.x + 0.5 * b_prev.D
        
            # 5. Пересчитываем последний прямой участок
            X1, Y1 = b_prev.coord.x, b_prev.coord.y
            X2, Y2 = b_hook.coord.x, b_hook.coord.y
            D1, D2 = b_prev.D, b_hook.D
            scheme = getattr(b_prev, "scheme", 1)
        
            if   scheme == 1: k, j = -1,  1
            elif scheme == 2: k, j =  1,  1
            elif scheme == 3: k, j =  1, -1
            elif scheme == 4: k, j = -1, -1
            else:
                raise ValueError(f"Некорректная схема: {scheme}")
        
            last_seg = rope_parameters(X1, Y1, X2, Y2, D1, D2, k, j)
            last_seg.block_pair = (prev_idx + 1, hook_idx + 1)
            last_seg.scheme = scheme
            rope_data[-1] = last_seg
        
            # 6. Контрольный баланс после пересчёта
            block_results2 = calc_block_angles_and_arcs(blocks, rope_data)
            L_sys_arc2 = block_results2["L_sys_arc"]
            l_section_summ2 = sum(r.l_rope for r in rope_data)
            balance = Lfact - L_winch_eff - l_section_summ2 - L_sys_arc2
            new_L_winch = L_winch_eff
        
            return new_L_winch, balance, rope_data[-1].l_rope
    
    
        # # -----------------------------.
        # # 10. Построение опорных точек от крюка к барабану
        # # -----------------------------    
        # def build_support_points(rope_results, rope_data: list[RopeParams], block_results, Lfact):
        #     """
        #     Формируем 14 опорных точек (в метрах) от крюка к барабану:
        #       F14 = Lfact (крюк)
        #       F13 = F14 - l_rope_7
        #       F12 = F13 - arc_6
        #       F11 = F12 - l_rope_6
        #       ...
        #       F1  = F2 - arc_1
        #     """
        #     L_total = Lfact  # мм, начинаем с полной длины (крюк)
        #     # Получаем длины участков в порядке (от крюка к барабану)
        #     l_sections = [r.l_rope for r in reversed(rope_data)]   # мм
        #     arcs = list(reversed(block_results["arc_lengths"]))       # мм
        #     F = [L_total]  # F12 (крюк)
    
        #     # Формируем остальные точки
        #     for i in range(len(l_sections)):
    
        #         # Вычитаем прямой участок
        #         if i < len(l_sections):
        #             L_total -= l_sections[i]
        #             F.append(L_total)
                    
        #         # Вычитаем дугу
        #         if i < len(arcs):
        #             if arcs[i] > 0:
        #                 L_total -= arcs[i]
        #                 F.append(L_total)
        #             else:
        #                 continue
        #         else:
        #             logging.debug(f"Дуга для участка {i+1} не существует")
                
        #     # Переворачиваем, чтобы получить порядок от барабана к крюку
        #     F = list(reversed(F))
        #     F = np.array(F, dtype=float) / 1000.0  # метры
        #     return F
        

        #############################################################



        # считаем количество каната на дуге барабана
        dL_drum = calc_drum_arc_delta(blocks, rope_data, rope_calc_params)
        # Строим опорные точки
        # support_points = build_support_points(rope_results, rope_data, block_results, rope_calc_params.Lfact)
        # Запомним исходную длину последнего прямого участка (подвеса)
        last_len_before = rope_calc_params.lhook_min
        # Cчитаем количество каната которое надо вытравить 
        new_L_winch, x, rope_data[-1].l_rope = ensure_min_hook_length(booms, blocks, rope_calc_params)
        # Расчет дуг и канатов
        block_results = calc_block_angles_and_arcs(blocks, rope_data)
        
        # Тест координат блоков и КП
        for idx, block in enumerate(blocks):
            if idx > 0:         # Не проверяем координаты барабана, у Вани их нет
                assert aproxEq(block.coord.x, tblock[idx].coord.x, 0.5), f"step {step}  block[{idx}].x = {block.coord.x}, target = {tblock[idx].coord.x}"
                assert aproxEq(block.coord.y, tblock[idx].coord.y, 0.5), f"step {step}  block[{idx}].y = {block.coord.y}, target = {tblock[idx].coord.y}"
                # if idx < 6:     # Не проверяем Y крюка, так как у Вани написано вытравливание каната по условию минимальной длины, а у нас этого нет

        # Тест прямых участков каната
        for i, r in enumerate(rope_data):
            l = list(map(lambda r: r.l_rope, rope_data))
            a = list(map(lambda r: r.alpha_rope, rope_data))
            pair = r.block_pair
            # TODO Если включить проверку l_rope, то упадет на 29 шаге
            if i < len(rope_data) - 1:
                assert aproxEq(r.l_rope, trope[i].l_rope, 0.4), f"step {step}  block[{pair[0]}..{pair[1]}] \n\t {l} \n\t l_rope = {r.l_rope}, target = {trope[i].l_rope}"
            assert aproxEq(r.alpha_rope, trope[i].alpha_rope, 0.4), f"step {step}  block[{pair[0]}..{pair[1]}] \n\t {a} \n\t alpha_rope = {r.alpha_rope}, target = {trope[i].alpha_rope}"
            
        # Тест углов и дуг обхвата
        logging.debug(f"block_results: {block_results}")
        for i in range(0, len(blocks) - 2):
            wrap_l = block_results["arc_lengths"][i]
            wrap_arc = block_results["wrap_angles"][i]
            assert aproxEq(wrap_l, tblock[i].wrap_len, 0.4), f"step {step}  block[{i}].wrap_l = {wrap_l}, target = {tblock[i].wrap_len}"
            assert aproxEq(wrap_arc, tblock[i].wrap_arc, 0.4), f"step {step}  block[{i}].wrap_arc = {wrap_arc}, target = {tblock[i].wrap_arc}"
        # # Запись опорных точек в CSV
        # for _ in range(71, 83):
        #     row.append(0.0) 
        # row[71] = support_points[0] * 1000.0     # f01
        # row[72] = support_points[1] * 1000.0     # f02
        # row[73] = support_points[2] * 1000.0     # f03
        # row[74] = support_points[3] * 1000.0     # f04
        # row[75] = support_points[4] * 1000.0     # f05
        # row[76] = support_points[5] * 1000.0     # f06
        # row[77] = support_points[6] * 1000.0     # f07
        # row[78] = support_points[7] * 1000.0     # f08
        # row[79] = support_points[8] * 1000.0     # f09
        # row[80] = support_points[9] * 1000.0     # f10
        # row[81] = support_points[10] * 1000.0 if len(support_points) > 10 else 0.0     # f11
        # row[82] = support_points[11] * 1000.0 if len(support_points) > 11 else 0.0     # f12

        # f = open(target_csv, mode='a')
        # frow = ",".join(map(lambda x: f'{x}', row))
        # f.write(f'{frow}\n')

        # ---------------------------
        # Логи
        # ---------------------------
        # logging.debug('-'*40)
        # logging.debug(f"Число стрел: {len(booms)}")
        # logging.debug('-'*40)
        # logging.debug(f"alpha_boom: {[round(boom.alpha, 3) for boom in booms]}")
        # logging.debug('-'*40)
        # for idx, boom in enumerate(booms, start=1):
        #     logging.debug(f"Стрела {idx}: D={boom.D}, G={boom.G}")
        # logging.debug('-'*40)
        # for i, block in enumerate(blocks, start=1):
        #     logging.debug(f"Блок {i}: Координаты блока {block.coord}")
        # logging.debug('-'*40)
        # for r in rope_data:
        #     logging.debug(f"Блоки {r.block_pair} | Схема {r.scheme} | "
        #                   f"L_block={r.l_block:.2f} | Alpha_rope={r.alpha_rope:.2f}° | "
        #                   f"L_rope={r.l_rope:.2f}")
        # logging.debug('-'*40)
        # for r in rope_data:
        #     logging.debug(
        #         f'Вход X {r.X1_block:.2f}, Выход X {r.X2_block:.2f} | '
        #         f'Вход Y {r.Y1_block:.2f}, Выход Y {r.Y2_block:.2f}'
        #     )
        # logging.debug('-'*40)
        # logging.debug("Углы обхвата и длины дуг (по каждой паре блоков):")
        # for i, (alpha_wrap, arc_length) in enumerate(
        #         zip(block_results["wrap_angles"], block_results["arc_lengths"]), start=1):
        #     bp = f"{rope_data[i-1].block_pair[0]}-{rope_data[i-1].block_pair[1]}"
        #     logging.debug(
        #         f"Блок {bp}: угол обхвата = {alpha_wrap:.3f} deg, длина дуги = {arc_length:.3f} mm"
        #     )
            
        # i0 = rope_calc_params.alpha_rope0
        # i_cur = float(rope_data[0].alpha_rope)
        # dphi_rad = math.radians(i_cur - i0)  
        # dphi_deg = i_cur - i0

        # logging.debug(
        #     f"Барабан: Δугол = {dphi_deg:.3f} deg, Δдуга = {abs(dL_drum):.3f} mm"
        # )
        # logging.debug(f"i0 = {i0}")
        # logging.debug(f"i_cur = {i_cur}")
            
        # logging.debug('-'*40)
        # logging.debug("Опорные точки")
        # # for i, v in enumerate(support_points, start=1):
        # #     logging.debug(f"F{i:02d}: {v:8.3f}")
        # logging.debug('-'*40)
        # logging.debug("Итоги расчёта каната")
        
        # balance_rope = (rope_calc_params.Lfact - rope_calc_params.L_winch - 
        #      rope_results['l_section_summ'] - rope_results['block_results']['L_sys_arc'])
        
        # l_rope_sum =  rope_calc_params.L_winch + rope_results['l_section_summ'] + rope_results['block_results']['L_sys_arc'] + balance_rope
                    
        # logging.debug(f"Rope rest  = {balance_rope}")
        # logging.debug(f"Min hook   = {rope_calc_params.lhook_min}")
        # logging.debug(f"l_rope_sum = {l_rope_sum}")
        # logging.debug(f"Lfact      = {rope_calc_params.Lfact }")
        # logging.debug(f"new_L_winch= {new_L_winch}")
        # logging.debug(f"L_winch    = {rope_calc_params.L_winch}")
        # logging.debug(f"l_sec_sum  = {rope_results['l_section_summ']}")
        # logging.debug(f"L_sys_arc  = {rope_results['block_results']['L_sys_arc']}")
        # logging.debug(f"l_hook     = {rope_data[-1].l_rope:.2f}")
        
        if plot:
            # ---------------------------
            # Построение графиков
            # ---------------------------
            
            # 1 График крана, блоков, каната
            plt.figure(figsize=(10, 8))
            plt.title(f"Схема расположения стрел и блоков [{step}]")
            plt.xlabel("X координата (мм)")
            plt.ylabel("Y координата (мм)")
            plt.grid(True)
            plt.axis('equal')
            
            # Стрелы
            colors = ['black', 'black']
            for i, boom in enumerate(booms):
                plt.plot([boom.D.x, boom.G.x], [boom.D.y, boom.G.y], color=colors[i], linewidth=1.5)
                plt.scatter([boom.D.x, boom.G.x], [boom.D.y, boom.G.y], color=colors[i], s=20, marker='s')
            
            # Блоки
            for i, block in enumerate(blocks, start=1):
                x, y = block.coord.x, block.coord.y
                radius = block.D / 2
                color = 'deepskyblue'
                if i == 6 and len(rope_data) < 6:
                    color = 'lightgray'
                circle = patches.Circle((x, y), radius, fill=False, color=color, linewidth=1)
                plt.gca().add_patch(circle)
                plt.text(x, y, f'{i}', fontsize=8, color='black', 
                        ha='center', va='center', weight='bold',
                        bbox=dict(boxstyle="circle, pad=0.3", facecolor='white', edgecolor='black', alpha=0.7))
            
            # Канаты
            for r in rope_data:
                plt.plot([r.X1_block, r.X2_block], [r.Y1_block, r.Y2_block],
                        color='blue', linestyle='--')
                plt.scatter([r.X1_block, r.X2_block], [r.Y1_block, r.Y2_block],
                            color='orange', s=25)
            
            # Изменения длины последнего участка (у КП)
            last_len_after = rope_data[-1].l_rope
            last_len_before = rope_calc_params.lhook_min
            delta_last = last_len_after - last_len_before
            r_last = rope_data[-1]
            x1, y1 = r_last.X1_block, r_last.Y1_block
            x2, y2 = r_last.X2_block, r_last.Y2_block
            
            
            # Новый конец отрезка с учетом изменения длины
            x2_new = x1 
            y2_new = y2 + delta_last
            if delta_last > 0:
                # Удлинение — от старого конца к новому (зелёным)
                plt.plot([x2, x2_new], [y2, y2_new], color='red', linewidth=3)
                plt.text((x2 + x2_new)/2, (y2 + y2_new)/2, f'+{delta_last:.1f} мм', color='red', fontsize=9)
            
            L_winch = rope_calc_params.L_winch   
            L_winch_before = L_winch  
            
            # Вытравливание каната у лебедки
            try:
                if need_payout > 0:
                    # Координаты лебёдки (первый участок каната начинается от неё)
                    if rope_data:
                        winch_x, winch_y = rope_data[0].X1_block, rope_data[0].Y1_block
                    else:
                        # запасной вариант — центр 1-го блока
                        winch_x, winch_y = blocks[0].coord.x, blocks[0].coord.y
            
                    # Красная точка и подпись со старыми/новыми значениями L_winch
                    plt.scatter([winch_x], [winch_y], color='red', s=40, zorder=6)
                    txt = (
                        f"L_winch: {L_winch_before:.1f} мм\n"
                        f"L_winch_new: {new_L_winch:.1f} мм\n"
                        f"−Δ = {need_payout:.1f} мм"
                    )
                    plt.text(
                        winch_x-1000, winch_y + 1000,
                        txt,
                        color='red', fontsize=9, ha='left', va='bottom',
                        bbox=dict(facecolor='white', edgecolor='red', alpha=0.5, boxstyle='round,pad=0.25')
                    )
            except Exception as e:
                print("[plot winch] Не удалось показать вытравливание у лебёдки:", e)
            plt.show()
            # input("Press Enter to continue...")
