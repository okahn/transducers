import itertools
goods = [
    [(1, 0, 0), (1, 0, 1), (1, 1, 0), (1, 1, 1)],
    [(1, 0, 0), (1, 0, 1), (0, 1, 0), (0, 1, 1)]
]

def closure(good, transition):
    signature = []
    for x in good:
        new = [0, 0, 0]
        for i in range(3):
            j, k = transition[2*i], transition[2*i+1]
            new[j] += x[i]
            new[k] += x[i]
            new[j] %= 2
            new[k] %= 2
        if tuple(new) in good:
            signature.append(tuple(new))
        else:
            signature.append(None)
    return signature

def has_cycle(good, signature):
    for x in signature:
        if x is None:
            continue
        next = x
        seen = set([x])
        while True:
            next = signature[good.index(next)]
            if next is None:
                break
            if next in seen:
                return True
            seen.add(next)
    return False


if __name__ == '__main__':
    for i, good in enumerate(goods):
        transitions = itertools.product(range(3), repeat=6)
        for t in transitions:
            if t == (0, 2, 0, 0, 0, 1):
                print(i, t, has_cycle(good, closure(good, t)), closure(good, t))
