#!/bin/bash
# Runs a tournament between two versions of coronene.

me=$0
function usage()
{
    echo "Usage:"
    echo "    $me [OPTIONS] version_a version_b"
    echo ""
    echo "Where OPTIONS is any of:"
    echo "-o | --openings=name     set of openings to use"
    echo "-r | --rounds=#          number of rounds to play"
    echo "-s | --size=#            boardsize to play on"
    echo
}

source common.sh
if [ $# != 2 ]; then
    usage;
    exit 1;
fi

A=$1
B=$2

DIRECTORY="jobs/$A-vs-$B"
mkdir -p $DIRECTORY

./twogtp.py \
--type $TYPE \
--dir $DIRECTORY \
--openings $OPENINGS \
--size $SIZE --rounds $ROUNDS \
--p1cmd "builds/$A" --p1name "$A" \
--p2cmd "builds/$B" --p2name "$B"

