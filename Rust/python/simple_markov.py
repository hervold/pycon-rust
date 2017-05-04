import argparse
import random
from collections import defaultdict

COMMA = object()
SENT_BREAK = object()


def weighted_choice( cts ):
    acc = []
    for word, ct in cts.items():
        for _ in range(ct):
            acc.append(word)
    return random.choice(acc)


def read_corpus(fh):
    counts = defaultdict(lambda: defaultdict(int))
    for line in fh:
        words = line.split()
        for i in range(len(words) - 1):
            if words[i].endswith(','):
                w = trimcomma( words[i] )
                counts[ w ][ COMMA ] += 1
                counts[ w ][ trimcomma( words[i+1] ).lower() ] += 1
            else:
                counts[ words[i].lower() ][ trimcomma( words[i+1] ).lower() ] += 1
        counts[ words[-1].lower() ][ SENT_BREAK ] += 1
    return counts

def generate_sentence(counts):
    first = random.choice( counts.keys() )
    acc = [first]

    next = weighted_choice( counts[first] )
    while next is not SENT_BREAK:
        if next is COMMA:
            acc[-1] = acc[-1] + ','
            next = weighted_choice( counts[first] )
            while next is COMMA:
                next = weighted_choice( counts[first] )

        else:
            acc.append(next)
            if counts.get(next):
                next = weighted_choice( counts[next] )
            else:
                next = random.choice( counts.keys() )

    return ' '.join(acc) + '.'


trimcomma = lambda s: s.rstrip(',')


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument('sentences', type=int)
    parser.add_argument('fname')

    args = parser.parse_args()

    with open(args.fname) as fh:
        counts = read_corpus(fh)

    for _ in range(args.sentences):
        print generate_sentence(counts)
