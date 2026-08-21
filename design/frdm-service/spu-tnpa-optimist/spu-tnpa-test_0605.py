import matplotlib.pyplot as plt
import math
import logging
import numpy as np
import matplotlib.patches as patches
from dataclasses import dataclass
from copy import deepcopy

plt.set_loglevel(level="INFO")
logging.getLogger("PIL").setLevel(logging.WARNING)
logging.basicConfig(level=logging.DEBUG, force=True)


# ============================================================
# Типы привязки блоков
# ============================================================

@dataclass
class BlockBindFixed:
    """Блок вне стрелы: барабан / неподвижный блок"""
    pass


@dataclass
class BlockBindBoom:
    """Блок на стреле"""
    boom: int

    def __init__(self, boom: int):
        self.boom = boom


@dataclass
class BlockBindHook:
    """Блок на крюковой подвеске"""
    pass


BlockBind = BlockBindFixed | BlockBindBoom | BlockBindHook


# ============================================================
# Базовые классы
# ============================================================

class Offset:
    def __init__(self, x: float, y: float):
        self.x = x
        self.y = y

    def __str__(self):
        return f"{self.x, self.y}"


class Boom:
    def __init__(self, alpha_rel: float, len: float, l1: float, l2: float, l3: float, l4: float):
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
    def __init__(self, lF: Offset, D: float, scheme: int, bind: BlockBind):
        self.lF = lF
        self.D = D
        self.scheme = scheme
        self.bind = bind
        self.coord = Offset(0.0, 0.0)


@dataclass
class CraneConfig:
    name: str
    booms: list
    blocks: list
    rope_calc_params: dict
    block_coord_mode: str
    use_rope_throwover: bool
    drum_layers: list | None = None


# ============================================================
# Вспомогательные функции
# ============================================================

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
    """
    Приводит угол к диапазону [-180; 180).
    """
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


def get_drum_layer_data(L_winch_mm, drum_layers):
    """
    Определяет слой барабана и эффективный диаметр барабана
    по текущей длине каната на барабане.

    L_winch_mm — длина каната на барабане, мм.
    drum_layers — список:
        (накопленная длина слоя, м; диаметр слоя, мм)
    """
    L_winch_m = L_winch_mm / 1000.0

    for layer_num, (L_layer_max_m, D_layer_mm) in enumerate(drum_layers, start=1):
        if L_winch_m <= L_layer_max_m:
            return layer_num, D_layer_mm, L_winch_m

    layer_num = len(drum_layers)
    D_layer_mm = drum_layers[-1][1]
    return layer_num, D_layer_mm, L_winch_m


def rope_parameters(X1, Y1, X2, Y2, D1, D2, k, j, block_pair=None):
    """
    Расчет касательной каната между двумя блоками.
    """
    l_block = l_section(Y1, Y2, X1, X2)

    if l_block == 0:
        raise ValueError(f"Блоки {block_pair}: расстояние между центрами равно 0")

    alpha_block = alpha_horiz(Y1, Y2, X1, X2)

    asin_arg = 0.5 * (D1 + k * D2) / l_block

    if abs(asin_arg) > 1:
        raise ValueError(
            f"\nНевозможная геометрия каната для пары блоков {block_pair}:\n"
            f"asin_arg = {asin_arg:.3f}, должен быть в диапазоне [-1; 1]\n"
            f"L между центрами = {l_block:.2f} мм\n"
            f"D1 = {D1}, D2 = {D2}, k = {k}, j = {j}\n"
            f"X1={X1:.2f}, Y1={Y1:.2f}\n"
            f"X2={X2:.2f}, Y2={Y2:.2f}\n"
            f"Проверьте координаты блоков или scheme."
        )

    alpha_rope = alpha_block + j * math.degrees(math.asin(asin_arg))
    alpha_rope = normalize_angle_deg(alpha_rope)

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

        params = rope_parameters(
            X1, Y1, X2, Y2,
            D1, D2,
            k, j,
            block_pair=(i + 1, i + 2),
        )

        params["block_pair"] = (i + 1, i + 2)
        params["scheme"] = scheme

        rope_data.append(params)

    return rope_data


def calc_block_angles_and_arcs(blocks, rope_data):
    wrap_angles = []
    arc_lengths = []
    L_sys_arc = 0

    prev_alpha = None

    for block, r in zip(blocks[:-1], rope_data):
        alpha_rope = r.get("alpha_rope", 0)

        if prev_alpha is not None:
            alpha_rope_list = [prev_alpha, alpha_rope]
        else:
            alpha_rope_list = [alpha_rope]

        r["alpha_rope_list"] = alpha_rope_list

        if len(alpha_rope_list) > 1:
            delta = normalize_angle_deg(alpha_rope_list[-1] - alpha_rope_list[0])
            alpha_wrap = abs(delta)
        else:
            alpha_wrap = 0

        R = block.D / 2
        L_arc = (math.pi * R * alpha_wrap) / 180

        L_sys_arc += L_arc
        wrap_angles.append(alpha_wrap)
        arc_lengths.append(L_arc)

        prev_alpha = alpha_rope

    return {
        "wrap_angles": wrap_angles,
        "arc_lengths": arc_lengths,
        "L_sys_arc": L_sys_arc,
    }


def calc_rope_sums(blocks, rope_data):
    l_section_summ = sum(r["l_rope"] for r in rope_data)
    block_results = calc_block_angles_and_arcs(blocks, rope_data)

    return {
        "l_section_summ": l_section_summ,
        "block_results": block_results,
    }


def calc_winch_length_auto(blocks, rope_data, rope_calc_params):
    rope_results = calc_rope_sums(blocks, rope_data)

    Lfact = rope_calc_params["Lfact"]
    l_section_summ = rope_results["l_section_summ"]
    L_sys_arc = rope_results["block_results"]["L_sys_arc"]

    L_winch_calc = Lfact - l_section_summ - L_sys_arc

    if L_winch_calc < 0:
        raise ValueError(
            f"Длины каната не хватает: L_winch_calc = {L_winch_calc / 1000:.3f} м"
        )

    return L_winch_calc, rope_results


def build_support_points(rope_data, block_results, Lfact):
    L_total = Lfact

    l_sections = [r["l_rope"] for r in reversed(rope_data)]
    arcs = list(reversed(block_results["arc_lengths"]))

    F = [L_total]

    for i in range(len(l_sections)):
        L_total -= l_sections[i]
        F.append(L_total)

        if i < len(arcs):
            if arcs[i] > 0:
                L_total -= arcs[i]
                F.append(L_total)

    F = list(reversed(F))
    F = np.array(F, dtype=float) / 1000.0
    return F


# ============================================================
# Конфигурации кранов
# ============================================================

CRANE_CONFIGS = {
    "КСШ": CraneConfig(
        name="КСШ",
        booms=[
            Boom(alpha_rel=0, len=11200.0, l1=0.0, l2=0.0, l3=0.0, l4=10330.0),
            Boom(alpha_rel=23.78, len=7984.0, l1=0.0, l2=0.0, l3=0.0, l4=0.0),
        ],
        blocks=[
            Block(lF=Offset(1830.0,   710.0), D=845.000, scheme=1, bind=BlockBindFixed()),
            Block(lF=Offset(308.0,   1100.0), D=816.000, scheme=1, bind=BlockBindBoom(0)),
            Block(lF=Offset(-6549.0, 1730.0), D=816.000, scheme=1, bind=BlockBindBoom(1)),
            Block(lF=Offset(-1121.0,  973.0), D=816.000, scheme=2, bind=BlockBindBoom(1)),
            Block(lF=Offset(267.0,    860.0), D=816.000, scheme=3, bind=BlockBindBoom(1)),
            Block(lF=Offset(136.0,    -35.0), D=816.000, scheme=1, bind=BlockBindBoom(1)),
            Block(lF=Offset(0.0,        0.0), D=0.0,     scheme=0, bind=BlockBindHook()),
        ],
        rope_calc_params={
            "Lfact": 85045,
            "L_winch": 58330,
            "lhook_min": 1200,
            "hook_block_num": 7,

            # Вытравливание каната с барабана, мм
            "payout": 0.0,

            # Кратность. Если не нужна — оставить 1.
            "reeving_ratio": 1.0,
        },
        block_coord_mode="boom_base",
        use_rope_throwover=True,
        drum_layers=None,
    ),

    "СПУ": CraneConfig(
        name="СПУ",
        booms=[
            # Меняешь только углы
            Boom(alpha_rel=68.36, len=7440.5, l1=0.0, l2=0.0, l3=0.0, l4=0.0),
            Boom(alpha_rel=21.64, len=1648.0, l1=0.0, l2=0.0, l3=0.0, l4=0.0),
        ],
        blocks=[
            # Блок 1 — барабан. D автоматически заменится по таблице слоёв.
            Block(lF=Offset(-3322.50, -2811.96), D=1435.000, scheme=4, bind=BlockBindFixed()),
            Block(lF=Offset(-3280.00,  -914.66), D=1435.000, scheme=3, bind=BlockBindFixed()),
            Block(lF=Offset(-7002.00,   520.34), D=1435.000, scheme=1, bind=BlockBindFixed()),
            Block(lF=Offset(-7002.10,  2365.00), D=1435.000, scheme=2, bind=BlockBindFixed()),
            Block(lF=Offset(-6252.00,  3800.00), D=1435.000, scheme=3, bind=BlockBindFixed()),
            Block(lF=Offset(-763.00,   -717.50), D=1435.000, scheme=1, bind=BlockBindBoom(1)),
            Block(lF=Offset(0.00,         0.00), D=0.0,      scheme=0, bind=BlockBindHook()),
        ],
        rope_calc_params={
            # Полная длина каната СПУ, мм
            "Lfact": 3528.387 * 1000,

            # L_winch считается автоматически
            "L_winch": None,

            # Минимальная длина подвеса, мм
            "lhook_min": 1321.68,

            # Вытравливание каната с барабана, мм
            # 10 м = 10_000
            # 500 м = 500_000
            "payout": 5000.0,

            # Кратность. Если не нужна — оставить 1.
            "reeving_ratio": 1.0,

            "hook_block_num": 7,
        },
        block_coord_mode="global_xy",
        use_rope_throwover=False,
        drum_layers=[
            (383.24, 1435.000),
            (782.38, 1494.540),
            (1197.42, 1554.080),
            (1628.36, 1613.620),
            (2075.20, 1673.160),
            (2537.94, 1732.701),
            (3016.58, 1792.241),
            (3511.13, 1851.781),
        ],
    ),
}


# ============================================================
# Основной расчет
# ============================================================

if __name__ == "__main__":

    # ========================================================
    # Выбор типа крана
    # ========================================================

    crane_type = "СПУ"
    # crane_type = "КСШ"

    cfg = deepcopy(CRANE_CONFIGS[crane_type])

    booms = cfg.booms
    blocks = cfg.blocks
    rope_calc_params = cfg.rope_calc_params

    payout = float(rope_calc_params.get("payout", 0.0) or 0.0)
    reeving_ratio = float(rope_calc_params.get("reeving_ratio", 1.0) or 1.0)
    hook_extra = payout / reeving_ratio

    # ========================================================
    # 1. Угол наклона каждой стрелы относительно глобальной СК
    # ========================================================

    alpha_sum = 0.0

    for i, boom in enumerate(booms):
        alpha_sum += boom.alpha_rel
        boom.alpha = alpha_sum - i * 180

    # ========================================================
    # 2. Расчет точек D и G для каждой стрелы
    # ========================================================

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
        D_point = Offset(XY_start.x + Dx, XY_start.y + Dy)

        Gx, Gy = XY_rotate(boom.len - boom.l2, boom.l1, boom.alpha)
        G_point = Offset(XY_start.x + Gx, XY_start.y + Gy)

        boom.D = D_point
        boom.G = G_point

    # ========================================================
    # 3. Расчет координат блоков
    # ========================================================

    for idx, block in enumerate(blocks):

        match block.bind:

            case BlockBindFixed():

                if cfg.block_coord_mode == "global_xy":
                    block.coord.x = block.lF.x
                    block.coord.y = block.lF.y

                elif cfg.block_coord_mode == "boom_base":
                    dx1, dy1 = XY_rotate(-block.lF.x, block.lF.y, 0)
                    dx2, dy2 = XY_rotate(booms[0].l4, booms[0].l3, 90)

                    block.coord.x = dx1 + dx2
                    block.coord.y = dy1 + dy2

                else:
                    raise ValueError(f"Неизвестный режим привязки fixed-блоков: {cfg.block_coord_mode}")

            case BlockBindBoom(boom_index):
                boom = booms[boom_index]
                base_point = boom.G

                dx, dy = XY_rotate(block.lF.x, block.lF.y, boom.alpha)

                block.coord.x = base_point.x + dx
                block.coord.y = base_point.y + dy

            case BlockBindHook():
                block.coord.x = float("nan")
                block.coord.y = float("nan")

            case _:
                raise ValueError(f"Неизвестный тип блока [{idx}]")

    # ========================================================
    # 4. Расчет координат крюковой подвески с учетом payout
    # ========================================================

    hook_block_num = rope_calc_params["hook_block_num"]
    l_hook = rope_calc_params["lhook_min"]

    prev_idx = hook_block_num - 2

    x_prev = blocks[prev_idx].coord.x
    y_prev = blocks[prev_idx].coord.y
    D_prev = blocks[prev_idx].D

    blocks[hook_block_num - 1].coord.x = x_prev + 0.5 * D_prev
    blocks[hook_block_num - 1].coord.y = y_prev - (l_hook + hook_extra)

    # ========================================================
    # 5. Расчет прямолинейных участков каната
    # ========================================================

    rope_data = calc_rope_data(blocks)

    # ========================================================
    # 5.1. Перекидывание каната для КСШ
    # ========================================================

    if cfg.use_rope_throwover:

        alpha_pen = rope_data[-2].get("alpha_rope")

        if alpha_pen > 90.0:

            idx_56 = next((idx for idx, r in enumerate(rope_data) if r.get("block_pair") == (5, 6)), None)
            idx_67 = next((idx for idx, r in enumerate(rope_data) if r.get("block_pair") == (6, 7)), None)

            if idx_56 is not None and idx_67 is not None:

                l_hook_current = float(rope_data[-1].get("l_rope"))

                b5 = blocks[4]
                b7 = blocks[6]

                b7.coord.x = b5.coord.x - 0.5 * b5.D
                b7.coord.y = b5.coord.y - l_hook_current

                X1 = b5.coord.x
                Y1 = b5.coord.y
                X2 = b7.coord.x
                Y2 = b7.coord.y

                D1 = b5.D
                D2 = b7.D

                scheme = b5.scheme
                k, j = get_scheme_params(scheme)

                new_params = rope_parameters(
                    X1, Y1, X2, Y2,
                    D1, D2,
                    k, j,
                    block_pair=(5, 7),
                )

                new_params["block_pair"] = (5, 7)
                new_params["scheme"] = scheme

                for idx in sorted([idx_56, idx_67], reverse=True):
                    rope_data.pop(idx)

                rope_data.insert(idx_56, new_params)

    # ========================================================
    # 6. Для СПУ автоматически подбираем слой барабана
    # ========================================================

    if cfg.name == "СПУ" and cfg.drum_layers is not None:

        for iteration in range(10):
            old_D = blocks[0].D

            L_winch_calc, rope_results = calc_winch_length_auto(
                blocks,
                rope_data,
                rope_calc_params,
            )

            L_winch_max_mm = cfg.drum_layers[-1][0] * 1000.0

            if L_winch_calc > L_winch_max_mm:
                raise ValueError(
                    f"Расчётная длина каната на барабане превышает канатоёмкость: "
                    f"L_winch = {L_winch_calc / 1000:.3f} м, "
                    f"L_winch_max = {L_winch_max_mm / 1000:.3f} м"
                )

            layer_num, D_drum_eff, L_winch_m = get_drum_layer_data(
                L_winch_calc,
                cfg.drum_layers,
            )

            rope_calc_params["L_winch"] = L_winch_calc
            blocks[0].D = D_drum_eff

            rope_data = calc_rope_data(blocks)

            if abs(D_drum_eff - old_D) < 1e-6:
                break

        # Финальный пересчёт после стабилизации диаметра
        L_winch_calc, rope_results = calc_winch_length_auto(
            blocks,
            rope_data,
            rope_calc_params,
        )

        layer_num, D_drum_eff, L_winch_m = get_drum_layer_data(
            L_winch_calc,
            cfg.drum_layers,
        )

        rope_calc_params["L_winch"] = L_winch_calc
        blocks[0].D = D_drum_eff
        rope_data = calc_rope_data(blocks)
        rope_results = calc_rope_sums(blocks, rope_data)

    else:
        # Для КСШ L_winch задано в исходных данных.
        # При payout канат вытравливается с барабана, поэтому L_winch уменьшается.
        L_winch_base = float(rope_calc_params["L_winch"])
        new_L_winch = L_winch_base - payout

        if new_L_winch < 0:
            raise ValueError(
                f"Недостаточно каната на барабане для КСШ: "
                f"L_winch_base = {L_winch_base / 1000:.3f} м, "
                f"payout = {payout / 1000:.3f} м"
            )

        rope_calc_params["L_winch"] = new_L_winch
        rope_results = calc_rope_sums(blocks, rope_data)

    block_results = rope_results["block_results"]

    # ========================================================
    # 7. Опорные точки
    # ========================================================

    support_points = build_support_points(
        rope_data,
        block_results,
        rope_calc_params["Lfact"],
    )

    # ========================================================
    # 8. Итоговые параметры
    # ========================================================

    prev_idx = rope_data[-1]["block_pair"][0] - 1
    hook_idx = rope_data[-1]["block_pair"][1] - 1

    current_hook_length = blocks[prev_idx].coord.y - blocks[hook_idx].coord.y

    if cfg.drum_layers is not None:
        L_winch_max_mm = cfg.drum_layers[-1][0] * 1000.0
    else:
        L_winch_max_mm = float(rope_calc_params["L_winch"]) + payout

    payout_from_full_drum = L_winch_max_mm - float(rope_calc_params["L_winch"])

    if payout_from_full_drum < 0:
        payout_from_full_drum = 0.0

    balance_rope = (
        rope_calc_params["Lfact"]
        - rope_calc_params["L_winch"]
        - rope_results["l_section_summ"]
        - rope_results["block_results"]["L_sys_arc"]
    )

    l_rope_sum = (
        rope_calc_params["L_winch"]
        + rope_results["l_section_summ"]
        + rope_results["block_results"]["L_sys_arc"]
        + balance_rope
    )

    # ========================================================
    # 9. Логи
    # ========================================================

    logging.debug("-" * 40)
    logging.debug(f"Тип крана: {cfg.name}")
    logging.debug(f"Число стрел: {len(booms)}")
    logging.debug("-" * 40)

    logging.debug(f"alpha_boom: {[round(boom.alpha, 3) for boom in booms]}")
    logging.debug("-" * 40)

    for idx, boom in enumerate(booms, start=1):
        logging.debug(f"Стрела {idx}: D={boom.D}, G={boom.G}")

    logging.debug("-" * 40)

    for i, block in enumerate(blocks, start=1):
        logging.debug(f"Блок {i}: координаты {block.coord}, D={block.D}, scheme={block.scheme}")

    logging.debug("-" * 40)

    for r in rope_data:
        logging.debug(
            f"Блоки {r['block_pair']} | Схема {r['scheme']} | "
            f"L_block={r['l_block']:.2f} | Alpha_rope={r['alpha_rope']:.2f}° | "
            f"L_rope={r['l_rope']:.2f}"
        )

    logging.debug("-" * 40)
    logging.debug("Углы обхвата и длины дуг:")

    for i, (alpha_wrap, arc_length) in enumerate(
        zip(block_results["wrap_angles"], block_results["arc_lengths"]),
        start=1,
    ):
        if i - 1 < len(rope_data):
            bp = f"{rope_data[i - 1]['block_pair'][0]}-{rope_data[i - 1]['block_pair'][1]}"
        else:
            bp = "?"

        logging.debug(
            f"Блок {bp}: угол обхвата = {alpha_wrap:.3f} deg, "
            f"длина дуги = {arc_length:.3f} mm"
        )

    logging.debug("-" * 40)
    logging.debug("Опорные точки:")

    for i, v in enumerate(support_points, start=1):
        logging.debug(f"F{i:02d}: {v:8.3f}")

    logging.debug("-" * 40)
    logging.debug("Итоги расчёта каната")

    logging.debug(f"Полная длина каната Lfact = {rope_calc_params['Lfact'] / 1000:.3f} м")
    logging.debug(f"Минимальная длина подвеса lhook_min = {rope_calc_params['lhook_min'] / 1000:.3f} м")
    logging.debug(f"Фактическая длина подвеса current_hook_length = {current_hook_length / 1000:.3f} м")

    logging.debug(f"Заданное вытравливание payout = {payout / 1000:.3f} м")
    logging.debug(f"Дополнительное опускание крюка hook_extra = {hook_extra / 1000:.3f} м")

    logging.debug(f"Максимальная канатоёмкость барабана L_winch_max = {L_winch_max_mm / 1000:.3f} м")
    logging.debug(f"Текущая длина каната на барабане L_winch = {rope_calc_params['L_winch'] / 1000:.3f} м")
    logging.debug(f"Вытравлено с полного барабана = {payout_from_full_drum / 1000:.3f} м")

    if cfg.drum_layers is not None:
        layer_num_log, D_drum_log, L_winch_m_log = get_drum_layer_data(
            rope_calc_params["L_winch"],
            cfg.drum_layers,
        )

        logging.debug(f"Текущий слой барабана = {layer_num_log} из {len(cfg.drum_layers)}")
        logging.debug(f"Диаметр слоя барабана = {D_drum_log:.3f} мм")
    else:
        logging.debug(f"Диаметр барабана = {blocks[0].D:.3f} мм")

    # logging.debug(f"l_rope_sum = {l_rope_sum:.3f} мм")
    # logging.debug(f"Lfact      = {rope_calc_params['Lfact']:.3f} мм")
    # logging.debug(f"l_sec_sum  = {rope_results['l_section_summ']:.3f} мм")
    # logging.debug(f"L_sys_arc  = {rope_results['block_results']['L_sys_arc']:.3f} мм")
    # logging.debug(f"l_hook     = {rope_data[-1]['l_rope']:.3f} мм")
    # logging.debug(f"Rope rest  = {balance_rope:.3f} мм")

    # ========================================================
    # 10. Построение графика
    # ========================================================

    plt.figure(figsize=(10, 8))
    plt.title(f"Схема расположения стрел и блоков — {cfg.name}")
    plt.xlabel("X координата (мм)")
    plt.ylabel("Y координата (мм)")
    plt.grid(True)
    plt.axis("equal")

    # Стрелы
    for boom in booms:
        plt.plot(
            [boom.D.x, boom.G.x],
            [boom.D.y, boom.G.y],
            color="black",
            linewidth=1.5,
        )
        plt.scatter(
            [boom.D.x, boom.G.x],
            [boom.D.y, boom.G.y],
            color="black",
            s=20,
            marker="s",
        )

    # Блоки
    for i, block in enumerate(blocks, start=1):
        x_block = block.coord.x
        y_block = block.coord.y
        radius = block.D / 2

        if block.D > 0:
            circle = patches.Circle(
                (x_block, y_block),
                radius,
                fill=False,
                color="deepskyblue",
                linewidth=1,
            )
            plt.gca().add_patch(circle)

        plt.text(
            x_block,
            y_block,
            f"{i}",
            fontsize=8,
            color="black",
            ha="center",
            va="center",
            weight="bold",
            bbox=dict(
                boxstyle="circle, pad=0.3",
                facecolor="white",
                edgecolor="black",
                alpha=0.7,
            ),
        )

    # Канаты
    for r in rope_data:
        plt.plot(
            [r["X1_block"], r["X2_block"]],
            [r["Y1_block"], r["Y2_block"]],
            color="blue",
            linestyle="--",
        )
        plt.scatter(
            [r["X1_block"], r["X2_block"]],
            [r["Y1_block"], r["Y2_block"]],
            color="orange",
            s=25,
        )

    # Информационная плашка
    try:
        txt = (
            f"Вытравлено каната:\n"
            f"задано payout = {payout / 1000:.3f} м\n"
            f"с полного барабана = {payout_from_full_drum / 1000:.3f} м\n"
            f"L_winch = {rope_calc_params['L_winch'] / 1000:.3f} м\n"
            f"L_winch max = {L_winch_max_mm / 1000:.3f} м\n"
            f"D барабана = {blocks[0].D:.3f} мм"
        )

        if cfg.drum_layers is not None:
            txt += f"\nслой = {layer_num_log} из {len(cfg.drum_layers)}"

        ax = plt.gca()
        ax.text(
            0.02, 0.98,
            txt,
            transform=ax.transAxes,
            color="red",
            fontsize=10,
            ha="left",
            va="top",
            bbox=dict(
                facecolor="white",
                edgecolor="red",
                linewidth=1.5,
                alpha=0.90,
                boxstyle="round,pad=0.4",
            ),
            zorder=20,
        )

    except Exception as e:
        print("[plot payout info] Не удалось показать вытравленный канат:", e)

    plt.show()
    