use std::{
    io::{stdout, Write},
    sync::{atomic::AtomicBool, Arc, Mutex},
};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use ore::{self, state::Bus, BUS_ADDRESSES, BUS_COUNT, EPOCH_DURATION};
use rand::Rng;
use solana_program::{keccak::HASH_BYTES, program_memory::sol_memcmp, pubkey::Pubkey};
use solana_program::keccak::Hash;
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction,
    keccak::{hashv, Hash as KeccakHash},
    signature::Signer,
};
use tokio::io::AsyncWriteExt;

use crate::{cu_limits::{CU_LIMIT_MINE, CU_LIMIT_RESET}, utils::{get_clock_account, get_proof, get_treasury}, Miner, utils};

// Odds of being selected to submit a reset tx
const RESET_ODDS: u64 = 20;

impl Miner {
    pub async fn mine(&self, threads: u64) {
        // Register, if needed.
        let signer = self.signer();
        self.register().await;
        let mut stdout = stdout();
        let mut rng = rand::thread_rng();

        // Start mining loop
        loop {
            // Fetch account state
            let balance = self.get_ore_display_balance().await;
            let treasury = get_treasury(&self.rpc_client).await;
            let proof = get_proof(&self.rpc_client, signer.pubkey()).await;
            let rewards =
                (proof.claimable_rewards as f64) / (10f64.powf(ore::TOKEN_DECIMALS as f64));
            let reward_rate =
                (treasury.reward_rate as f64) / (10f64.powf(ore::TOKEN_DECIMALS as f64));
            stdout.write_all(b"\x1b[2J\x1b[3J\x1b[H").ok();
            println!("Balance: {} ORZ", balance);
            println!("Claimable: {} ORZ", rewards);
            println!("Reward rate: {} ORZ", reward_rate);

            // Escape sequence that clears the screen and the scrollback buffer
            println!("\nMining for a valid hash...");
            // let (next_hash, nonce) =
            //     self.find_next_hash_par(proof.hash.into(), treasury.difficulty.into(), threads);

            let all_pubkey = vec![signer.pubkey()];
            let proofs = vec![proof];
            let hash_and_pubkey = all_pubkey
                .iter()
                .zip(proofs.iter())
                .map(|(signer, proof)| (solana_sdk::keccak::Hash::new_from_array(proof.hash.0), *signer))
                .collect::<Vec<_>>();
            let (mining_duration, mining_results) = self
                .mine_hashes_gpu(&treasury.difficulty.into(), &hash_and_pubkey)
                .await;
            let (next_hash, nonce) = mining_results[0];

            // Submit mine tx.
            // Use busses randomly so on each epoch, transactions don't pile on the same busses
            println!("\n\nSubmitting hash for validation...");
            'submit: loop {
                // Double check we're submitting for the right challenge
                let proof_ = get_proof(&self.rpc_client, signer.pubkey()).await;
                if !self.validate_hash(
                    next_hash,
                    proof_.hash.into(),
                    signer.pubkey(),
                    nonce,
                    treasury.difficulty.into(),
                ) {
                    println!("Hash already validated! An earlier transaction must have landed.");
                    break 'submit;
                }

                // Reset epoch, if needed
                let treasury = get_treasury(&self.rpc_client).await;
                let clock = get_clock_account(&self.rpc_client).await;
                let threshold = treasury.last_reset_at.saturating_add(EPOCH_DURATION);
                if clock.unix_timestamp.ge(&threshold) {
                    // There are a lot of miners right now, so randomly select into submitting tx
                    if rng.gen_range(0..RESET_ODDS).eq(&0) {
                        println!("Sending epoch reset transaction...");
                        let cu_limit_ix =
                            ComputeBudgetInstruction::set_compute_unit_limit(CU_LIMIT_RESET);
                        let cu_price_ix =
                            ComputeBudgetInstruction::set_compute_unit_price(self.priority_fee);
                        let reset_ix = ore::instruction::reset(signer.pubkey());
                        self.send_and_confirm(&[cu_limit_ix, cu_price_ix, reset_ix], false, true)
                            .await
                            .ok();
                    }
                }

                // Submit request.
                let bus = self.find_bus_id(treasury.reward_rate).await;
                let bus_rewards = (bus.rewards as f64) / (10f64.powf(ore::TOKEN_DECIMALS as f64));
                println!("Sending on bus {} ({} ORZ)", bus.id, bus_rewards);
                let cu_limit_ix = ComputeBudgetInstruction::set_compute_unit_limit(CU_LIMIT_MINE);
                let cu_price_ix =
                    ComputeBudgetInstruction::set_compute_unit_price(self.priority_fee);
                let ix_mine = ore::instruction::mine(
                    signer.pubkey(),
                    BUS_ADDRESSES[bus.id as usize],
                    next_hash.into(),
                    nonce,
                );
                match self
                    .send_and_confirm(&[cu_limit_ix, cu_price_ix, ix_mine], false, false)
                    .await
                {
                    Ok(sig) => {
                        println!("Success: {}", sig);
                        break;
                    }
                    Err(_err) => {
                        // TODO
                    }
                }
            }
        }
    }

    async fn find_bus_id(&self, reward_rate: u64) -> Bus {
        let mut rng = rand::thread_rng();
        loop {
            let bus_id = rng.gen_range(0..BUS_COUNT);
            if let Ok(bus) = self.get_bus(bus_id).await {
                if bus.rewards.ge(&reward_rate.saturating_mul(0)) {
                    return bus;
                }
            }
        }
    }

    fn _find_next_hash(&self, hash: KeccakHash, difficulty: KeccakHash) -> (KeccakHash, u64) {
        let signer = self.signer();
        let mut next_hash: KeccakHash;
        let mut nonce = 0u64;
        loop {
            next_hash = hashv(&[
                hash.to_bytes().as_slice(),
                signer.pubkey().to_bytes().as_slice(),
                nonce.to_le_bytes().as_slice(),
            ]);
            if next_hash.le(&difficulty) {
                break;
            } else {
                println!("Invalid hash: {} Nonce: {:?}", next_hash.to_string(), nonce);
            }
            nonce += 1;
        }
        (next_hash, nonce)
    }

    fn find_next_hash_par(
        &self,
        hash: KeccakHash,
        difficulty: KeccakHash,
        threads: u64,
    ) -> (KeccakHash, u64) {
        let found_solution = Arc::new(AtomicBool::new(false));
        let solution = Arc::new(Mutex::<(KeccakHash, u64)>::new((
            KeccakHash::new_from_array([0; 32]),
            0,
        )));
        let signer = self.signer();
        let pubkey = signer.pubkey();
        let thread_handles: Vec<_> = (0..threads)
            .map(|i| {
                std::thread::spawn({
                    let found_solution = found_solution.clone();
                    let solution = solution.clone();
                    let mut stdout = stdout();
                    move || {
                        let n = u64::MAX.saturating_div(threads).saturating_mul(i);
                        let mut next_hash: KeccakHash;
                        let mut nonce: u64 = n;
                        loop {
                            next_hash = hashv(&[
                                hash.to_bytes().as_slice(),
                                pubkey.to_bytes().as_slice(),
                                nonce.to_le_bytes().as_slice(),
                            ]);
                            if nonce % 10_000 == 0 {
                                if found_solution.load(std::sync::atomic::Ordering::Relaxed) {
                                    return;
                                }
                                if n == 0 {
                                    stdout
                                        .write_all(
                                            format!("\r{}", next_hash.to_string()).as_bytes(),
                                        )
                                        .ok();
                                }
                            }
                            if next_hash.le(&difficulty) {
                                stdout
                                    .write_all(format!("\r{}", next_hash.to_string()).as_bytes())
                                    .ok();
                                found_solution.store(true, std::sync::atomic::Ordering::Relaxed);
                                let mut w_solution = solution.lock().expect("failed to lock mutex");
                                *w_solution = (next_hash, nonce);
                                return;
                            }
                            nonce += 1;
                        }
                    }
                })
            })
            .collect();

        for thread_handle in thread_handles {
            thread_handle.join().unwrap();
        }

        let r_solution = solution.lock().expect("Failed to get lock");
        *r_solution
    }

    pub fn validate_hash(
        &self,
        hash: KeccakHash,
        current_hash: KeccakHash,
        signer: Pubkey,
        nonce: u64,
        difficulty: KeccakHash,
    ) -> bool {
        // Validate hash correctness
        let hash_ = hashv(&[
            current_hash.as_ref(),
            signer.as_ref(),
            nonce.to_le_bytes().as_slice(),
        ]);
        if sol_memcmp(hash.as_ref(), hash_.as_ref(), HASH_BYTES) != 0 {
            return false;
        }

        // Validate hash difficulty
        if hash.gt(&difficulty) {
            return false;
        }

        true
    }

    pub async fn get_ore_display_balance(&self) -> String {
        let client = self.rpc_client.clone();
        let signer = self.signer();
        let token_account_address = spl_associated_token_account::get_associated_token_address(
            &signer.pubkey(),
            &ore::MINT_ADDRESS,
        );
        match client.get_token_account(&token_account_address).await {
            Ok(token_account) => {
                if let Some(token_account) = token_account {
                    token_account.token_amount.ui_amount_string
                } else {
                    "0.00".to_string()
                }
            }
            Err(_) => "0.00".to_string(),
        }
    }

    pub async fn mine_hashes_gpu(
        &self,
        difficulty: &Hash,
        hash_and_pubkey: &[(Hash, Pubkey)],
    ) -> (Duration, Vec<(Hash, u64)>) {
        self.mine_hashes(utils::get_gpu_nonce_worker_path(), 0, difficulty, hash_and_pubkey)
            .await
    }

    pub async fn mine_hashes(
        &self,
        worker: PathBuf,
        threads: usize,
        difficulty: &Hash,
        hash_and_pubkey: &[(Hash, Pubkey)],
    ) -> (Duration, Vec<(Hash, u64)>) {
        let mining_start = Instant::now();

        let mut child = tokio::process::Command::new(worker)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("nonce_worker failed to spawn");

        {
            let stdin = child.stdin.as_mut().unwrap();

            stdin.write_u8(threads as u8).await.unwrap();
            stdin.write_all(difficulty.as_ref()).await.unwrap();

            for (hash, pubkey) in hash_and_pubkey {
                stdin.write_all(hash.as_ref()).await.unwrap();
                stdin.write_all(pubkey.as_ref()).await.unwrap();
            }
        }

        let output = child.wait_with_output().await.unwrap().stdout;
        let mut results = vec![];

        for item in output.chunks(40) {
            let hash = Hash(item[..32].try_into().unwrap());
            let nonce = u64::from_le_bytes(item[32..40].try_into().unwrap());

            results.push((hash, nonce));
        }

        let mining_duration = mining_start.elapsed();
        (mining_duration, results)
    }
}
