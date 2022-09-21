import graphviz

class Transducer(object):
    def __init__(self, transitions):
        self.transitions = transitions

    def step(self, word):
        state = 0
        new_word = []
        for c in word:
            state, o = self.transitions[(state, c)]
            new_word.append(o)
        return new_word

    def orbits(self, n):
        if n == 0:
            return [[]]
        else:
            out = []
            for root in self.orbits(n-1):
                rootl = root + [0]
                rootr = root + [1]
                word = rootl[::1]
                done = False
                while not done:
                    done = True
                    word = self.step(word)
                    if word == rootl:
                        out.append(rootl)
                        out.append(rootr)
                    elif word == rootr:
                        out.append(rootl)
                    else:
                        done = False
            return out

    def orbit_tree(self, n):
        dot = graphviz.Digraph()
        dot.node('e', '')
        for i in range(1, n+1):
            for root in self.orbits(i):
                root = ''.join([str(d) for d in root])
                dot.node(root, '')
                if i == 1:
                    dot.edge('e', root, label=' ' + root[-1])
                else:
                    dot.edge(root[:-1], root, label=' ' + root[-1])
        return dot

example = Transducer({
    (0, 0): (2, 1),
    (0, 1): (1, 0),
    (1, 0): (0, 0),
    (1, 1): (0, 1),
    (2, 0): (1, 0),
    (2, 1): (1, 1)
})

example2 = Transducer({
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

example3 = Transducer({
    (0, 0): (1, 1),
    (0, 1): (2, 0),
    (1, 0): (0, 0),
    (1, 1): (1, 1),
    (2, 0): (1, 1),
    (2, 1): (1, 0),
})

example4 = Transducer({
    (0, 0): (0, 0),
    (0, 1): (1, 1),
    (1, 0): (0, 1),
    (1, 1): (2, 0),
    (2, 0): (0, 0),
    (2, 1): (0, 1)
})

if __name__ == '__main__':
    """
    word = [0] * 5
    for i in range(32):
        print(word)
        word = example.step(word)
    for i in range(10):
        print(example.orbits(i))
    """

    example4.orbit_tree(8).render('example', engine='dot', view=True)
