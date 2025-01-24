#!/bin/bash

# Script to generate an evm-bloomer daily report.

rpcs=(
    "https://eth-mainnet.g.alchemy.com/v2/${ALCHEMY_API_KEY}"          #     1 - Ethereum
    "https://opt-mainnet.g.alchemy.com/v2/${ALCHEMY_API_KEY}"          #    10 - Optimism
    "https://zksync-mainnet.g.alchemy.com/v2/${ALCHEMY_API_KEY}"       #   324 - zkSync
    "https://polygonzkevm-mainnet.g.alchemy.com/v2/${ALCHEMY_API_KEY}" #  1101 - Polygon zkEVM
    "https://arb-mainnet.g.alchemy.com/v2/${ALCHEMY_API_KEY}"          # 42161 - Arbitrum One
    "https://arbnova-mainnet.g.alchemy.com/v2/${ALCHEMY_API_KEY}"      # 42170 - Arbitrum Nova
)

# Generate report
report="reports/report_$(date -u +"%Y-%m-%d").json"
cargo run -- -r "${rpcs[@]}" | jq > "$report"

# Debugging
echo "" && cat "$report"
