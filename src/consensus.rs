use serde::de;

use crate::types::{Event, PlRequest, Message, NodeId, Height, Round, Step, Value, Proposal, Vote, VoteType};
use std::collections::HashMap;

pub struct ConsensusState {
    id: NodeId, 
    height: Height,
    round: Round,
    step: Step,
    locked_value: Option<Value>,
    locked_round: Option<Round>,
    valid_value: Option<Value>,
    valid_round: Option<Round>,
    proposal: Option<Proposal>,
    votes: HashMap<Round, RoundVote>,
    total_nodes: u32,
}

pub struct RoundVote {
    pub prevotes: HashMap<NodeId, Vote>,
    pub precommits: HashMap<NodeId, Vote>,
}

impl ConsensusState {
    pub fn new(id: NodeId, total_nodes: u32) -> Self {
        ConsensusState {
            id,
            height: 1,
            round: 0,
            step: Step::Propose,
            locked_value: None,
            locked_round: None,
            valid_value: None,
            valid_round: None,
            proposal: None,
            votes: HashMap::new(),
            total_nodes,
        }
    }

    pub fn start_round(&mut self, round: Round) -> Option<PlRequest> {
        self.round = round;
        self.step = Step::Propose;
        self.proposal = None;
        self.votes.insert(self.round, RoundVote {
            prevotes: HashMap::new(),
            precommits: HashMap::new(),
        });

        if self.get_proposer() == self.id {
            let value_to_propose = self.valid_value.clone().unwrap_or(Value { data: "New Block".to_string() });
        
            let proposal = Proposal {
                height: self.height,
                round: self.round,
                value: value_to_propose,
                valid_round: self.valid_round,
                sender: self.id,
            };

            return Some(PlRequest::Broadcast { msg: Message::Proposal(proposal) });
        }

        None
    }

    pub fn get_proposer(&self) -> NodeId {
        ((self.height + self.round as u64) % self.total_nodes as u64) as NodeId
    }

    pub fn handle_proposal(&mut self, proposal: Proposal) -> Option<PlRequest> {
        if self.step != Step::Propose || proposal.round != self.round || proposal.height != self.height {
            return None;
        }

        if proposal.sender != self.get_proposer() {
            return None;
        }

        let mut vote_value = Some(proposal.value.clone());
        
        if let Some(locked) = &self.locked_value {
            if locked != &proposal.value {
                vote_value = None;
            }
        }

        self.proposal = Some(proposal);
        self.step = Step::Prevote;

        let vote = Vote {
            vote_type: VoteType::Prevote,
            height: self.height,
            round: self.round,
            value: vote_value,
            sender: self.id,
        };

        Some(PlRequest::Broadcast { msg: Message::Vote(vote) })
    }

    pub fn handle_vote(&mut self, vote: Vote) -> Option<PlRequest> {
        if vote.height != self.height || vote.round != self.round {
            return None;
        }

        let quorum_size: usize = ((self.total_nodes * 2) / 3 + 1)  as usize;

        let round_votes = self.votes.entry(vote.round).or_insert(RoundVote {
            prevotes: HashMap::new(),
            precommits: HashMap::new(),
        });

        match vote.vote_type {
            VoteType::Prevote => {
                round_votes.prevotes.insert(vote.sender, vote.clone());
                
                let count = round_votes.prevotes.values()
                    .filter(|v| v.value == vote.value)
                    .count();

                if count >= quorum_size && self.step == Step::Prevote {
                    // TODO logs
                    // TODO update locks, -> Precommit, and return precommit braodcast
                    if vote.value.is_some() {
                        self.locked_value = vote.value.clone();
                        self.locked_round = Some(vote.round);
                        self.valid_value = vote.value.clone();
                        self.valid_round = Some(vote.round);
                    }
                    
                    self.step = Step::Precommit;
                    let precommit_vote = Vote {
                        vote_type: VoteType::Precommit,
                        height: self.height,
                        round: self.round,
                        value: vote.value.clone(),
                        sender: self.id,
                    };

                    return Some(PlRequest::Broadcast { msg: Message::Vote(precommit_vote) });
                }
            }
            VoteType::Precommit => {
                round_votes.precommits.insert(vote.sender, vote.clone());

                let count = round_votes.precommits.values()
                    .filter(|v| v.value == vote.value)
                    .count();

                if count >= quorum_size && self.step == Step::Precommit {
                    // TODO logs
                    if let Some(decided_value) = &vote.value {
                        self.height += 1;
                        // fresh height
                        self.locked_round = None;
                        self.locked_value = None;
                        self.valid_round = None;
                        self.valid_value = None;

                        return self.start_round(0);
                    } else {
                        let next_round = self.round + 1;
                        return  self.start_round(next_round);
                    }
               }
            } 
        }
        
        None 
    }
}
