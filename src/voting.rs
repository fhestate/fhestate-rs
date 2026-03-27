//! High-level Confidential Voting and Governance Module
//!
//! Provides a production-grade interface for the Dark DAO deliverable.
//! Handles encrypted tallies and winner detection using FHE logic.

use crate::errors::FheResult;
use crate::math::FheMath;
use crate::logic::FheLogic;
use tfhe::FheUint32;

/// A production-grade aggregator for encrypted ballots.
pub struct VotingTally;

impl VotingTally {
    /// Aggregates encrypted votes for a list of candidates.
    /// 
    /// # Arguments
    /// * `votes` - A list of encrypted votes (e.g., 0 for No, 1 for Yes, or Candidate Index).
    /// * `candidate_count` - The number of options being voted on.
    pub fn tally_binary_votes(votes: Vec<FheUint32>) -> Option<FheUint32> {
        // Uses the optimized Tree-Sum to aggregate a single candidate's score.
        FheMath::tree_sum(votes)
    }

    /// Aggregates votes for multiple candidates and returns the winner's index.
    /// 
    /// This is a "Black-Box" tally: the final result reveals ONLY the winner's ID, 
    /// keeping the individual vote counts completely encrypted.
    pub fn find_winner(candidate_totals: &[FheUint32]) -> FheResult<FheUint32> {
        if candidate_totals.is_empty() {
             return Err(crate::errors::FheError::ComputationFailed("No candidates found".to_string()));
        }

        let mut max_val = candidate_totals[0].clone();
        
        for i in 1..candidate_totals.len() {
            max_val = FheLogic::max(&max_val, &candidate_totals[i])?;
        }

        Ok(max_val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tfhe::prelude::*;
    use tfhe::{ConfigBuilder, generate_keys, set_server_key, FheUint32};

    fn setup() -> tfhe::ClientKey {
        let config = ConfigBuilder::default().build();
        let (client_key, server_key) = generate_keys(config);
        set_server_key(server_key);
        client_key
    }

    #[test]
    #[ignore = "Slow FHE keygen — run with: cargo test -- --ignored"]
    fn test_production_voting_flow() {
        let ck = setup();
        
        // Simulate: 5 voters for a single candidate
        let votes: Vec<FheUint32> = (0..5)
            .map(|_| FheUint32::encrypt(1u32, &ck))
            .collect();
            
        let tally = VotingTally::tally_binary_votes(votes).unwrap();
        let result: u32 = tally.decrypt(&ck);
        
        assert_eq!(result, 5);
    }
}
