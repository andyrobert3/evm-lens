// re implementation of trace items in eip3155 bcs it doesn't export them ffs

use revm::{
    bytecode::OpCode,
    primitives::{HashMap, U256},
};
use serde::Deserialize;

use crate::sort::SortMarker;

#[derive(Debug, Clone, Deserialize)]
pub struct Summary {
    // Required fields:
    /// Root of the state trie after executing the transaction
    state_root: String,
    /// Return values of the function
    output: String,
    /// All gas used by the transaction
    gas_used: u64,
    /// Bool whether transaction was executed successfully
    pass: bool,

    // Optional fields:
    /// Time in nanoseconds needed to execute the transaction
    time: Option<u128>,
    /// Name of the fork rules used for execution
    fork: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
pub struct Output {
    // Required fields:
    /// Program counter
    pc: u64,
    /// EOF code section
    section: Option<u64>,
    /// OpCode
    op: u8,
    /// Gas left before executing this operation
    gas: u64,
    /// Gas cost of this operation
    gas_cost: u64,
    /// Array of all values on the stack
    stack: Vec<U256>,
    /// Depth of the call stack
    depth: u64,
    /// Depth of the EOF function call stack
    function_depth: Option<u64>,
    /// Data returned by the function call
    return_data: String,
    /// Amount of **global** gas refunded
    refund: u64,
    /// Size of memory array
    mem_size: u64,

    // Optional fields:
    /// Name of the operation
    op_name: Option<String>,
    /// Description of an error (should contain revert reason if supported)
    error: Option<String>,
    /// Array of all allocated values
    memory: Option<String>,
    /// Array of all stored values
    storage: Option<HashMap<String, String>>,
    /// Array of values, Stack of the called function
    return_stack: Option<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrderedTrace(Vec<Output>);

impl OrderedTrace {
    pub fn sort(raw: Vec<Output>) -> Vec<Self> {
        let mut out_buff = vec![];
        let mut tmp = vec![];

        for raw_trace in raw {
            let op = OpCode::new(raw_trace.op).expect("unknown opcode encountered");

            if op == OpCode::CALL || op == OpCode::DELEGATECALL {
                if !tmp.is_empty() {
                    out_buff.push(OrderedTrace(tmp));
                }

                tmp = vec![];

                tmp.push(raw_trace);
            } else {
                tmp.push(raw_trace);
            }
        }

        // push last tmp if any
        if !tmp.is_empty() {
            out_buff.push(OrderedTrace(tmp));
        }

        out_buff
    }
}

#[derive(Debug, Clone)]
pub enum TraceKind {
    Output(Output),
    Summary(Summary),
    Ordered(OrderedTrace),
}

impl SortMarker for TraceKind {
    fn sort(&mut self) {
        todo!()
    }
}

impl TryFrom<&[u8]> for TraceKind {
    type Error = serde_json::error::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let b = value.to_owned();
        let a = serde_json::from_slice::<Output>(&b);

        if let Ok(output) = serde_json::from_slice::<Output>(value) {
            Ok(Self::Output(output))
        } else {
            Ok(Self::Summary(serde_json::from_slice::<Summary>(value)?))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn dummy_trace(op: OpCode) -> Output {
        let mut out: Output = Default::default();

        out.op = op.as_usize() as u8;

        out
    }

    #[test]
    fn test_sort() {
        let unordered_shit: Vec<Output> = vec![
            dummy_trace(OpCode::CALL),
            dummy_trace(OpCode::DATASIZE),
            dummy_trace(OpCode::SSTORE),
            dummy_trace(OpCode::DELEGATECALL),
            dummy_trace(OpCode::SLOAD),
            dummy_trace(OpCode::RETURN),
        ];

        let ordered = OrderedTrace::sort(unordered_shit);

        println!("{:#?}", ordered);

        assert_eq!(ordered.len(), 2);
        assert_eq!(
            ordered[0],
            OrderedTrace(vec![
                dummy_trace(OpCode::CALL),
                dummy_trace(OpCode::DATASIZE),
                dummy_trace(OpCode::SSTORE),
            ])
        );
        assert_eq!(
            ordered[1],
            OrderedTrace(vec![
                dummy_trace(OpCode::DELEGATECALL),
                dummy_trace(OpCode::SLOAD),
                dummy_trace(OpCode::RETURN),
            ])
        );
    }
}
