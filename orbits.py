import graphviz
import itertools
import sys

class Transducer(object):
    def __init__(self, start, transitions):
        self.start = start
        self.transitions = transitions

    def step(self, word):
        state = self.start
        new_word = []
        for c in word:
            state, o = self.transitions[(state, int(c))]
            new_word.append(o)
        return ''.join([str(x) for x in new_word])

    def orbits(self, n):
        if n == 0:
            return frozenset([frozenset([''])])
        else:
            out = set()
            for orbit in self.orbits(n-1):
                root = min(orbit)
                rootl = root + '0'
                rootr = root + '1'
                word = rootl
                new_orbit = set()
                while word not in new_orbit:
                    new_orbit.add(word)
                    word = self.step(word)
                out.add(frozenset(new_orbit))
                if rootr in new_orbit:
                    continue
                word = rootr
                new_orbit = set()
                while word not in new_orbit:
                    new_orbit.add(word)
                    word = self.step(word)
                out.add(frozenset(new_orbit))
            return frozenset(out)

    def orbit_tree(self, n):
        dot = graphviz.Digraph()
        dot.node('e', '', root='true')
        for i in range(1, n+1):
            for orbit in self.orbits(i):
                root = min(orbit)
                #print(root)
                #word = self.step(root)
                #while word != root:
                #    print('\t', word)
                #    word = self.step(word)
                dot.node(root, '')
                if i == 1:
                    dot.edge('e', root, label=' ' + root[-1])
                else:
                    dot.edge(root[:-1], root, label=' ' + root[-1])
        return dot

def has_unreachable_state(t_dict):
    seen = set([0])
    for _ in range(len(t_dict) // 2):
        for (k, c) in t_dict:
            if k in seen:
                seen.add(t_dict[(k, c)][0])
    return len(seen) < (len(t_dict) // 2)

def redundant_graph_permutation(t):
    # Error if some state is unreachable.
    states = range(len(t) // 2)
    for (a, b) in zip(states[1:-1], states[2:]):
        if t[0].index(a) > t[0].index(b):
            return True
    return False

def redundant_inversion(t_dict):
    for i in range(len(t_dict) // 2):
        if t_dict[(i, 0)][0] == 0:
            continue
        elif t_dict[(i, 0)][0] == t_dict[(i, 1)][0]:
            continue
        return t_dict[(i, 0)][0] > t_dict[(i, 1)][0]
    return False


def all_transducers(size):
    states = range(size)
    start = 0
    state_transitions = itertools.product(
        states, repeat=2*size)
    symbol_transitions = itertools.product(range(2), repeat=size)
    transitions = itertools.product(state_transitions, symbol_transitions)
    for t in transitions:
        t_dict = {}
        for i, (state, sym) in enumerate(itertools.product(states, range(2))):
            t_dict[(state, sym)] = (t[0][i], (t[1][i//2] + sym) % 2)
        if has_unreachable_state(t_dict):
            continue
        if redundant_graph_permutation(t):
            continue
        if redundant_inversion(t_dict):
            continue
        yield Transducer(start, t_dict)

example = Transducer(0, {
    (0, 0): (2, 1),
    (0, 1): (1, 0),
    (1, 0): (0, 0),
    (1, 1): (0, 1),
    (2, 0): (1, 0),
    (2, 1): (1, 1)
})

example2 = Transducer(0, {
    (0, 0): (2, 1),
    (0, 1): (1, 0),
    (1, 0): (3, 1),
    (1, 1): (4, 0),
    (2, 0): (3, 0),
    (2, 1): (4, 1),
    (3, 0): (0, 0),
    (3, 1): (1, 1),
    (4, 0): (2, 1),
    (4, 1): (0, 0)
})

example3 = Transducer(0, {
    (0, 0): (1, 0),
    (0, 1): (2, 1),
    (1, 0): (0, 1),
    (1, 1): (1, 0),
    (2, 0): (1, 0),
    (2, 1): (1, 1),
})

example4 = Transducer('C', {
    ('A', 0): ('B', 1),
    ('A', 1): ('C', 0),
    ('B', 0): ('A', 1),
    ('B', 1): ('B', 0),
    ('C', 0): ('B', 0),
    ('C', 1): ('A', 1)
})

example5 = Transducer('C', {
    ('A', 0): ('B', 1),
    ('A', 1): ('C', 0),
    ('B', 0): ('C', 1),
    ('B', 1): ('A', 0),
    ('C', 0): ('A', 0),
    ('C', 1): ('C', 1)
})

def classify(size, depth):
    classes = {}
    exemplars = {}
    for i, x in enumerate(all_transducers(size)):
        if i in (62, 78):
            print(i, x.transitions)
        fingerprint = x.orbits(depth)
        if fingerprint in classes:
            classes[fingerprint].append((i, x))
        else:
            classes[fingerprint] = [(i, x)]
        exemplars[i] = classes[fingerprint][0][0]
    print('! ',  i+1)
    return classes, exemplars

if __name__ == '__main__':

    """
    size = 3
    for i, x in enumerate(all_transducers(size)):
        if i in (62, 78):
            print(x.transitions)
            x.orbit_tree(12).render('nasty2_' + str(size) + '_' + str(i) + '.dot',
                                    engine='twopi',
                                    format='png',
                                    view=False)

    sys.exit(0)

    size = int(sys.argv[1])
    exemplars = None
    for depth in range(1, 20):
        old_exemplars = exemplars
        classes, exemplars = classify(size, depth)
        if old_exemplars:
            for k in exemplars:
                if exemplars[k] != old_exemplars[k]:
                    print(depth, k, exemplars[k], old_exemplars[k])
        xs = [len(classes[cls]) for cls in classes]
        xs2 = {x: xs.count(x) for x in xs}
        print(depth, len(classes), sorted(xs2.items()))


    word = [0] * 5
    for i in range(32):
        print(word)
        word = example.step(word)
    for i in range(10):
        print(example.orbits(i))
    """

    example4.orbit_tree(12).render('example', engine='twopi', view=True)
    """
    seen = set()
    size = 3
    for i, x in enumerate(all_transducers(size)):
        fingerprint = str(x.orbits(8))
        if fingerprint in seen:
            continue
        seen.add(fingerprint)
        print(i, len(seen), x.transitions)
        x.orbit_tree(12).render('example' + str(size) + '_' + str(i) + '.dot',
                                engine='dot',
                                format='png',
                                view=False)
    print(i+1)
    """
