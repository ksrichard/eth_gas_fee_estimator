Rust Ethereum Gas Fee Estimator HTTP server
---

This HTTP server serves a single endpoint that estimates the gas cost of the given transaction.

The estimation itself is based on a local calculation, so all the application does which could be longer is an external call 
that is made in the background periodically to get current base fee from ethereum network.

Because of this, the response time is really-really fast, so in case of a simple contract creation it takes around `2-4ms`
 locally to have an estimation.

## Prerequisites

1. **Ethereum JSON-RPC access**

    In order to have always the latest base fee/gas price, you will need to have an `Ethereum compatible JSON-RPC HTTP endpoint`.

    To get one, you can use [MetaMask Developer site](https://developer.metamask.io/register) to have simple access to ethereum network
through API calls.

    After registration and API key creation, you will have URLs accessible like this: `https://mainnet.infura.io/v3/<YOUR_API_KEY>`

    These URLs can be used by simply passing to the server via CLI argument (and you can change the URL to get estimation based on other Ethereum network).

2. **Rust**

   This project is fully written in `Rust`, so in order to compile and run, you will need [rust installed](https://www.rust-lang.org/tools/install) on your machine.

## Build & Run

1. Compile the project (in project root)
```shell
cargo build --release
```
2. Run the server
```shell
./target/release/eth_gas_fee_estimator --eth-json-rpc-client-url=https://mainnet.infura.io/v3/<YOUR_API_KEY>
```

### Options
```shell
./target/release/eth_gas_fee_estimator --help
Usage: eth_gas_fee_estimator [OPTIONS] --eth-json-rpc-client-url <ETH_JSON_RPC_CLIENT_URL>

Options:
  -p, --port <PORT>
          HTTP port where API is exposed [default: 9999]
  -u, --eth-json-rpc-client-url <ETH_JSON_RPC_CLIENT_URL>
          Ethereum client JSON-RPC URL. Example: https://mainnet.infura.io/v3/<YOUR_API_KEY>
  -h, --help
          Print help
  -V, --version
          Print version
```

## Transaction type support

Gas cost estimation supports `Legacy`, `EIP-2930` and `EIP-1559` transactions. 

## Test

To test the estimations you can call the `/estimate` HTTP endpoint on the server.

**Please note** that all the numeric values are encoded as `hex strings` and response value is in `WEI`!

### Request JSON structures (as Rust representations)

**Note:** if action is `Call` then the following scheme works to set that in JSON:
```json
"action": {"Call": "0x95222290dd7278aa3ddd389cc1e1d165cc4bafe5"},
```

#### Legacy
```rust
pub enum TransactionAction {
	Call(H160),
	Create,
}

pub struct LegacyTransaction {
    pub gas_price: U256,
    pub gas_limit: U256,
    pub input: String,
    pub action: TransactionAction,
}
```

#### EIP-2930
```rust
pub enum TransactionAction {
	Call(H160),
	Create,
}

pub type AccessList = Vec<AccessListItem>;

pub struct AccessListItem {
	pub address: Address,
	pub storage_keys: Vec<H256>,
}

pub struct EIP2930Transaction {
    pub gas_price: U256,
    pub gas_limit: U256,
    pub input: String,
    pub action: TransactionAction,
    pub access_list: AccessList,
}
```

#### EIP-1559
```rust
pub enum TransactionAction {
	Call(H160),
	Create,
}

pub type AccessList = Vec<AccessListItem>;

pub struct AccessListItem {
	pub address: Address,
	pub storage_keys: Vec<H256>,
}

pub struct EIP1559Transaction {
    pub max_priority_fee_per_gas: U256,
    pub max_fee_per_gas: U256,
    pub gas_limit: U256,
    pub input: String,
    pub action: TransactionAction,
    pub access_list: AccessList,
}
```

### Examples

In the examples (input field acts as data field), the following smart contract was compiled to bytecode:
```solidity
// SPDX-License-Identifier: GPL-3.0

pragma solidity >=0.8.2 <0.9.0;

/**
 * @title Storage
 * @dev Store & retrieve value in a variable
 */
contract Storage {

    uint256 number;

    /**
     * @dev Store value in variable
     * @param num value to store
     */
    function store(uint256 num) public {
        number = num;
    }

    /**
     * @dev Return value 
     * @return value of 'number'
     */
    function retrieve() public view returns (uint256){
        return number;
    }
}
```

#### Legacy transaction
```shell
curl --location 'http://127.0.0.1:9999/estimate' \
--header 'Content-Type: application/json' \
--data '{
    "Legacy": {
        "gas_price": "0xA",
        "gas_limit": "0x30D40",
        "action": "Create",
        "value": "0x0",
        "input": "6080604052348015600e575f80fd5b506101438061001c5f395ff3fe608060405234801561000f575f80fd5b5060043610610034575f3560e01c80632e64cec1146100385780636057361d14610056575b5f80fd5b610040610072565b60405161004d919061009b565b60405180910390f35b610070600480360381019061006b91906100e2565b61007a565b005b5f8054905090565b805f8190555050565b5f819050919050565b61009581610083565b82525050565b5f6020820190506100ae5f83018461008c565b92915050565b5f80fd5b6100c181610083565b81146100cb575f80fd5b50565b5f813590506100dc816100b8565b92915050565b5f602082840312156100f7576100f66100b4565b5b5f610104848285016100ce565b9150509291505056fea26469706673582212209a0dd35336aff1eb3eeb11db76aa60a1427a12c1b92f945ea8c8d1dfa337cf2264736f6c634300081a0033"
    }
}'
```

Example Response:

```json
{
    "estimated_fee_wei": "0x3f9da16276800",
    "error": null
}
```

---

#### EIP-2930 transaction
```shell
curl --location 'http://127.0.0.1:9999/estimate' \
--header 'Content-Type: application/json' \
--data '{
    "EIP2930": {
        "gas_price": "0xA",
        "gas_limit": "0x30D40",
        "action": "Create",
        "value": "0x0",
        "access_list": [
            {
                "address": "0x4838b106fce9647bdf1e7877bf73ce8b0bad5f97",
                "storageKeys": []
            }
        ],
        "input": "6080604052348015600e575f80fd5b506101438061001c5f395ff3fe608060405234801561000f575f80fd5b5060043610610034575f3560e01c80632e64cec1146100385780636057361d14610056575b5f80fd5b610040610072565b60405161004d919061009b565b60405180910390f35b610070600480360381019061006b91906100e2565b61007a565b005b5f8054905090565b805f8190555050565b5f819050919050565b61009581610083565b82525050565b5f6020820190506100ae5f83018461008c565b92915050565b5f80fd5b6100c181610083565b81146100cb575f80fd5b50565b5f813590506100dc816100b8565b92915050565b5f602082840312156100f7576100f66100b4565b5b5f610104848285016100ce565b9150509291505056fea26469706673582212209a0dd35336aff1eb3eeb11db76aa60a1427a12c1b92f945ea8c8d1dfa337cf2264736f6c634300081a0033"
    }
}'
```

Example response:
```json
{
    "estimated_fee_wei": "0x40fae05a0e800",
    "error": null
}
```

---


#### EIP-1559 transaction
```shell
curl --location 'http://127.0.0.1:9999/estimate' \
--header 'Content-Type: application/json' \
--data '{
    "EIP1559": {
        "max_priority_fee_per_gas": "0xA",
        "max_fee_per_gas": "0x2C4CDD88",
        "gas_limit": "0x30D40",
        "action": "Create",
        "value": "0x0",
        "access_list": [
            {
                "address": "0x4838b106fce9647bdf1e7877bf73ce8b0bad5f97",
                "storageKeys": []
            }
        ],
        "input": "6080604052348015600e575f80fd5b506101438061001c5f395ff3fe608060405234801561000f575f80fd5b5060043610610034575f3560e01c80632e64cec1146100385780636057361d14610056575b5f80fd5b610040610072565b60405161004d919061009b565b60405180910390f35b610070600480360381019061006b91906100e2565b61007a565b005b5f8054905090565b805f8190555050565b5f819050919050565b61009581610083565b82525050565b5f6020820190506100ae5f83018461008c565b92915050565b5f80fd5b6100c181610083565b81146100cb575f80fd5b50565b5f813590506100dc816100b8565b92915050565b5f602082840312156100f7576100f66100b4565b5b5f610104848285016100ce565b9150509291505056fea26469706673582212209a0dd35336aff1eb3eeb11db76aa60a1427a12c1b92f945ea8c8d1dfa337cf2264736f6c634300081a0033"
    }
}'
```

Example response:
```json
{
    "estimated_fee_wei": "0x43b348c34ce1a",
    "error": null
}
```


