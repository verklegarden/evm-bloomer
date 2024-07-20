<img align="right" width="150" height="150" top="100" src="./assets/logo.png">

# EVMBloom

> An attempt to orchestrate the many blooms of the EVM

## tl;dr

- The Ethereum virtual machine supports a total of 256 possible opcodes
- The set of supported opcodes for an EVM instance can be encoded in a 256 bit bloom filter
- These _EVM blooms_ enable:
    - simple version matching
    - easy comparison
    - beautiful visualizations

## TODOs

- Cooler visualization, should look more like a flower
- Bloom to opcode mapping, eg show mnemonic if possible
- Bulk run, ie check list of rpc urls, and serialize to json with `(chain_id, bloom)`
- Deserialize json file and verify if still valid
    - Last two points important for automated monitoring of evm updates on chains
