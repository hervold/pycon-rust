import re
import argparse


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument('fnames', nargs='*')

    args = parser.parse_args()

    for fname in args.fnames:
        with open(fname) as f:
            contents = f.read()

            contents = contents.replace('\n',' ')
            for block in re.findall(r'<BODY>(.*?) Reuter &#3;</BODY>', contents, re.I):

                for sent in re.split(r'\. +', block):
                    sent = sent.strip()
                    # this will wind up throwing away most sentences, but we'll ditch anything
                    #   containing non-alphabetic characters
                    if sent and not re.sub(r'[a-z, ]', '', sent.lower()):
                        print sent
