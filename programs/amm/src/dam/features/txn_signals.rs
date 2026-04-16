use anchor_lang::{
    prelude::{pubkey, *},
    solana_program::sysvar::instructions::{
        load_current_index_checked, load_instruction_at_checked,
    },
};

const COMPUTE_BUDGET_PROGRAM_ID: Pubkey = pubkey!("ComputeBudget111111111111111111111111111111");
const COMPUTE_BUDGET_SET_CU_LIMIT_DISCRIMINATOR: u8 = 2;
const COMPUTE_BUDGET_SET_CU_PRICE_DISCRIMINATOR: u8 = 3;
const JUPITER_AGGREGATOR_PROGRAM_ID: Pubkey =
    pubkey!("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4");
const JUPITER_DCA_PROGRAM_ID: Pubkey = pubkey!("DCA265Vj8a9CEuX1eb1LWRnDT7uK6q1xMipnNyatn23M");
const TITAN_AGGREGATOR_PROGRAM_ID: Pubkey = pubkey!("T1TANpTeScyeqVzzgNViGDNrkQ6qHz9KrSBS4aNXvGT");
const DFLOW_AGGREGATOR_PROGRAM_ID: Pubkey = pubkey!("DF1ow4tspfHX9JwWJsAb9epbkA8hmpSEAtxXy1V27QBH");
const SYSTEM_PROGRAM_ID: Pubkey = pubkey!("11111111111111111111111111111111");
const TOKEN_PROGRAM_LEGACY_ID: Pubkey = pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
const TOKEN_2022_PROGRAM_ID: Pubkey = pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");
const SYSTEM_TRANSFER_DISCRIMINATOR_BYTE: u8 = 2;
const SPL_TOKEN_CLOSE_ACCOUNT_DISCRIMINATOR: u8 = 9;
const MEMO_PROGRAM_ID: Pubkey = pubkey!("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr");
const MEMO_PROGRAM_V1_ID: Pubkey = pubkey!("Memo1UhkJBfCR6MNLc2VgsU4sBnkDFP2SXLUQbr4XzJ");
const ATA_PROGRAM_ID: Pubkey = pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
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
const MAX_TRACKED_TOP_LEVEL_ACCOUNTS: usize = MAX_TL_SCAN as usize;
const ROUTE_ORIGIN_DIRECT: u8 = 0;
const ROUTE_ORIGIN_KNOWN_ROUTER_MEDIATED: u8 = 1;
const ROUTE_ORIGIN_UNKNOWN_MEDIATED: u8 = 2;
const ROUTE_ORIGIN_RESERVED: u8 = 3;

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
    pub route_origin_bucket: u8,
    pub route_parse_known: bool,
    pub multiple_router_families_touched: bool,
    pub prior_instruction_data_bytes_sum: u32,
    pub prior_instruction_data_bytes_max: u16,
    pub prior_nonempty_data_ix_count: u8,
    pub prior_unique_first_data_byte_count: u8,
    pub total_top_level_ix_count: u8,
    pub post_top_level_ix_count: u8,
    pub post_non_compute_budget_ix_count: u8,
    pub has_top_level_memo_ix_anywhere: bool,
    pub has_top_level_ata_create_ix_anywhere: bool,
    pub has_top_level_close_account_anywhere: bool,
    pub has_top_level_temp_token_account_create_and_close_same_tx: bool,
    pub has_jito_tip_transfer_top_level: bool,
    pub jito_tip_lamports_max_top_level: u64,
    pub jito_tip_transfer_count_top_level: u8,
    pub jito_tip_position_bucket_top_level: u8,
    pub has_non_jito_tip_transfer_top_level: bool,
    pub non_jito_tip_lamports_max_top_level: u64,
    pub non_jito_tip_transfer_count_top_level: u8,
    pub tip_service_presence_context_bucket_top_level: u8,
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
    let mut created_atas_top_level = [Pubkey::default(); MAX_TRACKED_TOP_LEVEL_ACCOUNTS];
    let mut created_atas_len = 0usize;
    let mut closed_accounts_top_level = [Pubkey::default(); MAX_TRACKED_TOP_LEVEL_ACCOUNTS];
    let mut closed_accounts_len = 0usize;
    let mut current_top_level_program_id = None;

    let mut jito_tip_before = false;
    let mut jito_tip_after = false;
    let mut non_jito_tip_before = false;
    let mut non_jito_tip_after = false;

    for idx in 0..=current_index {
        let ix = match load_instruction_at_checked(idx as usize, ix_sysvar) {
            Ok(ix) => ix,
            Err(_) => break,
        };

        if idx == current_index {
            current_top_level_program_id = Some(ix.program_id);
        }

        let router_bit = router_context_bit_for_program(&ix.program_id);
        router_bits |= router_bit;

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
            if router_bit != 0 {
                signals.prior_known_router_ix_count =
                    signals.prior_known_router_ix_count.saturating_add(1);
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

        observe_top_level_lifecycle(
            &mut signals,
            &mut created_atas_top_level,
            &mut created_atas_len,
            &mut closed_accounts_top_level,
            &mut closed_accounts_len,
            &ix.program_id,
            &ix.data,
            &ix.accounts,
        );

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

    let mut total_count: u8 = current_index.min(u8::MAX as u16) as u8;
    total_count = total_count.saturating_add(1);

    for idx in (current_index + 1)..MAX_TL_SCAN {
        let ix = match load_instruction_at_checked(idx as usize, ix_sysvar) {
            Ok(ix) => ix,
            Err(_) => break,
        };

        router_bits |= router_context_bit_for_program(&ix.program_id);
        total_count = total_count.saturating_add(1);
        signals.post_top_level_ix_count = signals.post_top_level_ix_count.saturating_add(1);
        if ix.program_id != COMPUTE_BUDGET_PROGRAM_ID {
            signals.post_non_compute_budget_ix_count =
                signals.post_non_compute_budget_ix_count.saturating_add(1);
        }

        observe_top_level_lifecycle(
            &mut signals,
            &mut created_atas_top_level,
            &mut created_atas_len,
            &mut closed_accounts_top_level,
            &mut closed_accounts_len,
            &ix.program_id,
            &ix.data,
            &ix.accounts,
        );

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
    signals.has_top_level_temp_token_account_create_and_close_same_tx = has_account_overlap(
        &created_atas_top_level,
        created_atas_len,
        &closed_accounts_top_level,
        closed_accounts_len,
    );
    signals.router_context_bucket = router_bits;
    signals.multiple_router_families_touched = router_bits.count_ones() > 1;
    signals.prior_unique_first_data_byte_count = count_marked_bytes(&first_byte_bitset);

    if let Some(current_top_level_program_id) = current_top_level_program_id.as_ref() {
        let (route_origin_bucket, route_parse_known) = route_origin_for_current_top_level_program(
            current_top_level_program_id,
            current_program_id,
        );
        signals.route_origin_bucket = route_origin_bucket;
        signals.route_parse_known = route_parse_known;
    } else {
        signals.route_origin_bucket = ROUTE_ORIGIN_RESERVED;
        signals.route_parse_known = false;
    }

    Ok(signals)
}

fn observe_top_level_lifecycle(
    signals: &mut TxnSignals,
    created_atas_top_level: &mut [Pubkey; MAX_TRACKED_TOP_LEVEL_ACCOUNTS],
    created_atas_len: &mut usize,
    closed_accounts_top_level: &mut [Pubkey; MAX_TRACKED_TOP_LEVEL_ACCOUNTS],
    closed_accounts_len: &mut usize,
    program_id: &Pubkey,
    data: &[u8],
    accounts: &[anchor_lang::solana_program::instruction::AccountMeta],
) {
    if *program_id == MEMO_PROGRAM_ID || *program_id == MEMO_PROGRAM_V1_ID {
        signals.has_top_level_memo_ix_anywhere = true;
    }

    if let Some(ata_account) = associated_token_account_created(program_id, accounts) {
        signals.has_top_level_ata_create_ix_anywhere = true;
        record_account(created_atas_top_level, created_atas_len, ata_account);
    }

    if let Some(closed_account) = parse_top_level_close_account(program_id, data, accounts) {
        signals.has_top_level_close_account_anywhere = true;
        record_account(
            closed_accounts_top_level,
            closed_accounts_len,
            closed_account,
        );
    }
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
    if data[0] != SYSTEM_TRANSFER_DISCRIMINATOR_BYTE || data[1] != 0 || data[2] != 0 || data[3] != 0
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

fn associated_token_account_created(
    program_id: &Pubkey,
    accounts: &[anchor_lang::solana_program::instruction::AccountMeta],
) -> Option<Pubkey> {
    if *program_id != ATA_PROGRAM_ID || accounts.len() < 2 {
        return None;
    }

    Some(accounts[1].pubkey)
}

fn parse_top_level_close_account(
    program_id: &Pubkey,
    data: &[u8],
    accounts: &[anchor_lang::solana_program::instruction::AccountMeta],
) -> Option<Pubkey> {
    if *program_id != TOKEN_PROGRAM_LEGACY_ID && *program_id != TOKEN_2022_PROGRAM_ID {
        return None;
    }
    if data.first().copied() != Some(SPL_TOKEN_CLOSE_ACCOUNT_DISCRIMINATOR) || accounts.is_empty() {
        return None;
    }

    Some(accounts[0].pubkey)
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

fn route_origin_for_current_top_level_program(
    current_top_level_program_id: &Pubkey,
    current_program_id: &Pubkey,
) -> (u8, bool) {
    if *current_top_level_program_id == *current_program_id {
        return (ROUTE_ORIGIN_DIRECT, true);
    }

    if router_context_bit_for_program(current_top_level_program_id) != 0 {
        return (ROUTE_ORIGIN_KNOWN_ROUTER_MEDIATED, true);
    }

    (ROUTE_ORIGIN_UNKNOWN_MEDIATED, false)
}

fn record_account(
    accounts: &mut [Pubkey; MAX_TRACKED_TOP_LEVEL_ACCOUNTS],
    len: &mut usize,
    key: Pubkey,
) {
    if *len < accounts.len() {
        accounts[*len] = key;
        *len += 1;
    }
}

fn has_account_overlap(
    lhs: &[Pubkey; MAX_TRACKED_TOP_LEVEL_ACCOUNTS],
    lhs_len: usize,
    rhs: &[Pubkey; MAX_TRACKED_TOP_LEVEL_ACCOUNTS],
    rhs_len: usize,
) -> bool {
    lhs[..lhs_len]
        .iter()
        .any(|lhs_key| rhs[..rhs_len].iter().any(|rhs_key| rhs_key == lhs_key))
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

#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::solana_program::account_info::AccountInfo;
    use anchor_lang::solana_program::instruction::{AccountMeta, Instruction};
    use anchor_lang::solana_program::sysvar::instructions::{
        construct_instructions_data, BorrowedAccountMeta, BorrowedInstruction,
        ID as INSTRUCTIONS_SYSVAR_ID,
    };
    use anchor_lang::system_program;

    fn build_ix_sysvar_account(
        instructions: &[Instruction],
        current_index: u16,
    ) -> AccountInfo<'static> {
        let borrowed_ixs: Vec<BorrowedInstruction<'_>> = instructions
            .iter()
            .map(|ix| BorrowedInstruction {
                program_id: &ix.program_id,
                accounts: ix
                    .accounts
                    .iter()
                    .map(|meta| BorrowedAccountMeta {
                        pubkey: &meta.pubkey,
                        is_signer: meta.is_signer,
                        is_writable: meta.is_writable,
                    })
                    .collect(),
                data: &ix.data,
            })
            .collect();
        let mut data = construct_instructions_data(&borrowed_ixs);
        let last_index = data.len().saturating_sub(2);
        data[last_index..].copy_from_slice(&current_index.to_le_bytes());

        let key = Box::leak(Box::new(INSTRUCTIONS_SYSVAR_ID));
        let lamports = Box::leak(Box::new(0u64));
        let owner = Box::leak(Box::new(system_program::ID));
        let data = Box::leak(data.into_boxed_slice());

        AccountInfo::new(key, false, false, lamports, data, owner, false, 0)
    }

    fn build_system_transfer_to_tip_ix(tip_dest: Pubkey, lamports: u64) -> Instruction {
        let mut data = vec![SYSTEM_TRANSFER_DISCRIMINATOR_BYTE, 0, 0, 0];
        data.extend_from_slice(&lamports.to_le_bytes());
        Instruction {
            program_id: SYSTEM_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(Pubkey::new_unique(), true),
                AccountMeta::new(tip_dest, false),
            ],
            data,
        }
    }

    fn build_ata_create_ix(created_account: Pubkey) -> Instruction {
        Instruction {
            program_id: ATA_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(Pubkey::new_unique(), false),
                AccountMeta::new(created_account, false),
            ],
            data: vec![],
        }
    }

    fn build_close_account_ix(closed_account: Pubkey) -> Instruction {
        Instruction {
            program_id: TOKEN_PROGRAM_LEGACY_ID,
            accounts: vec![AccountMeta::new(closed_account, false)],
            data: vec![SPL_TOKEN_CLOSE_ACCOUNT_DISCRIMINATOR],
        }
    }

    #[test]
    fn parse_txn_signals_uses_full_top_level_router_bitset() {
        let current_program_id = Pubkey::new_unique();
        let tx_ixs = vec![
            Instruction {
                program_id: JUPITER_AGGREGATOR_PROGRAM_ID,
                accounts: vec![],
                data: vec![1, 2, 3],
            },
            Instruction {
                program_id: current_program_id,
                accounts: vec![],
                data: vec![4, 5, 6],
            },
            Instruction {
                program_id: TITAN_AGGREGATOR_PROGRAM_ID,
                accounts: vec![],
                data: vec![7, 8, 9],
            },
        ];
        let ix_sysvar_ai = build_ix_sysvar_account(&tx_ixs, 1);

        let signals = parse_txn_signals(Some(&ix_sysvar_ai), &current_program_id).unwrap();

        assert_eq!(signals.prior_known_router_ix_count, 1);
        assert_eq!(signals.router_context_bucket, 3);
        assert!(signals.multiple_router_families_touched);
        assert_eq!(signals.route_origin_bucket, ROUTE_ORIGIN_DIRECT);
        assert!(signals.route_parse_known);
    }

    #[test]
    fn parse_txn_signals_marks_known_router_mediated_origin() {
        let current_program_id = Pubkey::new_unique();
        let tx_ixs = vec![Instruction {
            program_id: JUPITER_AGGREGATOR_PROGRAM_ID,
            accounts: vec![],
            data: vec![1, 2, 3],
        }];
        let ix_sysvar_ai = build_ix_sysvar_account(&tx_ixs, 0);

        let signals = parse_txn_signals(Some(&ix_sysvar_ai), &current_program_id).unwrap();

        assert_eq!(
            signals.route_origin_bucket,
            ROUTE_ORIGIN_KNOWN_ROUTER_MEDIATED
        );
        assert!(signals.route_parse_known);
        assert_eq!(signals.router_context_bucket, 1);
    }

    #[test]
    fn parse_txn_signals_marks_unknown_mediated_origin() {
        let current_program_id = Pubkey::new_unique();
        let tx_ixs = vec![Instruction {
            program_id: Pubkey::new_unique(),
            accounts: vec![],
            data: vec![1, 2, 3],
        }];
        let ix_sysvar_ai = build_ix_sysvar_account(&tx_ixs, 0);

        let signals = parse_txn_signals(Some(&ix_sysvar_ai), &current_program_id).unwrap();

        assert_eq!(signals.route_origin_bucket, ROUTE_ORIGIN_UNKNOWN_MEDIATED);
        assert!(!signals.route_parse_known);
    }

    #[test]
    fn parse_txn_signals_tracks_top_level_account_churn() {
        let current_program_id = Pubkey::new_unique();
        let temp_account = Pubkey::new_unique();
        let tx_ixs = vec![
            build_ata_create_ix(temp_account),
            Instruction {
                program_id: current_program_id,
                accounts: vec![],
                data: vec![1, 2, 3],
            },
            build_close_account_ix(temp_account),
        ];
        let ix_sysvar_ai = build_ix_sysvar_account(&tx_ixs, 1);

        let signals = parse_txn_signals(Some(&ix_sysvar_ai), &current_program_id).unwrap();

        assert!(signals.has_top_level_ata_create_ix_anywhere);
        assert!(signals.has_top_level_close_account_anywhere);
        assert!(signals.has_top_level_temp_token_account_create_and_close_same_tx);
    }

    #[test]
    fn parse_txn_signals_tracks_top_level_tip_buckets() {
        let current_program_id = Pubkey::new_unique();
        let tx_ixs = vec![
            build_system_transfer_to_tip_ix(JITO_TIP_ACCOUNTS[0], 900),
            Instruction {
                program_id: current_program_id,
                accounts: vec![],
                data: vec![1, 2, 3],
            },
            build_system_transfer_to_tip_ix(HELIUS_TIP_ACCOUNTS[0], 1_500),
        ];
        let ix_sysvar_ai = build_ix_sysvar_account(&tx_ixs, 1);

        let signals = parse_txn_signals(Some(&ix_sysvar_ai), &current_program_id).unwrap();

        assert!(signals.has_jito_tip_transfer_top_level);
        assert!(signals.has_non_jito_tip_transfer_top_level);
        assert_eq!(signals.jito_tip_transfer_count_top_level, 1);
        assert_eq!(signals.non_jito_tip_transfer_count_top_level, 1);
        assert_eq!(signals.jito_tip_position_bucket_top_level, 1);
        assert_eq!(signals.tip_service_presence_context_bucket_top_level, 3);
    }
}
