line = 0

prevEx = None
prevAc = None

for expected, actual in zip(open("expected/mgba.out"), open("log.out")):
    cleaned = expected
    for i in range(1,10):
        cleaned = cleaned.replace("PC: 0"+str(i), "PC: 00")
    if not cleaned[0:68] == actual[0:68]:
        print("Problem on", line)
        print(prevEx)
        print(prevAc)
        print(expected)
        print(actual)
    else:
        prevEx = expected
        prevAc = actual

    line += 1
