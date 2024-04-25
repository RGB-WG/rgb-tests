# RGB integration tests

Clone the project, including submodules:

```sh
git clone https://github.com/RGB-WG/rgb-integration-tests --recurse-submodules
```

Then, from the project root, run the integration tests by running:
```sh
cargo test
```

To run the tests and generate a code coverage report run:
```sh
./tests/coverage.sh
```
This will generate a report in `target/llvm-cov/html/index.html` that you can
visualize on a browser (e.g. `firefox target/llvm-cov/html/index.html`).

GitHub organizations of submodule repositories:
- https://github.com/RGB-WG
- https://github.com/LNP-BP
- https://github.com/BP-WG
- https://github.com/aluvm
- https://github.com/strict-types
- https://github.com/rust-amplify
- https://github.com/UBIDECO
