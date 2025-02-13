#!/usr/bin/env bash
#
# utility script to run and command regtest services
#

# Get the directory where the script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

name="./$(basename "$0")"

COMPOSE="docker compose -p rgb-tests"
if ! docker compose version >/dev/null; then
    echo "could not call docker compose (hint: install docker compose plugin)"
    exit 1
fi

# Bitcoin CLI commands for each node
BITCOIN_CLI_1="$COMPOSE exec bitcoin-core-1 bitcoin-cli -regtest"
BITCOIN_CLI_2="$COMPOSE exec bitcoin-core-2 bitcoin-cli -regtest"
BITCOIN_CLI_3="$COMPOSE exec bitcoin-core-3 bitcoin-cli -regtest"

# Node IP addresses
NODE2_IP="172.30.2.205"
NODE3_IP="172.30.2.206"

INITIAL_BLOCKS=103
WALLET_NAME="miner"

# Load environment variables from .env file
if [ -f .env ]; then
    export $(grep -v '^#' .env | xargs)
else
    echo "ERR: .env file not found"
    exit 1
fi

_die () {
    echo "ERR: $*"
    exit 1
}

_create_or_load_wallet() {
    local cli_cmd=$1
    local node_num=$2
    
    echo "Setting up wallet for node $node_num..."
    
    # Check if wallet is already loaded
    if $cli_cmd listwallets 2>/dev/null | grep -q "\"$WALLET_NAME\""; then
        echo "Wallet already loaded for node $node_num"
        return 0
    fi
    
    # Check if wallet file exists
    if $cli_cmd listwalletdir 2>/dev/null | grep -q "\"$WALLET_NAME\""; then
        echo "Loading existing wallet for node $node_num"
        if $cli_cmd loadwallet "$WALLET_NAME" 2>/dev/null; then
            echo "Successfully loaded wallet for node $node_num"
            return 0
        fi
    else
        echo "Creating new wallet for node $node_num"
        if $cli_cmd createwallet "$WALLET_NAME" 2>/dev/null; then
            echo "Successfully created wallet for node $node_num"
            return 0
        fi
    fi
    
    echo "Failed to create or load wallet for node $node_num"
    return 1
}

# Function to check if a port is in use
_check_port_in_use() {
    local port=$1
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        if [ -n "$(netstat -an | grep LISTEN | grep ".$port")" ]; then
            return 0
        else
            return 1
        fi
    else
        # Linux
        if [ -n "$(ss -HOlnt "sport = :$port")" ]; then
            return 0
        else
            return 1
        fi
    fi
}

_connect_nodes() {
    # Ensure nodes 2 and 3 are connected to each other
    echo "Establishing connection between nodes..."
    
    # Wait for nodes to be fully started and listening
    sleep 5
    
    # Clear existing connections
    $BITCOIN_CLI_2 clearbanned
    $BITCOIN_CLI_3 clearbanned
    
    # Use 'add' instead of 'onetry' for persistent connections
    echo "Adding peer connections..."
    $BITCOIN_CLI_2 addnode "$NODE3_IP:18444" "onetry"
    $BITCOIN_CLI_3 addnode "$NODE2_IP:18444" "onetry"
}

_wait_for_sync() {
    echo "Waiting for block synchronization..."
    local max_attempts=30
    local attempt=1
    
    while [ $attempt -le $max_attempts ]; do
        local height_2=$($BITCOIN_CLI_2 getblockcount)
        local height_3=$($BITCOIN_CLI_3 getblockcount)
        
        if [ "$height_2" = "$height_3" ]; then
            echo "Nodes synchronized at height $height_2"
            return 0
        fi
        
        echo "Attempt $attempt: Node 2 height: $height_2, Node 3 height: $height_3"
        sleep 2
        attempt=$((attempt + 1))
    done
    
    echo "Warning: Synchronization failed after $max_attempts attempts"
    return 1
}

_check_network_status() {
    echo "Checking network status..."
    
    # Check node 2's network information
    echo "Node 2 connections:"
    $BITCOIN_CLI_2 getpeerinfo | grep "addr\|subver"
    
    # Check node 3's network information
    echo "Node 3 connections:"
    $BITCOIN_CLI_3 getpeerinfo | grep "addr\|subver"
}

_wait_for_bitcoin_ready() {
    local cli_cmd=$1
    local node_num=$2
    local max_attempts=30
    local attempt=1

    echo "Waiting for node $node_num to be ready..."
    while [ $attempt -le $max_attempts ]; do
        if $cli_cmd getblockchaininfo >/dev/null 2>&1; then
            # Wait a bit more to ensure wallet system is also ready
            sleep 2
            echo "Node $node_num is ready"
            return 0
        fi
        echo "Waiting for node $node_num... (attempt $attempt/$max_attempts)"
        sleep 2
        attempt=$((attempt + 1))
    done

    echo "Timeout waiting for node $node_num"
    return 1
}

_start_services() {
    _stop_services

    # Check exposed ports
    EXPOSED_PORTS=(
        $BITCOIN_RPC_PORT_1 $ELECTRUM_PORT_1 $API_PORT_1
        $BITCOIN_RPC_PORT_2 $ELECTRUM_PORT_2 $API_PORT_2
        $BITCOIN_RPC_PORT_3 $ELECTRUM_PORT_3 $API_PORT_3
    )
    for port in "${EXPOSED_PORTS[@]}"; do
        if _check_port_in_use "$port"; then
            _die "port $port is already bound, services can't be started"
        fi
    done

    echo "Starting services..."
    $COMPOSE up -d

    echo "Waiting for services to start..."
    sleep 10

    # Wait for all nodes to be ready
    _wait_for_bitcoin_ready "$BITCOIN_CLI_1" "1" || _die "Node 1 failed to start"
    _wait_for_bitcoin_ready "$BITCOIN_CLI_2" "2" || _die "Node 2 failed to start"
    _wait_for_bitcoin_ready "$BITCOIN_CLI_3" "3" || _die "Node 3 failed to start"

    # Create or load wallets
    echo "Setting up wallets..."
    _create_or_load_wallet "$BITCOIN_CLI_1" "1" || _die "Failed to setup wallet for node 1"
    _create_or_load_wallet "$BITCOIN_CLI_2" "2" || _die "Failed to setup wallet for node 2"
    _create_or_load_wallet "$BITCOIN_CLI_3" "3" || _die "Failed to setup wallet for node 3"

    # Node 1 (isolated network)
    echo "Generating initial blocks for node 1..."
    $BITCOIN_CLI_1 -rpcwallet="$WALLET_NAME" -generate $INITIAL_BLOCKS > /dev/null
    
    # Node 2 & 3 (shared network)
    # Establish and verify node connections
    _connect_nodes
    
    # Generate blocks on node 2
    echo "Generating blocks on node 2..."
    $BITCOIN_CLI_2 -rpcwallet="$WALLET_NAME" -generate $INITIAL_BLOCKS > /dev/null
    
    # Wait for synchronization
    _wait_for_sync

    # Verify block heights are the same for nodes 2 and 3
    HEIGHT_2=$($BITCOIN_CLI_2 getblockcount)
    HEIGHT_3=$($BITCOIN_CLI_3 getblockcount)
    
    if [ "$HEIGHT_2" = "$HEIGHT_3" ]; then
        echo "Node 2 and Node 3 are synchronized at height $HEIGHT_2"
    else
        echo "Warning: Node 2 ($HEIGHT_2) and Node 3 ($HEIGHT_3) heights differ"
    fi

    # Wait for Esplora services to start
    echo "Waiting for Esplora services to start..."
    sleep 10

    echo "Setup completed successfully"
}

_stop_services() {
    echo "Stopping services..."
    $COMPOSE down
}

_clean_environment() {
    echo "Cleaning up environment..."
    # Stop all services
    $COMPOSE down --remove-orphans
    
    # Remove all related docker volumes
    echo "Removing volumes..."
    docker volume ls -q | grep "rgb-tests" | xargs -r docker volume rm
    
    # Remove related docker networks
    echo "Removing networks..."
    docker network ls --filter name=rgb-tests -q | xargs -r docker network rm
    
    # Remove wallet data directory
    echo "Removing wallet data..."
    rm -rf data{core,index,ldk0,ldk1,ldk2}
    
    echo "Environment cleaned successfully"
}

_mine() {
    local node=$1
    local blocks=$2
    case $node in
        1)
            $BITCOIN_CLI_1 -rpcwallet="$WALLET_NAME" -generate $blocks > /dev/null
            ;;
        2)
            $BITCOIN_CLI_2 -rpcwallet="$WALLET_NAME" -generate $blocks > /dev/null
            ;;
        3)
            $BITCOIN_CLI_3 -rpcwallet="$WALLET_NAME" -generate $blocks > /dev/null
            ;;
        *)
            _die "Invalid node number: $node"
            ;;
    esac
}

_help() {
    echo "$name [-h|--help]"
    echo "    show this help message"
    echo
    echo "$name start"
    echo "    stop services, clean up, start services,"
    echo "    create bitcoind wallets and generate initial blocks"
    echo
    echo "$name stop"
    echo "    stop services (preserving data)"
    echo
    echo "$name clean"
    echo "    clean up environment (remove volumes, networks)"
    echo
    echo "$name status"
    echo "    check network connection status"
    echo
    echo "$name mine <node> <blocks>"
    echo "    mine the requested number of blocks on specified node (1, 2, or 3)"
}

# cmdline arguments
[ -z "$1" ] && _help
while [ -n "$1" ]; do
    case $1 in
        -h|--help)
            _help
            exit 0
            ;;
        start)
            start=1
            ;;
        stop)
            stop=1
            ;;
        clean)
            clean=1
            ;;
        status)
            _check_network_status
            ;;
        mine)
            [ -n "$2" ] || _die "node number is required"
            [ -n "$3" ] || _die "num blocks is required"
            NODE_NUM="$2"
            NUM_BLOCKS="$3"
            mine=1
            shift 2
            ;;
        *)
            _die "unsupported argument \"$1\""
            ;;
    esac
    shift
done

# start services if requested
[ "$start" = "1" ] && _start_services

# stop services if requested
[ "$stop" = "1" ] && _stop_services

# clean environment if requested
[ "$clean" = "1" ] && _clean_environment

# mine blocks if requested
[ "$mine" = "1" ] && _mine $NODE_NUM $NUM_BLOCKS

exit 0
