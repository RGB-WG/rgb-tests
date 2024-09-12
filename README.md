# RGB tests

This repository contains tests for RGB.

## Requirements
- Linux OS
- Docker with its `compose` plugin
- Rust

## Run
Clone the project, including submodules:

```sh
git clone https://github.com/RGB-WG/rgb-tests --recurse-submodules
```

Note: after checking out to another commit, remember to run:
```sh
git submodule update
```

Then, from the project root, run the tests by running:
```sh
cargo test
```

:warning: **Warning:** if your machine has a lot of CPU cores, it could
happen that calls to indexers fail because of too many parallel requests. To
limit the test threads and avoid this issue set the `--test-threads` option
(e.g. `cargo test -- --test-threads=8`).

### Test services

Test services will be automatically (re)started by the test command and will
run in docker containers.
If you don't have the docker images they will be automatically pulled. Note
that in this case the first test execution will be slower.
Also note that there's no automatic shutdown of test services, you'll need to
manually remove the docker containers with:
```sh
docker compose -f tests/docker-compose.yml --profile='*' down -v --remove-orphans
```

The indexer used by the tests is configurable, currently esplora and electrum
are supported. You can change the indexer type by setting the `INDEXER`
environment variable, for example:
```sh
INDEXER=esplora cargo test  # default
INDEXER=electrum cargo test
```

If you are developing new tests and want a faster execution, you can set
`SKIP_INIT=1` to avoid restarting the test services. Please note that you
cannot switch to another indexer when using this option, you'll have to use the
same indexer type from the previous test execution.

### Coverage

To run the tests and generate a code coverage report run:
```sh
./tests/coverage.sh
```
This will generate a report in `target/llvm-cov/html/index.html` that you can
visualize on a browser (e.g. `firefox target/llvm-cov/html/index.html`).

Coverage will be measured for all patched crates.

The GitHub organizations of submodule repositories are:
- https://github.com/RGB-WG
- https://github.com/LNP-BP
- https://github.com/BP-WG
- https://github.com/aluvm
- https://github.com/strict-types
- https://github.com/rust-amplify
- https://github.com/UBIDECO
