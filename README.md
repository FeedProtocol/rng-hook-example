# RNG-HOOK


This project contains an example program for Solana Transfer Hook Token Extension that makes a CPI into Feed Protocol RNG.

## Table of Contents

- [Introduction](#introduction)
- [Installation](#installation)


## Introduction

The Feed Protocol Program is a random number generating on-chain program. The program uses on-chain derived data to generate a random number and provide it to the client. [Tranfer Hook Token Extension](#https://solana.com/developers/guides/token-extensions/transfer-hook) allows to apply a custom program at every token transaction. Making a CPI into Feed Protocol Program in a Transfer Hook Program provides user a random number that can be used in the same transaction. This way user can create a random number generator embedded token. In this program how to get random number is shown. However, the uses cases are left to developers who would like to create an rng embedded token.

## Installation


1. **Install Rust**: https://rustup.rs/

2. **Install Solana CLI**: https://docs.solanalabs.com/cli/install

3. **Clone This Repository**
    ```sh
    git clone https://github.com/MintLabsDev/rng-hook-example.git
    ```

4. **Install Dependencies**
    ```sh
    cd rng-hook-example
    cargo add
    ```
5. **Build And Deploy The Program**: Ensure you have enough Sol and configured clusters
    ```sh
    cargo build-bpf
    solana program deploy target/deploy/rng-hook-example.so
    ```

6. **Look at https://solana.com/developers to see how to create token and use transfer hook extension**

