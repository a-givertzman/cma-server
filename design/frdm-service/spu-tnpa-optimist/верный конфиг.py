    "СПУ": CraneConfig(
        name="СПУ",
        booms=[
            # Меняешь только углы
            Boom(alpha_rel=50, len=7440.5, l1=0.0, l2=0.0, l3=0.0, l4=0.0),
            Boom(alpha_rel=40, len=1648.0, l1=0.0, l2=0.0, l3=0.0, l4=0.0),
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
            "payout": 0,

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
