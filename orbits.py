from typing import Union
import graphviz
import itertools
import pathlib
import sys
from datetime import datetime

t_label = Union[int, str]
t_transition = dict[tuple[t_label, int], tuple[t_label, int]]

class Transducer(object):
    def __init__(self, start: t_label, transitions: t_transition):
        self.start = start
        self.transitions = transitions

    def step(self, word: str):
        state = self.start
        new_word = []
        for c in word:
            state, o = self.transitions[(state, int(c))]
            new_word.append(o)
        return ''.join([str(x) for x in new_word])

    def orbits(self, n: int):
        if n == 0:
            return frozenset([frozenset([''])])
        else:
            out : set[frozenset[str]] = set()
            for orbit in self.orbits(n-1):
                root = min(orbit)
                rootl = root + '0'
                rootr = root + '1'
                word = rootl
                new_orbit : set[str] = set()
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

    def orbit_label(self, start: str) -> str:
        label = start
        word = start
        while True:
            word = self.step(word)
            label = min(word, label)
            if word == start:
                break
        return label

    def orbit_compare(self, other:'Transducer', depth: int) -> bool:
        for word in itertools.product(range(2), repeat=depth):
            word = ''.join(map(str, word))
            if self.orbit_label(word) != other.orbit_label(word):
                return False
        return True

    def orbit_tree(self, n : int):
        dot = graphviz.Digraph()
        dot.node('e', '0', root='true')
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

    def machine_graph(self):
        dot = graphviz.Digraph()
        seen = set()
        dot.node('init', '', style='invis')
        for ((n1, b1), (n2, b2)) in self.transitions.items():
            if len(seen) == 0:
                dot.node(str(n1), str(n1), root='true')
                dot.edge('init', str(n1))
                seen.add(n1)
            if n1 not in seen:
                dot.node(str(n1), str(n1))
            if n2 not in seen:
                dot.node(str(n2), str(n2))
            dot.edge(str(n1), str(n2), label=' ' + str(b1) + '/' + str(b2))
        return dot

def has_unreachable_state(t_dict: t_transition):
    seen : set[t_label] = set([0])
    for _ in range(len(t_dict) // 2):
        for (k, c) in t_dict:
            if k in seen:
                seen.add(t_dict[(k, c)][0])
    return len(seen) < (len(t_dict) // 2)

def redundant_graph_permutation(t: tuple[tuple[int, ...], tuple[int, ...]]):
    # Error if some state is unreachable.
    states = range(len(t[0]) // 2)
    for (a, b) in zip(states[1:-1], states[2:]):
        if t[0].index(a) > t[0].index(b):
            return True
    return False

def redundant_inversion(t_dict : t_transition):
    for i in range(len(t_dict) // 2):
        if t_dict[(i, 0)][0] == 0:
            continue
        elif t_dict[(i, 0)][0] == t_dict[(i, 1)][0]:
            continue
        return str(t_dict[(i, 0)][0]) > str(t_dict[(i, 1)][0])
    return False


def all_transducers(size: int):
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
        #if has_unreachable_state(t_dict):
        #    continue
        #if redundant_graph_permutation(t):
        #    continue
        #if redundant_inversion(t_dict):
        #    continue
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
    ('C', 0): ('A', 0),
    ('C', 1): ('B', 1)
})

example5 = Transducer('C', {
    ('A', 0): ('B', 1),
    ('A', 1): ('C', 0),
    ('B', 0): ('C', 1),
    ('B', 1): ('A', 0),
    ('C', 0): ('A', 0),
    ('C', 1): ('C', 1)
})

def classify(size: int, depth: int, debug = False) -> tuple[dict[int, list[tuple[int, Transducer]]], dict[int, int]]:
    if depth == 0:
        out = list(enumerate(all_transducers(size)))
        return {0: out}, {x: 0 for x in (x[0] for x in out)}
    classes: dict[int, list[tuple[int, Transducer]]] = {}
    exemplars: dict[int, int] = {}
    old_classes, old_exemplars = classify(size, depth=depth-1, debug=debug)
    a = datetime.now()
    for i, x in enumerate(all_transducers(size)):
        b = datetime.now()
        interval = (b-a) * (len(old_exemplars) / max(1, i) - 1)
        if debug:
            print('testing {}/{}, {:02d}:{:02d}:{:02d} remaining'.format(
                i, len(old_exemplars),
                int(interval.total_seconds() / 3600),
                int((interval.total_seconds() % 3600) / 60),
                int(interval.total_seconds() % 60)
            ), end = '\r')
        if old_exemplars[i] == i:
            classes[i] = [(i, x)]
            exemplars[i] = i
            continue
        for j, y in old_classes[old_exemplars[i]]:
            if j >= i:
                classes[i] = [(i, x)]
                exemplars[i] = i
                break
            if exemplars[j] != j:
                continue
            if y.orbit_compare(x, depth):
                classes[j].append((i, x))
                exemplars[i] = j
                break
    xs = [len(classes[cls]) for cls in classes]
    xs2 = {x: xs.count(x) for x in xs}
    if debug:
        print(depth, len(classes), sorted(xs2.items()), flush=True)
    return classes, exemplars

def coin_images():
    size = 3
    classes, exemplars = classify(size, 6)
    for (i, machines) in classes.items():
        depth = 6
        for j in range(6, 10):
            x = machines[0][1].orbits(j)
            if len(x) < 100:
                depth = j
            else:
                break
        machines[0][1].orbit_tree(depth).render('images/tree_{}_{}.gv'.format(size, i), engine='dot', format='png', view=False)
        pathlib.Path('images/{}_{}'.format(size, i)).mkdir(exist_ok=True)
        for (j, m) in machines:
            m.machine_graph().render('images/{size}_{i}/machine_{size}_{i}_{j}.gv'.format(size=size, i=i, j=j), engine='dot', format='png', view=False)
        print(machines[0])

if __name__ == '__main__':

    #        x.orbit_tree(12).render('nasty2_' + str(size) + '_' + str(i) + '.dot',
    #                                engine='twopi',
    #                                format='png',
    #                                view=False)

    size = 3
    # target = int(sys.argv[1])
    depth = int(sys.argv[1])
    # for (i, m) in enumerate(all_transducers(3)):
    #     if i == target:
    #         m.orbit_tree(depth).render('debug/{}_{}.gv'.format(size, target), engine='dot', format='png', view=True)
    #         last = 1
    classes, exemplars = classify(size, depth, debug=False)
    print('a', len(classes))
    sys.exit(0)
    for (i, machines) in classes.items():
        a = machines[0][1].orbits(6)
        b = machines[0][1].orbits(10)
        c = machines[0][1].orbits(9)
        d = machines[0][1].orbits(11)
        if len(a) > 1:
            assert len(c) > 1
        if len(c) == len(d):
            print(i)
