pub const DAM_POOL_CONFIG_SEED: &[u8] = b"dam_pool_config";
pub const DAM_CONFIG_FLAG_EMIT_RETURN_DATA: u8 = 0b0000_0001;

pub const DAM_BASE_FEATURE_COUNT: usize = 40;
pub const DAM_FEATURE_COUNT: usize = 45;
pub const DAM_MODEL_MAX_FEATURES: usize = 48;
pub const DAM_MODEL_EXPECTED_VERSION: u8 = 3;
pub const DAM_MODEL_MAX_W_SCALE_Q16: i32 = 16 * 65_536;

pub const DAM_LOGIT_CLAMP_Q16: i32 = 12 * 65_536;
pub const DAM_MAX_RISK_THRESHOLD_Q16: u16 = 65_000;

// Fee rate is represented as hundredths of a basis point.
pub const DAM_FEE_RATE_HARD_LIMIT: u32 = 100_000;
