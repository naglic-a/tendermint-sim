use serde::{Deserialize, Serialize};

pub type NodeId = u32;
pub type Height = u64;
pub type Round = u32;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Value {
    pub data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub height: Height,
    pub round: Round,
    pub value: Value,
    pub valid_round: Option<Round>,
    pub sender: NodeId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub vote_type: VoteType,
    pub height: Height,
    pub round: Round,
    pub value: Option<Value>, // None represents a 'nil' vote
    pub sender: NodeId,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VoteType {
    Prevote,
    Precommit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    Proposal(Proposal),
    Vote(Vote),
}
