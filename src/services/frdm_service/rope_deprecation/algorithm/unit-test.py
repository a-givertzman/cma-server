import logging as log
log.basicConfig(level=log.DEBUG, force=True)

testData = [
    # step  input         target
    (01,    1, 0,         2),
    (02,    1, 1,         2),
    (03,    1, 2,         2),
    (04,    1, 3,         2),
    (05,    1, 4,         2),
]

for step, a, b, target in testData:
    result = a * b
    log.debug(f'{a} * {b}: {result},  target: {target}')
    assert(result == target, f'')