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

// TODO: stage transition functions
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, PartialEq, Clone, Copy)]
pub enum ProposalStage {
    PreVoting,
    Voting,
    Completed,
}

// TODO: less static categories + way of interfacing w frontend
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, PartialEq, Clone, Copy)]
pub enum ProposalCategory {
    Referendum,
    Funding,
    NetworkChange,
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, PartialEq, Clone, Copy)]
pub struct ProposalRecord<AccountId> {
    pub index: u32,
    pub author: AccountId,
    pub stage: ProposalStage,
    pub category: Proposal,
    pub contents: Vec<u8>,
    pub comments: Vec<(Vec<u8>, AccountId)>,
}

pub trait Trait: balances::Trait {
    /// The overarching event type
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        pub fn create_proposal(origin, proposal: Vec<u8>, category: ProposalCategory) -> Result {
            let _sender = ensure_signed(origin)?;
            let index = <ProposalCount<T>>::mutate(|i| *i += 1);
            let hash = T::Hashing::Hash(&proposal);
            let record = ProposalRecord { index: index,
                                          author: _sender.clone(),
                                          stage: ProposalStage::PreVoting,
                                          category: category,
                                          contents: proposal,
                                          comments: vec![] };
            <ProposalOf<T>>::insert(&hash, record);
            <Proposals<T>>::put(&hash);
            Self::deposit_event(RawEvent::NewProposal(_sender, hash));
            Ok(())
        }

        // TODO: give comments unique numbers/ids?
        pub fn add_comment(origin, proposal_hash: T::Hash, comment: Vec<u8>) -> Result {
            let _sender = ensure_signed(origin)?;
            // TODO: can we mut borrow somehow?
            let record = <ProposalOf<T>>::get(&proposal_hash).ok_or("Proposal does not exist")?;
            let mut new_record = record.clone();
            new_record.comments.push((comment, _sender.clone()));
            <ProposalOf<T>>::insert(&proposal_hash, new_record);
            Self::deposit_event(RawEvent::NewComment(_sender, hash));
            Ok(())
            }
        }

        pub fn vote(origin, proposal_hash: T::Hash, vote: bool) -> Result {
            // TODO: how do we know when ready for voting?
            let _sender = ensure_signed(origin)?;
            let record = <ProposalOf<T>>::get(&proposal_hash).ok_or("Proposal does not exist")?;
            if record.stage != ProposalStage::Voting {
                return Err("Proposal is not in voting stage");
            }
            // TODO: how do we handle....actually voting? With the Democracy module?
            unimplemented!();
        }
    }
}

decl_event!(
    pub enum Event<T> where <T as system::Trait>::Hash,
                            <T as system::Trait>::AccountId {
        NewProposal(AccountId, Hash),
        NewComment(AccountId, Hash)
    }
);

decl_storage! {
    trait Store for Module<T: trait> as GovernanceStorage {
        pub ProposalCount get(proposal_count) : u32;
        pub Proposals get(proposals): Vec<T::Hash>;
        pub ProposalOf get(proposal_of): map T::Hash => Option<ProposalRecord<T::AccountId>>;
    }
}
