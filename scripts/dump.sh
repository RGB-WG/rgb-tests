#!/usr/bin/env bash

cargo +stable build --manifest-path=rgb/cli/Cargo.toml --all-features --bin rgb --release

cd tests/tmp
for dir in */; do
  dir_name=${dir%/}

  if [[ $dir_name =~ ^[0-9a-fA-F]{8}$ ]]; then
    echo "Processing $dir_name..."
    mkdir $dir_name/testnet3;
    mv $dir_name/*.dat $dir_name/wallet_name $dir_name/testnet3;
    ../../rgb/target/release/rgb -d $dir_name dump $dir_name/dump
  fi
done
