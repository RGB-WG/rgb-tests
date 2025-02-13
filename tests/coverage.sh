#!/bin/bash -e
#
# script to run project tests and report code coverage
# uses llvm-cov (https://github.com/taiki-e/cargo-llvm-cov)

COV="cargo llvm-cov"
COV_OPTS="--html"
CARGO_TEST_OPTS=("--")

_die() {
    echo "err $*"
    exit 1
}

_tit() {
    echo
    echo "========================================"
    echo "$@"
    echo "========================================"
}

help() {
    echo "$NAME [-h|--help] [-t|--test] [--ci] [--no-clean]"
    echo ""
    echo "options:"
    echo "    -h --help     show this help message"
    echo "    -t --test     only run these test(s)"
    echo "       --ci       run for the CI"
    echo "       --no-clean don't cleanup before the run"
}

# cmdline arguments
while [ -n "$1" ]; do
    case $1 in
        -h|--help)
            help
            exit 0
            ;;
        -t|--test)
            CARGO_TEST_OPTS+=("$2")
            shift
            ;;
        --ci)
            COV_OPTS="--lcov --output-path coverage.lcov"
            CI=1
            ;;
        *)
            help
            _die "unsupported argument \"$1\""
            ;;
    esac
    shift
done

if [ -z "$CI" ]; then
    _tit "installing requirements"
    rustup component add llvm-tools-preview
    cargo install cargo-llvm-cov
fi

_tit "generating coverage report"
# shellcheck disable=2086
INDEXER=electrum $COV $COV_OPTS "${CARGO_TEST_OPTS[@]}"
INDEXER=esplora $COV $COV_OPTS --no-clean "${CARGO_TEST_OPTS[@]}"

if [ -z "$CI" ]; then
    ## show html report location
    echo "generated html report: target/llvm-cov/html/index.html"
fi
