// RGB node providing smart contracts functionality for Bitcoin & Lightning.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use std::collections::BTreeSet;
use std::fmt::{self, Display, Formatter};

use amplify::Wrapper;
use bitcoin::{OutPoint, Txid};
use internet2::presentation;
use lnpbp::chain::Chain;
use microservices::rpc;
use rgb::schema::TransitionType;
use rgb::{
    validation, ConsignmentType, Contract, ContractConsignment, ContractId, ContractState,
    InmemConsignment, StateTransfer, TransferConsignment,
};

use crate::FailureCode;

/// We need this wrapper type to be compatible with RGB Node having multiple message buses
#[derive(Clone, Debug, Display, From, Api)]
#[api(encoding = "strict")]
#[non_exhaustive]
pub(crate) enum BusMsg {
    #[api(type = 4)]
    #[display(inner)]
    #[from]
    Rpc(RpcMsg),
}

impl rpc::Request for BusMsg {}

#[derive(Clone, Debug, Display, From)]
#[derive(NetworkEncode, NetworkDecode)]
#[display(inner)]
pub enum RpcMsg {
    #[from]
    Hello(HelloReq),

    // Contract operations
    // -------------------
    #[display(inner)]
    AcceptContract(AcceptReq<ContractConsignment>),

    #[display("list_contracts")]
    ListContracts,

    #[display("get_contract({0})")]
    GetContract(ContractReq),

    #[display("get_contract_state({0})")]
    GetContractState(ContractId),

    // Stash operations
    // ----------------
    BlindUtxo,

    ConsignTransfer,

    #[display("accept_transfer(...)")]
    AcceptTransfer(AcceptReq<TransferConsignment>),

    // Responses to CLI
    // ----------------
    #[display("contract_ids(...)")]
    ContractIds(BTreeSet<ContractId>),

    #[display("contract(...)")]
    Contract(Contract),

    #[display("contract_state(...)")]
    ContractState(ContractState),

    #[display("state_transfer(...)")]
    StateTransfer(StateTransfer),

    #[display("progress(\"{0}\")")]
    #[from]
    Progress(String),

    #[display("success{0}")]
    Success(OptionDetails),

    #[display("failure({0:#})")]
    #[from]
    Failure(rpc::Failure<FailureCode>),

    #[display("unresolved_txids(...)")]
    UnresolvedTxids(Vec<Txid>),

    #[display("invalid(...)")]
    Invalid(validation::Status),
}

impl From<presentation::Error> for RpcMsg {
    fn from(err: presentation::Error) -> Self {
        RpcMsg::Failure(rpc::Failure {
            code: rpc::FailureCode::Presentation,
            info: format!("{}", err),
        })
    }
}

impl RpcMsg {
    pub fn success() -> Self { RpcMsg::Success(None.into()) }
    pub fn failure(code: FailureCode, message: impl ToString) -> Self {
        RpcMsg::Failure(rpc::Failure {
            code: rpc::FailureCode::Other(code),
            info: message.to_string(),
        })
    }
}

#[derive(Clone, Debug)]
#[derive(StrictEncode, StrictDecode)]
pub enum ContractValidity {
    Valid,
    Invalid(validation::Status),
    UnknownTxids(Vec<Txid>),
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[derive(StrictEncode, StrictDecode)]
pub enum OutpointSelection {
    All,
    Spending(BTreeSet<OutPoint>),
}

impl OutpointSelection {
    pub fn includes(&self, outpoint: OutPoint) -> bool {
        match self {
            OutpointSelection::All => true,
            OutpointSelection::Spending(set) => set.contains(&outpoint),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Display)]
#[derive(NetworkEncode, NetworkDecode)]
#[display("accept(force: {force}, ...)")]
pub struct AcceptReq<T: ConsignmentType> {
    pub consignment: InmemConsignment<T>,
    pub force: bool,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
#[derive(NetworkEncode, NetworkDecode)]
#[display("hello({network}, {user_agent})")]
pub struct HelloReq {
    pub user_agent: String,
    pub network: Chain,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
#[derive(NetworkEncode, NetworkDecode)]
#[display("get_contract({contract_id}, ...)")]
pub struct ContractReq {
    pub contract_id: ContractId,
    pub include: BTreeSet<TransitionType>,
    pub outpoints: OutpointSelection,
}

#[derive(Wrapper, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, From, Default)]
#[derive(NetworkEncode, NetworkDecode)]
pub struct OptionDetails(pub Option<String>);

impl Display for OptionDetails {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.as_inner() {
            None => Ok(()),
            Some(msg) => write!(f, "; \"{}\"", msg),
        }
    }
}

impl OptionDetails {
    pub fn with(s: impl ToString) -> Self { Self(Some(s.to_string())) }

    pub fn new() -> Self { Self(None) }
}

impl From<String> for OptionDetails {
    fn from(s: String) -> Self { OptionDetails(Some(s)) }
}

impl From<&str> for OptionDetails {
    fn from(s: &str) -> Self { OptionDetails(Some(s.to_string())) }
}

impl From<&str> for RpcMsg {
    fn from(s: &str) -> Self { RpcMsg::Progress(s.to_owned()) }
}
