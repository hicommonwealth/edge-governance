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
// along with Edgeware.  If not, see <http://www.gnu.org/licenses/>

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate serde;

// Needed for deriving `Serialize` and `Deserialize` for various types.
// We only implement the serde traits for std builds - they're unneeded
// in the wasm runtime.
#[cfg(feature = "std")]
#[macro_use]
extern crate serde_derive;
#[cfg(test)]
#[macro_use]
extern crate hex_literal;
#[macro_use] extern crate parity_codec_derive;
#[macro_use] extern crate srml_support;


extern crate parity_codec as codec;
extern crate substrate_primitives as primitives;
#[cfg_attr(not(feature = "std"), macro_use)]
extern crate sr_std as rstd;
extern crate srml_support as runtime_support;
extern crate sr_primitives as runtime_primitives;
extern crate sr_io as runtime_io;
extern crate srml_system as system;

use codec::Encode;
use rstd::prelude::*;
use runtime_support::dispatch::Result;

pub mod governance;
pub use governance::{Module, Trait, RawEvent, Event};

#[cfg(test)]
mod tests {
    use super::*;
    use system::{EventRecord, Phase};
    use runtime_io::with_externalities;
    use runtime_io::ed25519::Pair;
    use primitives::{H256, Blake2Hasher, Hasher};
    // The testing primitives are very useful for avoiding having to work with signatures
    // or public keys. `u64` is used as the `AccountId` and no `Signature`s are requried.
    use runtime_primitives::{
        BuildStorage, traits::{BlakeTwo256}, testing::{Digest, DigestItem, Header}
    };

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    impl_outer_event! {
        pub enum Event for Test {
            governance<T>,
        }
    }

    impl_outer_dispatch! {
        pub enum Call for Test where origin: Origin {}
    }

    // For testing the module, we construct most of a mock runtime. This means
    // first constructing a configuration type (`Test`) which `impl`s each of the
    // configuration traits of modules we want to use.
    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;
    impl system::Trait for Test {
        type Origin = Origin;
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type Digest = Digest;
        type AccountId = H256;
        type Header = Header;
        type Event = Event;
        type Log = DigestItem;
    }

    impl Trait for Test {
        type Event = Event;
    }

    pub type System = system::Module<Test>;
    pub type Governance = Module<Test>;

    fn new_test_ext() -> sr_io::TestExternalities<Blake2Hasher> {
        let t = system::GenesisConfig::<Test>::default().build_storage().unwrap().0;
        // We use default for brevity, but you can configure as desired if needed.
        t.into()
    }

    fn propose(who: H256, title: &[u8], proposal: &[u8], category: governance::ProposalCategory) -> super::Result {
        Governance::create_proposal(Origin::signed(who), title.to_vec(), proposal.to_vec(), category)
    }

    fn add_comment(who: H256, proposal_hash: H256, comment: &[u8]) -> super::Result {
        Governance::add_comment(Origin::signed(who), proposal_hash, comment.to_vec())
    }

    fn advance_proposal(who: H256, proposal_hash: H256) -> super::Result {
        Governance::advance_proposal(Origin::signed(who), proposal_hash)
    }

    fn submit_vote(who: H256, proposal_hash: H256, vote: bool) -> super::Result {
        Governance::submit_vote(Origin::signed(who), proposal_hash, vote)
    }

    fn build_proposal_hash(who: H256, proposal: &[u8]) -> H256 {
            let mut buf = Vec::new();
            buf.extend_from_slice(&who.encode());
            buf.extend_from_slice(proposal.as_ref());
            return Blake2Hasher::hash(&buf[..]);
    }

    fn get_test_key() -> H256 {
        let pair: Pair = Pair::from_seed(&hex!("9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60"));
        let public: H256 = pair.public().0.into();
        return public;
    }

    fn generate_proposal() -> (&'static[u8], &'static[u8]) {
        let title: &[u8] = b"Make Edgeware Free";
        let proposal: &[u8] = b"Simple: make Edgeware free for everyone";
        return (title, proposal);
    }

    #[test]
    fn propose_should_work() {
        with_externalities(&mut new_test_ext(), || {
            System::set_block_number(1);
            let public = get_test_key();
            let category = governance::ProposalCategory::Funding(12);
            let (title, proposal) = generate_proposal();
            let hash = build_proposal_hash(public, &proposal);
            assert_ok!(propose(public, title, proposal, category));
            assert_eq!(System::events(), vec![
                EventRecord {
                    phase: Phase::ApplyExtrinsic(0),
                    event: Event::governance(RawEvent::NewProposal(public, hash))
                }]
            );

            let title2: &[u8] = b"Proposal 2";
            let proposal2: &[u8] = b"Proposal 2";
            let hash2 = build_proposal_hash(public, &proposal2);
            assert_ok!(propose(public, title2, proposal2, category));
            assert_eq!(System::events(), vec![
                EventRecord {
                    phase: Phase::ApplyExtrinsic(0),
                    event: Event::governance(RawEvent::NewProposal(public, hash))
                },
                EventRecord {
                    phase: Phase::ApplyExtrinsic(0),
                    event: Event::governance(RawEvent::NewProposal(public, hash2))
                },]
            );
        });
    }

    #[test]
    fn propose_duplicate_should_fail() {
        with_externalities(&mut new_test_ext(), || {
            System::set_block_number(1);
            let public = get_test_key();
            let (title, proposal) = generate_proposal();
            let category = governance::ProposalCategory::Signaling;
            let hash = build_proposal_hash(public, &proposal);
            assert_ok!(propose(public, title, proposal, category));
            assert_eq!(propose(public, title, proposal, category), Err("Proposal already exists"));
        });
    }

    #[test]
    fn propose_empty_should_fail() {
        with_externalities(&mut new_test_ext(), || {
            System::set_block_number(1);
            let public = get_test_key();
            let (title, _) = generate_proposal();
            let proposal = vec![];
            let category = governance::ProposalCategory::Upgrade;
            let hash = build_proposal_hash(public, &proposal);
            assert_eq!(propose(public, title, &proposal, category), Err("Proposal must not be empty"));
        });
    }

    #[test]
    fn propose_empty_title_should_fail() {
        with_externalities(&mut new_test_ext(), || {
            System::set_block_number(1);
            let public = get_test_key();
            let (_, proposal) = generate_proposal();
            let title = vec![];
            let category = governance::ProposalCategory::Upgrade;
            let hash = build_proposal_hash(public, &proposal);
            assert_eq!(propose(public, &title, proposal, category), Err("Proposal must have title"));
        });
    }

    #[test]
    fn comment_should_work() {
        with_externalities(&mut new_test_ext(), || {
            System::set_block_number(1);
            let public = get_test_key();
            let (title, proposal) = generate_proposal();
            let category = governance::ProposalCategory::Upgrade;
            assert_ok!(propose(public, title, proposal, category));
            let hash = build_proposal_hash(public, &proposal);

            // create a comment
            let comment: &[u8] = b"pls do not do this";
            assert_ok!(add_comment(public, hash, comment));
            assert_eq!(System::events()[1], EventRecord {
                phase: Phase::ApplyExtrinsic(0),
                event: Event::governance(RawEvent::NewComment(public, hash))
            });
        });
    }

    #[test]
    fn comment_on_invalid_proposal_should_fail() {
        with_externalities(&mut new_test_ext(), || {
            System::set_block_number(1);
            let public = get_test_key();

            // create a comment and an invalid hash
            let comment: &[u8] = b"pls do not do this";
            let hash: H256 = public.clone();
            assert_eq!(add_comment(public, hash, comment), Err("Proposal does not exist"));
        });
    }

    #[test]
    fn advance_proposal_should_work_until_completed() {
        with_externalities(&mut new_test_ext(), || {
            System::set_block_number(1);
            let public = get_test_key();
            let category = governance::ProposalCategory::Funding(12);
            let (title, proposal) = generate_proposal();
            let hash = build_proposal_hash(public, &proposal);
            assert_ok!(propose(public, title, proposal, category));
            assert_ok!(advance_proposal(public, hash));
            assert_ok!(advance_proposal(public, hash));
            assert_eq!(advance_proposal(public, hash), Err("Proposal already completed"));
            assert_eq!(System::events(), vec![
                EventRecord {
                    phase: Phase::ApplyExtrinsic(0),
                    event: Event::governance(RawEvent::NewProposal(public, hash))
                },
                EventRecord {
                    phase: Phase::ApplyExtrinsic(0),
                    event: Event::governance(RawEvent::VotingStarted(hash))
                },
                EventRecord {
                    phase: Phase::ApplyExtrinsic(0),
                    event: Event::governance(RawEvent::VotingCompleted(hash))
                },]
            );
        });
    }

    #[test]
    fn non_author_advance_should_fail() {
        with_externalities(&mut new_test_ext(), || {
            System::set_block_number(1);
            let public = get_test_key();
            let category = governance::ProposalCategory::Funding(12);
            let (title, proposal) = generate_proposal();
            let hash = build_proposal_hash(public, &proposal);

            let other_pair: Pair = Pair::from_seed(&hex!("9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f61"));
            let other_public: H256 = other_pair.public().0.into();
            assert_ok!(propose(public, title, proposal, category));
            assert_eq!(advance_proposal(other_public, hash), Err("Proposal must be advanced by author"));
        });
    }

    #[test]
    fn submit_vote_should_work() {
        with_externalities(&mut new_test_ext(), || {
            System::set_block_number(1);
            let public = get_test_key();
            let category = governance::ProposalCategory::Funding(12);
            let (title, proposal) = generate_proposal();
            let hash = build_proposal_hash(public, &proposal);
            assert_ok!(propose(public, title, proposal, category));
            assert_ok!(advance_proposal(public, hash));
            assert_ok!(submit_vote(public, hash, true));
        });
    }

    #[test]
    fn submit_vote_at_wrong_stage_should_fail() {
        with_externalities(&mut new_test_ext(), || {
            System::set_block_number(1);
            let public = get_test_key();
            let category = governance::ProposalCategory::Funding(12);
            let (title, proposal) = generate_proposal();
            let hash = build_proposal_hash(public, &proposal);
            assert_ok!(propose(public, title, proposal, category));
            assert_eq!(submit_vote(public, hash, true), Err("Proposal not in voting stage"));
            assert_ok!(advance_proposal(public, hash));
            assert_ok!(advance_proposal(public, hash));
            assert_eq!(submit_vote(public, hash, true), Err("Proposal not in voting stage"));
        });
    }
} 
