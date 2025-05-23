<div align="center" style="width: 100%;">
  <img 
    src="https://expander.polyhedra.network/assets/static/logo-with-text.16d5af29.svg" 
    alt="Expander Logo"
    style="width: 400px; height: auto;"
  />
</div>

# Expander

[![Build](https://github.com/PolyhedraZK/Expander/actions/workflows/ci.yml/badge.svg)](https://github.com/PolyhedraZK/Expander/actions)
[![License](https://img.shields.io/github/license/PolyhedraZK/Expander)](https://github.com/PolyhedraZK/Expander/blob/main/LICENSE)

<div align="center">
  <h3>
    <a href="https://eprint.iacr.org/2019/317">
      Paper
    </a>
    <span> | </span>
    <a href="https://polyhedrazk.github.io/benchmark-pages/">
      Benchmarks
    </a>
    <span> | </span>
    <a href="https://github.com/PolyhedraZK/ExpanderCompilerCollection">
      Your Code Compiler
    </a>
    <span> | </span>
    <a href="https://t.me/+XEdEEknIdaI0YjEx">
      Telegram Group
    </a>
  </h3>
</div>

Expander is a proof generation backend for Polyhedra Network. It aims to support fast proof generation.

For more technical introduction, visit our markdown files [here](https://github.com/PolyhedraZK/Expander-cpp/tree/master/docs/doc.md).

And [here](./gkr/src/tests/gkr_correctness.rs) for an example on how to use the gkr lib.

This is a core repo for our prover, to write circuits on our prover, please visit [our compiler](https://github.com/PolyhedraZK/ExpanderCompilerCollection)

## Developer helper

We understand that the product is currently in development and may not be very user-friendly yet. We encourage developers to join our Telegram chat group for Q&A: https://t.me/+XEdEEknIdaI0YjEx

Additionally, please take a look at our circuit compiler: https://github.com/PolyhedraZK/ExpanderCompilerCollection

This compiler is your entry point for using our prover; the repository you have is primarily the core executor, not the developer frontend. Our product pipeline is as follows:

`Your circuit code -> Expander Compiler -> circuit.txt & witness.txt -> Expander -> proof `

Please note that the witness generation process is not yet optimal, and we are actively working on improving it.

## AVX
We use AVX2 by default. On an x86 or a mac, you can simply do
```
RUSTFLAGS="-C target-cpu=native" cargo test --release --workspace
```
For some platforms, if you do not indicate `target-cpu=native` it may simulate avx2 instructions, rather than use it directly, and this will cause performance decrease.

Our code also supports `avx512`. This is not turned on by default. To use `avx512`
```
RUSTFLAGS="-C target-cpu=native -C target-feature=+avx512f" cargo test --release --workspace
```

## Environment Setup

Before executing setup, please make sure you read through the system requirements, and make sure your CPU is in the list.

```sh
cargo run --bin=dev-setup --release
```


## Benchmarks

**Make sure you include `RUSTFLAGS="-C target-cpu=native"` to allow platform specific accelerations.**

Command template:

```sh
RUSTFLAGS="-C target-cpu=native" cargo run --release --bin gkr -- -f [fr|m31ext3] -t [#threads] -c [keccak|poseidon]
```

Concretely if you are running on a 16 physical core CPU for Bn256 scalar field:

```sh
RUSTFLAGS="-C target-cpu=native" cargo run --release --bin gkr -- -f fr -t 16
```

## Correctness test

[Here](./gkr/src/tests/gkr_correctness.rs) we provide a test case for end-to-end proof generation and verification.
To check the correctness, run the follow standard Rust test command:

```sh
RUSTFLAGS="-C target-cpu=native" cargo test --release -- --nocapture
```

## CLI

Usage:

```sh
RUSTFLAGS="-C target-cpu=native" cargo run --bin expander-exec --release -- prove -c <circuit_file> -w <witness_file> -o <output_proof_file>
RUSTFLAGS="-C target-cpu=native" cargo run --bin expander-exec --release -- verify -c <circuit_file> -w <witness_file> -i <input_proof_file>
RUSTFLAGS="-C target-cpu=native" cargo run --bin expander-exec --release -- serve -c <circuit_file> -h <host_ip> -p <port>
```

Example:

```sh
RUSTFLAGS="-C target-cpu=native" mpiexec -n 1 cargo run --bin expander-exec --release -- prove -c ./data/circuit_m31.txt -w ./data/witness_m31.txt -o ./data/out_m31.bin
RUSTFLAGS="-C target-cpu=native" mpiexec -n 1 cargo run --bin expander-exec --release -- verify -c ./data/circuit_m31.txt -w ./data/witness_m31.txt -i ./data/out_m31.bin
RUSTFLAGS="-C target-cpu=native" mpiexec -n 1 cargo run --bin expander-exec --release -- serve -c ./data/circuit_m31.txt -h 127.0.0.1 -p 3030
```

To change the hash function used in the fiat-shamir transform,  use`-f [SHA256|Poseidon|MiMC5]`. To change the polynomial commitment scheme, use `-p [Raw|Orion|Hyrax|KZG]`. These options are placed before the `prove/verify` command, for example:
```sh
RUSTFLAGS="-C target-cpu=native" cargo run --bin expander-exec --release -- -f SHA256 -p Raw prove -c <circuit_file> -w <witness_file> -o <output_proof_file>
```
Note that the hash function and the polynomial commitment scheme should be the same in the process of proving and verifying, otherwise the verification would fail.

To test the service started by `expander-exec serve`, you can use the following command:
```sh
python ./scripts/test_http.py  # need "requests" package
```

## Profiling
To get more fine-grained information about the running time, you can enable the `gkr/profile` feature, i.e.

```sh
RUSTFLAGS="-C target-cpu=native" cargo run --bin expander-exec --release --features gkr/profile -- prove -c ./data/circuit_m31.txt -w ./data/witness_m31.txt -o ./data/out_m31.bin
```

Note that enabling the `profile` feature will slightly reduce the overall performance so it is recommended not to enable it when benchmarking.

## How to contribute?

Thank you for your interest in contributing to our project! We seek contributors with a robust background in cryptography and programming, aiming to improve and expand the capabilities of our proof generation system.

### Contribution Guidelines:

#### Pull Requests

We welcome your pull requests (PRs) and ask that you follow these guidelines to facilitate the review process:

- **General Procedure**:

  1. **Fork the repository** and clone it locally.
  2. **Create a branch** for your changes related to a specific issue or improvement.
  3. **Commit your changes**: Use clear and meaningful commit messages.
  4. **Push your changes** to your fork and then **submit a pull request** to the main repository.

- **PR Types and Specific Guidelines**:
  - **[BUG]** for bug fixes:
    - **Title**: Start with `[BUG]` followed by a brief description.
    - **Content**: Explain the issue being fixed, steps to reproduce, and the impact of the bug. Include any relevant error logs or screenshots.
    - **Tests**: Include tests that ensure the bug is fixed and will not recur.
  - **[FEATURE]** for new features:
    - **Title**: Start with `[FEATURE]` followed by a concise feature description.
    - **Content**: Discuss the benefits of the feature, possible use cases, and any changes it introduces to existing functionality.
    - **Documentation**: Update relevant documentation and examples.
    - **Tests**: Add tests that cover the new feature's functionality.
  - **[DOC]** for documentation improvements:
    - **Title**: Start with `[DOC]` and a short description of what is being improved.
    - **Content**: Detail the changes made and why they are necessary, focusing on clarity and accessibility.
  - **[TEST]** for adding or improving tests:
    - **Title**: Begin with `[TEST]` and describe the type of testing enhancement.
    - **Content**: Explain what the tests cover and how they improve the project's reliability.
  - **[PERF]** for performance improvements:
    - **Title**: Use `[PERF]` and a brief note on the enhancement.
    - **Content**: Provide a clear comparison of performance before and after your changes, including benchmarks or profiling data.
    - **Tests/Benchmarks**: Add tests that cover the new feature's functionality, and benchmarks to prove your improvement.

#### Review Process

Each pull request will undergo a review by one or more core contributors. We may ask for changes to better align with the project's goals and standards. Once approved, a maintainer will merge the PR.

We value your contributions greatly and are excited to see what you bring to this project. Let’s build something great together!

## Acknowledgements
We would like to thank the following projects and individuals:

1. [Gnark](https://github.com/Consensys/gnark): for their exceptional frontend circuit language.
2. [Plonky2&3](https://github.com/Plonky3/Plonky3): for their inspiring work on Merseene prime AVX and ARM-Neon assembly implementation.
3. [Justin Thaler](https://people.cs.georgetown.edu/jthaler/): for pointing out the soundness issue of using repetition.
3. [Stwo](https://github.com/starkware-libs/stwo): for inspiring us to make the benchmark page.
4. [Intel](https://www.intel.com/content/dam/develop/external/us/en/documents/clmul-wp-rev-2-02-2014-04-20.pdf): for their detailed implementation of GF(2^128) field multiplication.
