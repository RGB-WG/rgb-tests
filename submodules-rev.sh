#!/usr/bin/env bash
#
# change all submodules in the current project to the requested branch or tag
#
# optionally use the specified remote where the branch or tag are to be found
# optionally fetch the specified remote, or all if none was specified
#
# if the specified parameters are invalid (e.g. the branch doesn't exist),
# nothing is done and execution continues to the next submodule

# colors
NC='\033[0m'        # no color
BLUE='\033[0;34m'   # blue
GREEN='\033[0;32m'  # green
ORANGE='\033[0;33m' # orange
RED='\033[0;31m'    # red

# vars
BRANCH=""                           # branch to change to
DIR="$(realpath "$(dirname "$0")")" # script path
FETCH=0                             # don't fetch by default
REMOTE=""                           # remote where to look for revs
TAG=""                              # tag to change to

# helper functions
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

# CLI handling
help() {
    echo "$0 <command>"
    echo ""
    echo "commands:"
    echo "    status         show all submodule revs"
    echo "    change         change submodule revs"
    echo "    help           show this help message"
}

change_help() {
    echo "$0 change [-h] [-f] [-r <remote>] (-b) <branch>"
    echo ""
    echo "options:"
    echo "    -b --branch    change to the specified branch"
    echo "    -f --fetch     fetch provided remote (or all if none specified)"
    echo "    -r --remote    remote to be used"
    echo "    -t --tag       change to the specified tag"
}

status() {
    git submodule foreach --quiet \
        'echo "" $(git rev-list -n1 HEAD) $sm_path $(git describe --all | sed "s,^\(tags\|remotes\|heads\)/,(,;s/$/)/")'
}

if [ -z "$1" ]; then
    help
    _die "please provide a command"
fi

case $1 in
    help)
        help
        exit 0
        ;;
    status)
        status
        exit 0
        ;;
    change)
        shift
        ;;
    *)
        help
        _die "unsupported command \"$1\""
        ;;
esac

while [ -n "$1" ]; do
    case $1 in
        -h | --help)
             change_help
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
        -t | --tag)
            TAG="$2"
            shift
            ;;
        *)
            change_help
            _die "unsupported option \"$1\""
            ;;
    esac
    shift
done

# check a branch or tag have been specified
if [ -z "$BRANCH" ] && [ -z "$TAG" ]; then
    change_help
    _die "please specify a branch or a tag to switch to"
fi
if [ -n "$BRANCH" ] && [ -n "$TAG" ]; then
    _die "please either specify a branch or a tag, not both"
fi

# optionally add the remote
if [ -n "$REMOTE" ]; then
    BRANCH="$REMOTE/$BRANCH"
fi

# change to the script directory (project root)
pushd "$DIR" >/dev/null || exit 1

# get list of submodules
if ! [ -r .gitmodules ]; then
    _die "project has no git submodules"
fi
SUBS=$(awk -F '"' '/submodule/ {print $2}' .gitmodules)

# update submodule revs
for sub in $SUBS; do
    _head "---- submodule: $sub"
    cd "$DIR/$sub" || exit 1
    # make sure the remote exists, if specified
    if [ -n "$REMOTE" ]; then
        if ! git remote | grep -xq "$REMOTE"; then
            _war "remote \"$REMOTE\" not found"
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
    if [ -n "$BRANCH" ]; then
        if ! git branch -r | grep -wq "$BRANCH"; then
            _war "branch \"$BRANCH\" not found"
            continue
        fi
        # checkout the specified branch
        git checkout "$BRANCH"
        # pull latest changes
        git pull
    fi
    if [ -n "$TAG" ]; then
        if ! git tag | grep -xq "$TAG"; then
            _war "tag \"$TAG\" not found"
            continue
        fi
        # checkout the specified tag
        git checkout "$TAG"
    fi
done

# go back to calling directory
popd >/dev/null || exit 1
