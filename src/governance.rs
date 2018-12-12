// Copyright 2018 Commonwealth Labs, Inc.
// This file is part of Edgeware.

// Edgeware is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Edgeware is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Edgeware.  If not, see <http://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(feature = "std")]
extern crate serde;

// Needed for deriving `Serialize` and `Deserialize` for various types.
// We only implement the serde traits for std builds - they're unneeded
// in the wasm runtime.
#[cfg(feature = "std")]

extern crate parity_codec as codec;
extern crate substrate_primitives as primitives;
extern crate sr_std as rstd;
extern crate srml_support as runtime_support;
extern crate sr_primitives as runtime_primitives;
extern crate sr_io as runtime_io;

extern crate srml_balances as balances;
extern crate srml_system as system;

use rstd::prelude::*;
use system::ensure_signed;
use runtime_support::{StorageValue, StorageMap, Parameter};
use runtime_support::dispatch::Result;

pub type ProposalIndex = u32;

#[derive(Encode, Decode)]
#[cfg_attr(feature = "std", derive(Serialize, PartialEq, Eq, Clone, Debug))]
pub enum ProposalStage {
    PreVoting,
    Voting,
    Completed,
}
// TODO: stage transition functions

pub enum ProposalCategory {
    Referendum,
    Funding,
    NetworkChange,
}
// TODO: less static categories + way of interfacing w frontend

pub struct ProposalRecord<AccountId> {
    pub stage: ProposalStage,
    pub category: Proposal,
    pub contents: Vec<u8>,
    pub comments: Vec<(Vec<u8>, AccountId)>
}

pub trait Trait: balances::Trait {
    /// The overarching event type
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        pub fn create_proposal(origin, proposal: Vec<u8>) -> Result {
            let _sender = ensure_signed(origin)?;
            let record = ProposalRecord { stage: ProposalStage::PreVoting,
                                          category: ProposalCategory::Referendum, // TODO
                                          contents: proposal,
                                          comments: vec![] };
            <Proposals<T>>::insert(<ProposalCount<T>>::get(), record);
            <ProposalCount<T>>::mutate(|i| *i += 1);
            Ok(())
        }

        pub fn add_comment(origin, proposal_index: ProposalIndex, comment: Vec<u8>) -> Result {
            let _sender = ensure_signed(origin)?;
            match <Proposals<T>>::get(proposal_index) {
                None => Err("Proposal not found"),
                Some(record) => {
                    let mut new_record = record.clone();
                    new_record.comments.push(comment);
                    <Proposals<T>>::insert(proposal_index, new_record);
                    Ok(())
                }
            }
        }

        pub fn vote(origin, proposal_index: ProposalIndex, vote: bool) -> Result {
            unimplemented!();
        }
    }
}

decl_storage! {
    trait Store for Module<T: trait> as GovernanceStorage {
        // TODO: change up these types for more extensibility and performance.
        // TODO: add mappings from accounts to proposals.
        pub Proposals get(proposals): map ProposalIndex => Vec<ProposalRecord>;
        pub ProposalCount get(proposal_count): ProposalIndex;
    }
}
