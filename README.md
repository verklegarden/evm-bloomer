<img align="right" width="150" height="150" top="100" src="./assets/logo.png">

# evm-bloomer • ![Apache2/MIT licensed][license-shield]

> An attempt to orchestrate the many blooms of the EVM

The `evm-bloomer` is a tool to analyse chains' EVM versions.

> [!WARNING]
> Does not work for many rpcs and may be unreliable.
>
> Released to gather feedback. Not production ready!

## Usage

```sh
evm-bloomer --rpc-urls [RPC_URL...]
```

Example:
```sh
$ evm-bloomer --rpc-urls $rpc_oeth | jq
> {
>   "blooms": [
>     {
>       "bloom": "0xfff0fffc8000ffffffe0fffffffffffffffffffff8000000000000000000fc27",
>       "chain_id": 10,
>       "unknown_opcodes": [],
>       "version": "cancun"
>     }
>   ]
> }
```

## Requesting a new Chain

To request a new chain please open an issue.

Test rpc url via:
```sh
cargo run -- -r "$RPC_URL" | jq
```

## Background

While there are many different EVM chains it is wrong to assume they all run the same EVM version.
Most chains except Ethereum run older EVM versions, some include custom opcodes, and some removed
"random" opcodes.

This inconsistency makes it dangerous to deploy smart contracts on multiple chains without auditing
the chain's EVM. Further, most chains do not properly document incompatibilities.

Note that in a smart contract's bytecode there is no distinction between code and data making it
very hard to analyse it regarding unsupported opcodes.

> [!NOTE]
> It is trivially possible to deploy a contract compiled for evm version X on a chain running
> EVM version Y. If an unsupported opcode is executed by the contract __the EVM will revert__.
> This can lead to smart contracts not working as intended, tested, and audited.

## EVM Blooms

The maximum number of opcodes the EVM can support (assuming no "opcode extension hacks")
256 because an opcode is of type `uint8`.

This means an EVM's opcode support can be encoded in a 256 bitmap or `bloom`.

A similar project, [`OpcodesBitmap`](https://github.com/AmadiMichael/OpcodesBitmap/blob/main/src/OpcodesBitmap.sol) created by Michael Amadi, aimed at making an EVM's `bloom` available onchain.

## Bloom Creation

`evm-bloomer` creates a chain's bloom from its rpc url by simulating a contract deployment whose
bytecode is a single opcode. If the rpc's EVM does not support the opcode the deployment fails,
otherwise it either succeeds or returns a known error (eg `stack underflow`).

## Contributing

All contributions are highly welcome!

Feel free to create a PR, an issue to discuss ideas, or reach out privately to [merkleplant](https://merkleplant.xyz).

## License

Licensed under either of <a href="LICENSE-APACHE">Apache License, Version 2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

<!--- Shields -->
[license-shield]: https://img.shields.io/badge/license-Apache2.0/MIT-blue.svg
