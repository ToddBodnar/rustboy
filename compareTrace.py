line = 0

prevEx = None
prevAc = None

for expected, actual in zip(open("expected/mgba.out"), open("log.out")):
    if not expected[0:68] == actual[0:68]:
        print("Problem on", line)
        print(prevEx)
        print(prevAc)
        print(expected)
        print(actual)
    else:
        prevEx = expected
        prevAc = actual

    line += 1
