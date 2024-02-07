<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://static.ringsnetwork.io/ringsnetwork_logo.png">
  <img alt="Rings Network" src="https://raw.githubusercontent.com/RingsNetwork/asserts/main/logo/rings_network_red.png">
</picture>

# Rings SNARK
======================

[![rings-node](https://github.com/RingsNetwork/rings-node/actions/workflows/auto-release.yml/badge.svg)](https://github.com/RingsNetwork/rings-node/actions/workflows/auto-release.yml)
[![cargo](https://img.shields.io/crates/v/rings-node.svg)](https://crates.io/crates/rings-node)
[![docs](https://docs.rs/rings-node/badge.svg)](https://docs.rs/rings-node/latest/rings_node/)
![GitHub](https://img.shields.io/github/license/RingsNetwork/rings-node)

This create contains the implementation of Rings SNARK, which is based on Nova

### TL;DR

Rings SNARK is a SNARK proof system based on NOVA. It automates the process of transferring circuits written in `circom` language to the [bellpepper proof system](https://github.com/lurk-lab/bellpepper) used by Rings SNARK. Additionally, it leverages Nova for folding operations. This is beneficial for breaking down large zkSNARK computations and enabling parallel processing.

### Workflow

The following diagram illustrates the core logic of Rings SNARK. The top side represents the input from `circom`, including the compiled r1cs and wasm. The right side shows the circuit's input, including public and private input.


![Snark workflow](https://raw.githubusercontent.com/RingsNetwork/asserts/main/imgs/snark.png)

#### WASM Reader

Rings SNARK includes a loader module for [Circom](https://github.com/iden3/circom) Witness Calculator (Wasm). It uses wasmer for reading wasm and transforms it into a Circuit compatible with Nova standards.

#### RICS Loader

Rings SNARK can read r1cs generated by circom and convert them to the [bell pepper proof system](https://github.com/lurk-lab/bellpepper). This is based on the work of [Circom-Scotia](https://github.com/lurk-lab/circom-scotia) and further adds support for browsers.

#### Nova Folder

Through Nova, Rings SNARK performs folding computations on circuits. A set of recursive circuits can be folded into two Relax R1CS-based circuits. This allows Rings SNARK to arbitrarily split the circuit list and distribute it across nodes via the Rings Network.

#### Recursive SNARK

Rings SNARK supports most recursive circuits. It requires that the number of inputs and outputs in a circuit be equal. The output from one step becomes the input for the next, thus creating a recursive process. For circuits not originally designed for recursive SNARK, simple modifications may be necessary. For instance, auxiliary variables can be used to make inputs and outputs recursive, or internal loops can be rewritten as recursive structures. For more details, refer to the merkle tree example in the examples section.