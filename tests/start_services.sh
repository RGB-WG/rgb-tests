#!/bin/bash
set -eu

_die () {
    echo "ERR: $*"
    exit 1
}

COMPOSE_BASE="docker compose"
if ! $COMPOSE_BASE >/dev/null; then
    echo "could not call docker compose (hint: install docker compose plugin)"
    exit 1
fi
COMPOSE_BASE="$COMPOSE_BASE -f tests/docker-compose.yml"
PROFILE=${PROFILE:-"esplora"}
COMPOSE="$COMPOSE_BASE --profile $PROFILE"
TEST_DIR="./tests/tmp"

# see docker-compose.yml for the exposed ports
if [ "$PROFILE" == "esplora" ]; then
    BCLI="$COMPOSE exec -T esplora cli"
    EXPOSED_PORTS=(8094 50002)
elif [ "$PROFILE" == "electrum" ]; then
    BCLI="$COMPOSE exec -T -u blits bitcoind bitcoin-cli -regtest"
    EXPOSED_PORTS=(50001)
else
    _die "invalid profile"
fi

# restart services (down + up) checking for ports availability
$COMPOSE_BASE --profile '*' down -v --remove-orphans
mkdir -p $TEST_DIR
for port in "${EXPOSED_PORTS[@]}"; do
    if [ -n "$(ss -HOlnt "sport = :$port")" ];then
        _die "port $port is already bound, services can't be started"
    fi
done
$COMPOSE up -d

# wait for services (pre-mining)
if [ "$PROFILE" == "esplora" ]; then
    # wait for esplora to have completed setup
    until $COMPOSE logs esplora |grep -q 'Bootstrapped 100%'; do
        sleep 1
    done
elif [ "$PROFILE" == "electrum" ]; then
    # wait for bitcoind to be up
    until $COMPOSE logs bitcoind |grep 'Bound to'; do
        sleep 1
    done
fi

# prepare bitcoin funds
$BCLI createwallet miner
$BCLI -rpcwallet=miner -generate 103

# wait for services (post-mining)
if [ "$PROFILE" == "esplora" ]; then
    # wait for esplora to have completed setup
    until $COMPOSE logs esplora |grep -q 'Electrum RPC server running'; do
        sleep 1
    done
elif [ "$PROFILE" == "electrum" ]; then
    # wait for electrs to have completed startup
    until $COMPOSE logs electrs |grep 'finished full compaction'; do
        sleep 1
    done
fi
