#!/usr/bin/env bash
#
# change all submodules in the current project to the requested branch
#
# optionally specify a remote where the branch is to be found
# optionally fetch the specified remote, or all of them
#
# nothing is done if the requested branch is not found

NC='\033[0m'        # no color
BLUE='\033[0;34m'   # blue
GREEN='\033[0;32m'  # green
ORANGE='\033[0;33m' # orange
RED='\033[0;31m'    # red
BRANCH=""           # branch to change to
DIR="$(pwd)"        # current working directory
FETCH=0             # don't fetch by default
REMOTE=""           # remote where to look for branches

_die() {
    printf "\n${RED}ERROR: %s${NC}\n" "$@"
    exit 1
}

_head() {
    printf "${GREEN}%s${NC}\n" "$@"
}

_log() {
    printf "${BLUE}%s${NC}\n" "$@"
}

_war() {
    printf "${ORANGE}WARNING: %s${NC}\n" "$@"
}

help() {
    echo "$0 [-h] [-f] [-r <remote>] (-b) <branch>"
    echo ""
    echo "options:"
    echo "    -h --help      show this help message"
    echo "    -b --branch    chante to the specified branch"
    echo "    -f --fetch     fetch provided remote (or all if none specified)"
    echo "    -r --remote    remote to be used"
}

# parse CLI options
while [ -n "$1" ]; do
    case $1 in
        -h | --help)
            help
            exit 0
            ;;
        -b | --branch)
            BRANCH="$2"
            shift
            ;;
        -f | --fetch)
            FETCH=1
            ;;
        -r | --remote)
            REMOTE="$2"
            shift
            ;;
        *)
            help
            _die "unsupported argument \"$1\""
            ;;
    esac
    shift
done

# check a branch has been specified
if [ -z "$BRANCH" ]; then
    _die "please specify a branch to switch to"
fi

# add remote to branch
if [ -n "$REMOTE" ]; then
    BRANCH="$REMOTE/$BRANCH"
fi

pushd "$DIR" >/dev/null || exit 1

# get list of submodules
if ! [ -r .gitmodules ]; then
    _die "project has no git submodules"
fi
SUBS=$(awk -F '"' '/submodule/ {print $2}' .gitmodules)

for sub in $SUBS; do
    _head "---- submodule: $sub"
    cd "$sub" || exit 1
    # make sure the remote exists, if specified
    if [ -n "$REMOTE" ]; then
        if ! git remote | grep "$REMOTE" >/dev/null; then
            _war "$REMOTE remote not found"
            cd ..
            continue
        fi
    fi
    # update repo
    if [ "$FETCH" = 1 ]; then
        if [ -n "$REMOTE" ]; then
            git fetch "$REMOTE"
        else
            git fetch --all
        fi
    fi
    # check if specified branch exists
    if ! git branch -r | grep "$BRANCH" >/dev/null; then
        _war "branch $BRANCH not found"
        cd ..
        continue
    fi
    # checkout the specified branch
    git checkout "$BRANCH"
    # pull latest changes
    git pull
    cd ..
done

popd >/dev/null || exit 1
