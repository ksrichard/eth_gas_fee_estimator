use evm_disassembler::Opcode;
use evm_gasometer::Gasometer;
use evm_runtime::{Config, ExitError};
use hex::FromHexError;
use log::info;
use primitive_types::{H160, H256};
use thiserror::Error;

use crate::fee_estimator::CONTRACT_CREATION_GAS;

use super::{Transaction, BASE_GAS_COUNT};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Gasometer error: {0:?}")]
    GasometerExit(ExitError),
    #[error("EVM disassembler error: {0}")]
    EvmDisassembler(#[from] eyre::Report),
    #[error("Hex decode error: {0}")]
    HexDecode(#[from] FromHexError),
}

pub struct GasUsedEstimator {
    config: Config,
    gas_limit: u64,
}

impl GasUsedEstimator {
    pub fn new(config: Config, gas_limit: u64) -> Self {
        Self { config, gas_limit }
    }

    pub fn estimate(&self, transaction: Transaction) -> Result<u64, Error> {
        // extract transaction details for calculation
        let transaction_input = match &transaction {
            Transaction::Legacy(tx) => tx.input.clone(),
            Transaction::EIP2930(tx) => tx.input.clone(),
            Transaction::EIP1559(tx) => tx.input.clone(),
        };
        let transaction_action = match &transaction {
            Transaction::Legacy(tx) => tx.action,
            Transaction::EIP2930(tx) => tx.action,
            Transaction::EIP1559(tx) => tx.action,
        };
        let transaction_access_list = match &transaction {
            Transaction::Legacy(_) => {
                vec![]
            }
            Transaction::EIP2930(tx) => tx.access_list.clone(),
            Transaction::EIP1559(tx) => tx.access_list.clone(),
        }
        .iter()
        .map(|item| (item.address, item.storage_keys.clone()))
        .collect::<Vec<(H160, Vec<H256>)>>();

        let mut gasometer = Gasometer::new(self.gas_limit, &self.config);

        // add the default base gas
        gasometer
            .record_cost(BASE_GAS_COUNT.as_u64())
            .map_err(Error::GasometerExit)?;

        // add the contract creation gas cost if action is create and input is not empty
        if matches!(transaction_action, ethereum::TransactionAction::Create)
            && transaction_input.trim() != ""
        {
            gasometer
                .record_cost(CONTRACT_CREATION_GAS.as_u64())
                .map_err(Error::GasometerExit)?;
        }

        let tx_input = hex::decode(transaction_input.as_str())?;

        // add transaction cost
        let tx_cost = match transaction_action {
            ethereum::TransactionAction::Call(_) => {
                evm_gasometer::call_transaction_cost(&tx_input, transaction_access_list.as_slice())
            }
            ethereum::TransactionAction::Create => evm_gasometer::create_transaction_cost(
                &tx_input,
                transaction_access_list.as_slice(),
            ),
        };
        gasometer
            .record_transaction(tx_cost)
            .map_err(Error::GasometerExit)?;

        // add opcode costs if applicable
        if !transaction_input.is_empty() {
            let operations = evm_disassembler::disassemble_bytes(tx_input.clone())?;
            for operation in operations {
                let op_code = self.get_evm_runtime_opcode(operation.opcode);
                if let Some(cost) = evm_gasometer::static_opcode_cost(op_code) {
                    gasometer.record_cost(cost).map_err(Error::GasometerExit)?;
                }
            }
        }

        info!("Gas used for transaction: {}", gasometer.total_used_gas());

        Ok(gasometer.total_used_gas())
    }

    fn get_evm_runtime_opcode(&self, op_code: evm_disassembler::Opcode) -> evm_runtime::Opcode {
        match op_code {
            Opcode::STOP => evm_runtime::Opcode::STOP,
            Opcode::ADD => evm_runtime::Opcode::ADD,
            Opcode::MUL => evm_runtime::Opcode::MUL,
            Opcode::SUB => evm_runtime::Opcode::SUB,
            Opcode::DIV => evm_runtime::Opcode::DIV,
            Opcode::SDIV => evm_runtime::Opcode::SDIV,
            Opcode::MOD => evm_runtime::Opcode::MOD,
            Opcode::SMOD => evm_runtime::Opcode::SMOD,
            Opcode::ADDMOD => evm_runtime::Opcode::ADDMOD,
            Opcode::MULMOD => evm_runtime::Opcode::MULMOD,
            Opcode::EXP => evm_runtime::Opcode::EXP,
            Opcode::SIGNEXTEND => evm_runtime::Opcode::SIGNEXTEND,
            Opcode::LT => evm_runtime::Opcode::LT,
            Opcode::GT => evm_runtime::Opcode::GT,
            Opcode::SLT => evm_runtime::Opcode::SLT,
            Opcode::SGT => evm_runtime::Opcode::SGT,
            Opcode::EQ => evm_runtime::Opcode::EQ,
            Opcode::ISZERO => evm_runtime::Opcode::ISZERO,
            Opcode::AND => evm_runtime::Opcode::AND,
            Opcode::OR => evm_runtime::Opcode::OR,
            Opcode::XOR => evm_runtime::Opcode::XOR,
            Opcode::NOT => evm_runtime::Opcode::NOT,
            Opcode::BYTE => evm_runtime::Opcode::BYTE,
            Opcode::SHL => evm_runtime::Opcode::SHL,
            Opcode::SHR => evm_runtime::Opcode::SHR,
            Opcode::SAR => evm_runtime::Opcode::SAR,
            Opcode::SHA3 => evm_runtime::Opcode::SHA3,
            Opcode::ADDRESS => evm_runtime::Opcode::ADDRESS,
            Opcode::BALANCE => evm_runtime::Opcode::BALANCE,
            Opcode::ORIGIN => evm_runtime::Opcode::ORIGIN,
            Opcode::CALLER => evm_runtime::Opcode::CALLER,
            Opcode::CALLVALUE => evm_runtime::Opcode::CALLVALUE,
            Opcode::CALLDATALOAD => evm_runtime::Opcode::CALLDATALOAD,
            Opcode::CALLDATASIZE => evm_runtime::Opcode::CALLDATASIZE,
            Opcode::CALLDATACOPY => evm_runtime::Opcode::CALLDATACOPY,
            Opcode::CODESIZE => evm_runtime::Opcode::CODESIZE,
            Opcode::CODECOPY => evm_runtime::Opcode::CODECOPY,
            Opcode::GASPRICE => evm_runtime::Opcode::GASPRICE,
            Opcode::EXTCODESIZE => evm_runtime::Opcode::EXTCODESIZE,
            Opcode::EXTCODECOPY => evm_runtime::Opcode::EXTCODECOPY,
            Opcode::RETURNDATASIZE => evm_runtime::Opcode::RETURNDATASIZE,
            Opcode::RETURNDATACOPY => evm_runtime::Opcode::RETURNDATACOPY,
            Opcode::EXTCODEHASH => evm_runtime::Opcode::EXTCODEHASH,
            Opcode::BLOCKHASH => evm_runtime::Opcode::BLOCKHASH,
            Opcode::COINBASE => evm_runtime::Opcode::COINBASE,
            Opcode::TIMESTAMP => evm_runtime::Opcode::TIMESTAMP,
            Opcode::NUMBER => evm_runtime::Opcode::NUMBER,
            Opcode::DIFFICULTY => evm_runtime::Opcode::DIFFICULTY,
            Opcode::GASLIMIT => evm_runtime::Opcode::GASLIMIT,
            Opcode::CHAINID => evm_runtime::Opcode::CHAINID,
            Opcode::SELFBALANCE => evm_runtime::Opcode::SELFBALANCE,
            Opcode::BASEFEE => evm_runtime::Opcode::BASEFEE,
            Opcode::POP => evm_runtime::Opcode::POP,
            Opcode::MLOAD => evm_runtime::Opcode::MLOAD,
            Opcode::MSTORE => evm_runtime::Opcode::MSTORE,
            Opcode::MSTORE8 => evm_runtime::Opcode::MSTORE8,
            Opcode::SLOAD => evm_runtime::Opcode::SLOAD,
            Opcode::SSTORE => evm_runtime::Opcode::SSTORE,
            Opcode::JUMP => evm_runtime::Opcode::JUMP,
            Opcode::JUMPI => evm_runtime::Opcode::JUMPI,
            Opcode::PC => evm_runtime::Opcode::PC,
            Opcode::MSIZE => evm_runtime::Opcode::MSIZE,
            Opcode::GAS => evm_runtime::Opcode::GAS,
            Opcode::JUMPDEST => evm_runtime::Opcode::JUMPDEST,
            Opcode::MCOPY => evm_runtime::Opcode::MCOPY,
            Opcode::TLOAD => evm_runtime::Opcode::TLOAD,
            Opcode::TSTORE => evm_runtime::Opcode::TSTORE,
            Opcode::PUSH0 => evm_runtime::Opcode::PUSH0,
            Opcode::PUSH1 => evm_runtime::Opcode::PUSH1,
            Opcode::PUSH2 => evm_runtime::Opcode::PUSH2,
            Opcode::PUSH3 => evm_runtime::Opcode::PUSH3,
            Opcode::PUSH4 => evm_runtime::Opcode::PUSH4,
            Opcode::PUSH5 => evm_runtime::Opcode::PUSH5,
            Opcode::PUSH6 => evm_runtime::Opcode::PUSH6,
            Opcode::PUSH7 => evm_runtime::Opcode::PUSH7,
            Opcode::PUSH8 => evm_runtime::Opcode::PUSH8,
            Opcode::PUSH9 => evm_runtime::Opcode::PUSH9,
            Opcode::PUSH10 => evm_runtime::Opcode::PUSH10,
            Opcode::PUSH11 => evm_runtime::Opcode::PUSH11,
            Opcode::PUSH12 => evm_runtime::Opcode::PUSH12,
            Opcode::PUSH13 => evm_runtime::Opcode::PUSH13,
            Opcode::PUSH14 => evm_runtime::Opcode::PUSH14,
            Opcode::PUSH15 => evm_runtime::Opcode::PUSH15,
            Opcode::PUSH16 => evm_runtime::Opcode::PUSH16,
            Opcode::PUSH17 => evm_runtime::Opcode::PUSH17,
            Opcode::PUSH18 => evm_runtime::Opcode::PUSH18,
            Opcode::PUSH19 => evm_runtime::Opcode::PUSH19,
            Opcode::PUSH20 => evm_runtime::Opcode::PUSH20,
            Opcode::PUSH21 => evm_runtime::Opcode::PUSH21,
            Opcode::PUSH22 => evm_runtime::Opcode::PUSH22,
            Opcode::PUSH23 => evm_runtime::Opcode::PUSH23,
            Opcode::PUSH24 => evm_runtime::Opcode::PUSH24,
            Opcode::PUSH25 => evm_runtime::Opcode::PUSH25,
            Opcode::PUSH26 => evm_runtime::Opcode::PUSH26,
            Opcode::PUSH27 => evm_runtime::Opcode::PUSH27,
            Opcode::PUSH28 => evm_runtime::Opcode::PUSH28,
            Opcode::PUSH29 => evm_runtime::Opcode::PUSH29,
            Opcode::PUSH30 => evm_runtime::Opcode::PUSH30,
            Opcode::PUSH31 => evm_runtime::Opcode::PUSH31,
            Opcode::PUSH32 => evm_runtime::Opcode::PUSH32,
            Opcode::DUP1 => evm_runtime::Opcode::DUP1,
            Opcode::DUP2 => evm_runtime::Opcode::DUP2,
            Opcode::DUP3 => evm_runtime::Opcode::DUP3,
            Opcode::DUP4 => evm_runtime::Opcode::DUP4,
            Opcode::DUP5 => evm_runtime::Opcode::DUP5,
            Opcode::DUP6 => evm_runtime::Opcode::DUP6,
            Opcode::DUP7 => evm_runtime::Opcode::DUP7,
            Opcode::DUP8 => evm_runtime::Opcode::DUP8,
            Opcode::DUP9 => evm_runtime::Opcode::DUP9,
            Opcode::DUP10 => evm_runtime::Opcode::DUP10,
            Opcode::DUP11 => evm_runtime::Opcode::DUP11,
            Opcode::DUP12 => evm_runtime::Opcode::DUP12,
            Opcode::DUP13 => evm_runtime::Opcode::DUP13,
            Opcode::DUP14 => evm_runtime::Opcode::DUP14,
            Opcode::DUP15 => evm_runtime::Opcode::DUP15,
            Opcode::DUP16 => evm_runtime::Opcode::DUP16,
            Opcode::SWAP1 => evm_runtime::Opcode::SWAP1,
            Opcode::SWAP2 => evm_runtime::Opcode::SWAP2,
            Opcode::SWAP3 => evm_runtime::Opcode::SWAP3,
            Opcode::SWAP4 => evm_runtime::Opcode::SWAP4,
            Opcode::SWAP5 => evm_runtime::Opcode::SWAP5,
            Opcode::SWAP6 => evm_runtime::Opcode::SWAP6,
            Opcode::SWAP7 => evm_runtime::Opcode::SWAP7,
            Opcode::SWAP8 => evm_runtime::Opcode::SWAP8,
            Opcode::SWAP9 => evm_runtime::Opcode::SWAP9,
            Opcode::SWAP10 => evm_runtime::Opcode::SWAP10,
            Opcode::SWAP11 => evm_runtime::Opcode::SWAP11,
            Opcode::SWAP12 => evm_runtime::Opcode::SWAP12,
            Opcode::SWAP13 => evm_runtime::Opcode::SWAP13,
            Opcode::SWAP14 => evm_runtime::Opcode::SWAP14,
            Opcode::SWAP15 => evm_runtime::Opcode::SWAP15,
            Opcode::SWAP16 => evm_runtime::Opcode::SWAP16,
            Opcode::LOG0 => evm_runtime::Opcode::LOG0,
            Opcode::LOG1 => evm_runtime::Opcode::LOG1,
            Opcode::LOG2 => evm_runtime::Opcode::LOG2,
            Opcode::LOG3 => evm_runtime::Opcode::LOG3,
            Opcode::LOG4 => evm_runtime::Opcode::LOG4,
            Opcode::CREATE => evm_runtime::Opcode::CREATE,
            Opcode::CALL => evm_runtime::Opcode::CALL,
            Opcode::CALLCODE => evm_runtime::Opcode::CALLCODE,
            Opcode::RETURN => evm_runtime::Opcode::RETURN,
            Opcode::DELEGATECALL => evm_runtime::Opcode::DELEGATECALL,
            Opcode::CREATE2 => evm_runtime::Opcode::CREATE2,
            Opcode::STATICCALL => evm_runtime::Opcode::STATICCALL,
            Opcode::REVERT => evm_runtime::Opcode::REVERT,
            Opcode::INVALID => evm_runtime::Opcode::INVALID,
            Opcode::SELFDESTRUCT => evm_runtime::Opcode::SUICIDE,
            Opcode::BLOBBASEFEE => evm_runtime::Opcode::BASEFEE,
            Opcode::BLOBHASH => evm_runtime::Opcode::INVALID,
        }
    }
}
