use anchor_lang::{
    prelude::{pubkey, *},
    solana_program::{
        instruction::get_stack_height,
        sysvar::instructions::{load_current_index_checked, load_instruction_at_checked},
    },
};

const COMPUTE_BUDGET_PROGRAM_ID: Pubkey =
    pubkey!("ComputeBudget111111111111111111111111111111");
const COMPUTE_BUDGET_SET_CU_LIMIT_DISCRIMINATOR: u8 = 2;
const COMPUTE_BUDGET_SET_CU_PRICE_DISCRIMINATOR: u8 = 3;
const JUPITER_AGGREGATOR_PROGRAM_ID: Pubkey =
    pubkey!("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4");
const JUPITER_DCA_PROGRAM_ID: Pubkey =
    pubkey!("DCA265Vj8a9CEuX1eb1LWRnDT7uK6q1xMipnNyatn23M");
const TITAN_AGGREGATOR_PROGRAM_ID: Pubkey =
    pubkey!("T1TANpTeScyeqVzzgNViGDNrkQ6qHz9KrSBS4aNXvGT");
const DFLOW_AGGREGATOR_PROGRAM_ID: Pubkey =
    pubkey!("DF1ow4tspfHX9JwWJsAb9epbkA8hmpSEAtxXy1V27QBH");
const SYSTEM_PROGRAM_ID: Pubkey = pubkey!("11111111111111111111111111111111");
const SYSTEM_TRANSFER_DISCRIMINATOR_BYTE: u8 = 2;
const MEMO_PROGRAM_ID: Pubkey =
    pubkey!("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr");
const MEMO_PROGRAM_V1_ID: Pubkey =
    pubkey!("Memo1UhkJBfCR6MNLc2VgsU4sBnkDFP2SXLUQbr4XzJ");
const ATA_PROGRAM_ID: Pubkey =
    pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
const JITO_TIP_ACCOUNTS: [Pubkey; 8] = [
    pubkey!("96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5"),
    pubkey!("HFqU5x63VTqvQss8hp11i4wVV8bD44PvwucfZ2bU7gRe"),
    pubkey!("Cw8CFyM9FkoMi7K7Crf6HNQqf4uEMzpKw6QNghXLvLkY"),
    pubkey!("ADaUMid9yfUytqMBgopwjb2DTLSokTSzL1zt6iGPaS49"),
    pubkey!("DfXygSm4jCyNCybVYYK6DwvWqjKee8pbDmJGcLWNDXjh"),
    pubkey!("ADuUkR4vqLUMWXxW9gh6D6L8pMSawimctcNZ5pGwDcEt"),
    pubkey!("DttWaMuVvTiduZRnguLF7jNxTgiMBZ1hyAumKUiL2KRL"),
    pubkey!("3AVi9Tg9Uo68tJfuvoKvqKNWKkC5wPdSSdeBnizKZ6jT"),
];
const HELIUS_TIP_ACCOUNTS: [Pubkey; 10] = [
    pubkey!("4ACfpUFoaSD9bfPdeu6DBt89gB6ENTeHBXCAi87NhDEE"),
    pubkey!("D2L6yPZ2FmmmTKPgzaMKdhu6EWZcTpLy1Vhx8uvZe7NZ"),
    pubkey!("9bnz4RShgq1hAnLnZbP8kbgBg1kEmcJBYQq3gQbmnSta"),
    pubkey!("5VY91ws6B2hMmBFRsXkoAAdsPHBJwRfBht4DXox3xkwn"),
    pubkey!("2nyhqdwKcJZR2vcqCyrYsaPVdAnFoJjiksCXJ7hfEYgD"),
    pubkey!("2q5pghRs6arqVjRvT5gfgWfWcHWmw1ZuCzphgd5KfWGJ"),
    pubkey!("wyvPkWjVZz1M8fHQnMMCDTQDbkManefNNhweYk5WkcF"),
    pubkey!("3KCKozbAaF75qEU33jtzozcJ29yJuaLJTy2jFdzUY8bT"),
    pubkey!("4vieeGHPYPG2MmyPRcYjdiDmmhN3ww7hsFNap8pVN3Ey"),
    pubkey!("4TQLFNWK8AovT1gFvda5jfw2oJeRMKEmw7aH6MGBJ3or"),
];
const NOZOMI_TIP_ACCOUNTS: [Pubkey; 17] = [
    pubkey!("TEMPaMeCRFAS9EKF53Jd6KpHxgL47uWLcpFArU1Fanq"),
    pubkey!("noz3jAjPiHuBPqiSPkkugaJDkJscPuRhYnSpbi8UvC4"),
    pubkey!("noz3str9KXfpKknefHji8L1mPgimezaiUyCHYMDv1GE"),
    pubkey!("noz6uoYCDijhu1V7cutCpwxNiSovEwLdRHPwmgCGDNo"),
    pubkey!("noz9EPNcT7WH6Sou3sr3GGjHQYVkN3DNirpbvDkv9YJ"),
    pubkey!("nozc5yT15LazbLTFVZzoNZCwjh3yUtW86LoUyqsBu4L"),
    pubkey!("nozFrhfnNGoyqwVuwPAW4aaGqempx4PU6g6D9CJMv7Z"),
    pubkey!("nozievPk7HyK1Rqy1MPJwVQ7qQg2QoJGyP71oeDwbsu"),
    pubkey!("noznbgwYnBLDHu8wcQVCEw6kDrXkPdKkydGJGNXGvL7"),
    pubkey!("nozNVWs5N8mgzuD3qigrCG2UoKxZttxzZ85pvAQVrbP"),
    pubkey!("nozpEGbwx4BcGp6pvEdAh1JoC2CQGZdU6HbNP1v2p6P"),
    pubkey!("nozrhjhkCr3zXT3BiT4WCodYCUFeQvcdUkM7MqhKqge"),
    pubkey!("nozrwQtWhEdrA6W8dkbt9gnUaMs52PdAv5byipnadq3"),
    pubkey!("nozUacTVWub3cL4mJmGCYjKZTnE9RbdY5AP46iQgbPJ"),
    pubkey!("nozWCyTPppJjRuw2fpzDhhWbW355fzosWSzrrMYB1Qk"),
    pubkey!("nozWNju6dY353eMkMqURqwQEoM3SFgEKC6psLCSfUne"),
    pubkey!("nozxNBgWohjR75vdspfxR5H9ceC7XXH99xpxhVGt3Bb"),
];
const ASTRALANE_TIP_ACCOUNTS: [Pubkey; 8] = [
    pubkey!("astrazznxsGUhWShqgNtAdfrzP2G83DzcWVJDxwV9bF"),
    pubkey!("astra4uejePWneqNaJKuFFA8oonqCE1sqF6b45kDMZm"),
    pubkey!("astra9xWY93QyfG6yM8zwsKsRodscjQ2uU2HKNL5prk"),
    pubkey!("astraRVUuTHjpwEVvNBeQEgwYx9w9CFyfxjYoobCZhL"),
    pubkey!("astraEJ2fEj8Xmy6KLG7B3VfbKfsHXhHrNdCQx7iGJK"),
    pubkey!("astraubkDw81n4LuutzSQ8uzHCv4BhPVhfvTcYv8SKC"),
    pubkey!("astraZW5GLFefxNPAatceHhYjfA1ciq9gvfEg2S47xk"),
    pubkey!("astrawVNP4xDBKT7rAdxrLYiTSTdqtUr63fSMduivXK"),
];
const MAX_TL_SCAN: u16 = 64;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TxnSignals {
    pub has_compute_budget_ix: bool,
    pub has_compute_unit_price_ix: bool,
    pub compute_unit_limit: u32,
    pub compute_unit_price_micro_lamports: u64,
    pub prior_top_level_ix_count: u8,
    pub prior_non_compute_budget_ix_count: u8,
    pub prior_same_program_ix_count: u8,
    pub prior_known_router_ix_count: u8,
    pub router_context_bucket: u8,
    pub prior_instruction_data_bytes_sum: u32,
    pub prior_instruction_data_bytes_max: u16,
    pub prior_nonempty_data_ix_count: u8,
    pub prior_unique_first_data_byte_count: u8,
    pub total_top_level_ix_count: u8,
    pub post_top_level_ix_count: u8,
    pub post_non_compute_budget_ix_count: u8,
    pub has_memo_ix_anywhere: bool,
    pub has_ata_create_ix_anywhere: bool,
    pub has_jito_tip_transfer_top_level: bool,
    pub jito_tip_lamports_max_top_level: u64,
    pub jito_tip_transfer_count_top_level: u8,
    pub jito_tip_position_bucket_top_level: u8,
    pub has_non_jito_tip_transfer_top_level: bool,
    pub non_jito_tip_lamports_max_top_level: u64,
    pub non_jito_tip_transfer_count_top_level: u8,
    pub tip_service_presence_context_bucket_top_level: u8,
    pub cpi_stack_height: u8,
    pub processed_sibling_ix_count: u8,
    pub processed_sibling_unique_program_count: u8,
    pub processed_sibling_ix_data_sum: u32,
    pub has_jito_tip_transfer_sibling: bool,
    pub jito_tip_lamports_max_sibling: u64,
    pub jito_tip_presence_context_bucket: u8,
}

pub fn parse_txn_signals(
    ix_sysvar: Option<&AccountInfo<'_>>,
    current_program_id: &Pubkey,
) -> Result<TxnSignals> {
    let Some(ix_sysvar) = ix_sysvar else {
        return Ok(TxnSignals::default());
    };

    let current_index = load_current_index_checked(ix_sysvar)?;
    let mut signals = TxnSignals::default();
    let mut router_bits: u8 = 0;
    let mut first_byte_bitset = [0u64; 4];

    let mut jito_tip_before = false;
    let mut jito_tip_after = false;
    let mut non_jito_tip_before = false;
    let mut non_jito_tip_after = false;

    for idx in 0..=current_index {
        let ix = match load_instruction_at_checked(idx as usize, ix_sysvar) {
            Ok(ix) => ix,
            Err(_) => break,
        };

        if idx < current_index {
            signals.prior_top_level_ix_count = signals.prior_top_level_ix_count.saturating_add(1);
            if ix.program_id != COMPUTE_BUDGET_PROGRAM_ID {
                signals.prior_non_compute_budget_ix_count =
                    signals.prior_non_compute_budget_ix_count.saturating_add(1);
            }
            if ix.program_id == *current_program_id {
                signals.prior_same_program_ix_count =
                    signals.prior_same_program_ix_count.saturating_add(1);
            }

            let router_bit = router_context_bit_for_program(&ix.program_id);
            if router_bit != 0 {
                signals.prior_known_router_ix_count =
                    signals.prior_known_router_ix_count.saturating_add(1);
                router_bits |= router_bit;
            }

            if !ix.data.is_empty() {
                signals.prior_nonempty_data_ix_count =
                    signals.prior_nonempty_data_ix_count.saturating_add(1);

                let data_len_u64 = u64::try_from(ix.data.len()).unwrap_or(u64::MAX);
                let sum = u64::from(signals.prior_instruction_data_bytes_sum)
                    .saturating_add(data_len_u64)
                    .min(u64::from(u32::MAX));
                signals.prior_instruction_data_bytes_sum = sum as u32;

                let data_len_u16 = u16::try_from(ix.data.len()).unwrap_or(u16::MAX);
                if data_len_u16 > signals.prior_instruction_data_bytes_max {
                    signals.prior_instruction_data_bytes_max = data_len_u16;
                }

                mark_first_byte(&mut first_byte_bitset, ix.data[0]);
            }

            if let Some((tip_service, lamports)) =
                parse_system_transfer_to_tip_service(&ix.program_id, &ix.data, &ix.accounts)
            {
                match tip_service {
                    TipService::Jito => {
                        jito_tip_before = true;
                        signals.jito_tip_transfer_count_top_level =
                            signals.jito_tip_transfer_count_top_level.saturating_add(1);
                        if lamports > signals.jito_tip_lamports_max_top_level {
                            signals.jito_tip_lamports_max_top_level = lamports;
                        }
                    }
                    TipService::NonJito => {
                        non_jito_tip_before = true;
                        signals.non_jito_tip_transfer_count_top_level = signals
                            .non_jito_tip_transfer_count_top_level
                            .saturating_add(1);
                        if lamports > signals.non_jito_tip_lamports_max_top_level {
                            signals.non_jito_tip_lamports_max_top_level = lamports;
                        }
                    }
                }
            }
        }

        if ix.program_id == MEMO_PROGRAM_ID || ix.program_id == MEMO_PROGRAM_V1_ID {
            signals.has_memo_ix_anywhere = true;
        }
        if ix.program_id == ATA_PROGRAM_ID {
            signals.has_ata_create_ix_anywhere = true;
        }

        if ix.program_id == COMPUTE_BUDGET_PROGRAM_ID {
            signals.has_compute_budget_ix = true;
            let (cu_limit, cu_price, has_cu_price_ix) = parse_compute_budget_instruction(&ix.data);
            if let Some(cu_limit) = cu_limit {
                signals.compute_unit_limit = cu_limit;
            }
            if has_cu_price_ix {
                signals.has_compute_unit_price_ix = true;
            }
            if let Some(cu_price) = cu_price {
                signals.compute_unit_price_micro_lamports = cu_price;
            }
        }
    }

    if let Ok(current_ix) = load_instruction_at_checked(current_index as usize, ix_sysvar) {
        router_bits |= router_context_bit_for_program(&current_ix.program_id);
    }

    let mut total_count: u8 = current_index.min(u8::MAX as u16) as u8;
    total_count = total_count.saturating_add(1);

    for idx in (current_index + 1)..MAX_TL_SCAN {
        let ix = match load_instruction_at_checked(idx as usize, ix_sysvar) {
            Ok(ix) => ix,
            Err(_) => break,
        };

        total_count = total_count.saturating_add(1);
        signals.post_top_level_ix_count = signals.post_top_level_ix_count.saturating_add(1);
        if ix.program_id != COMPUTE_BUDGET_PROGRAM_ID {
            signals.post_non_compute_budget_ix_count =
                signals.post_non_compute_budget_ix_count.saturating_add(1);
        }

        if ix.program_id == MEMO_PROGRAM_ID || ix.program_id == MEMO_PROGRAM_V1_ID {
            signals.has_memo_ix_anywhere = true;
        }
        if ix.program_id == ATA_PROGRAM_ID {
            signals.has_ata_create_ix_anywhere = true;
        }

        if let Some((tip_service, lamports)) =
            parse_system_transfer_to_tip_service(&ix.program_id, &ix.data, &ix.accounts)
        {
            match tip_service {
                TipService::Jito => {
                    jito_tip_after = true;
                    signals.jito_tip_transfer_count_top_level =
                        signals.jito_tip_transfer_count_top_level.saturating_add(1);
                    if lamports > signals.jito_tip_lamports_max_top_level {
                        signals.jito_tip_lamports_max_top_level = lamports;
                    }
                }
                TipService::NonJito => {
                    non_jito_tip_after = true;
                    signals.non_jito_tip_transfer_count_top_level = signals
                        .non_jito_tip_transfer_count_top_level
                        .saturating_add(1);
                    if lamports > signals.non_jito_tip_lamports_max_top_level {
                        signals.non_jito_tip_lamports_max_top_level = lamports;
                    }
                }
            }
        }
    }

    signals.total_top_level_ix_count = total_count;
    signals.has_jito_tip_transfer_top_level = jito_tip_before || jito_tip_after;
    signals.has_non_jito_tip_transfer_top_level = non_jito_tip_before || non_jito_tip_after;
    signals.jito_tip_position_bucket_top_level = match (jito_tip_before, jito_tip_after) {
        (false, false) => 0,
        (true, false) => 1,
        (false, true) => 2,
        (true, true) => 3,
    };
    signals.tip_service_presence_context_bucket_top_level = match (
        signals.has_jito_tip_transfer_top_level,
        signals.has_non_jito_tip_transfer_top_level,
    ) {
        (false, false) => 0,
        (true, false) => 1,
        (false, true) => 2,
        (true, true) => 3,
    };

    signals.router_context_bucket = router_bits;
    signals.prior_unique_first_data_byte_count = count_marked_bytes(&first_byte_bitset);

    // Sibling-only fields stay zero until we add a direct sibling-instruction syscall shim.
    signals.cpi_stack_height = (get_stack_height() as u8).min(127);
    signals.jito_tip_presence_context_bucket = match (
        signals.has_jito_tip_transfer_top_level,
        signals.has_jito_tip_transfer_sibling,
    ) {
        (false, false) => 0,
        (true, false) => 1,
        (false, true) => 2,
        (true, true) => 3,
    };

    Ok(signals)
}

fn parse_compute_budget_instruction(data: &[u8]) -> (Option<u32>, Option<u64>, bool) {
    let Some(discriminator) = data.first().copied() else {
        return (None, None, false);
    };

    if discriminator == COMPUTE_BUDGET_SET_CU_LIMIT_DISCRIMINATOR {
        if data.len() >= 5 {
            let mut bytes = [0u8; 4];
            bytes.copy_from_slice(&data[1..5]);
            return (Some(u32::from_le_bytes(bytes)), None, false);
        }
        return (None, None, false);
    }

    if discriminator == COMPUTE_BUDGET_SET_CU_PRICE_DISCRIMINATOR {
        if data.len() >= 9 {
            let mut bytes = [0u8; 8];
            bytes.copy_from_slice(&data[1..9]);
            return (None, Some(u64::from_le_bytes(bytes)), true);
        }
        return (None, None, true);
    }

    (None, None, false)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TipService {
    Jito,
    NonJito,
}

fn parse_system_transfer_to_tip_service(
    program_id: &Pubkey,
    data: &[u8],
    accounts: &[anchor_lang::solana_program::instruction::AccountMeta],
) -> Option<(TipService, u64)> {
    if *program_id != SYSTEM_PROGRAM_ID {
        return None;
    }
    if data.len() < 12 {
        return None;
    }
    if data[0] != SYSTEM_TRANSFER_DISCRIMINATOR_BYTE
        || data[1] != 0
        || data[2] != 0
        || data[3] != 0
    {
        return None;
    }
    if accounts.len() < 2 {
        return None;
    }
    let Some(tip_service) = classify_tip_account(&accounts[1].pubkey) else {
        return None;
    };
    let lamports = u64::from_le_bytes(data[4..12].try_into().ok()?);
    Some((tip_service, lamports))
}

fn classify_tip_account(pubkey: &Pubkey) -> Option<TipService> {
    if JITO_TIP_ACCOUNTS.iter().any(|tip| tip == pubkey) {
        Some(TipService::Jito)
    } else if HELIUS_TIP_ACCOUNTS.iter().any(|tip| tip == pubkey)
        || NOZOMI_TIP_ACCOUNTS.iter().any(|tip| tip == pubkey)
        || ASTRALANE_TIP_ACCOUNTS.iter().any(|tip| tip == pubkey)
    {
        Some(TipService::NonJito)
    } else {
        None
    }
}

fn router_context_bit_for_program(program_id: &Pubkey) -> u8 {
    if *program_id == JUPITER_AGGREGATOR_PROGRAM_ID || *program_id == JUPITER_DCA_PROGRAM_ID {
        1
    } else if *program_id == TITAN_AGGREGATOR_PROGRAM_ID {
        2
    } else if *program_id == DFLOW_AGGREGATOR_PROGRAM_ID {
        4
    } else {
        0
    }
}

fn mark_first_byte(bitset: &mut [u64; 4], byte: u8) {
    let idx = usize::from(byte / 64);
    let bit = u32::from(byte % 64);
    if idx < bitset.len() {
        bitset[idx] |= 1u64 << bit;
    }
}

fn count_marked_bytes(bitset: &[u64; 4]) -> u8 {
    let sum = bitset.iter().map(|x| x.count_ones()).sum::<u32>().min(255);
    sum as u8
}
