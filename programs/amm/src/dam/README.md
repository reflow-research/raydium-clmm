# DAM Layout

This folder is the direct Raydium-side DAM integration slice.

## Structure

- `accounts.rs`
  - Anchor account/state for DAM pool config and shared model weights.
- `features/`
  - Raydium swap feature extraction and feature-frame adapters for the
    canonical DAM CPI structural atomic v1 ABI.
- `model/`
  - Pure inference code. No account loading and no instruction context.
- `policy/`
  - Risk-threshold and fee-upcharge policy.
- `schema.rs`
  - Frozen DAM v1 slot map, schema identifier, and intentional-zero slot list.
- `engine.rs`
  - One orchestration function that composes feature source, model, and policy.
- `traits.rs`
  - Static-dispatch seams for feature writing, inference, and fee policy.
- `types.rs`
  - Fee/risk/logit newtypes and the const-generic feature vector.

## Zero-Cost Abstractions

- `FeatureVector<const N: usize>`
  - Feature width is fixed at compile time. No heap allocation and no runtime shape checks in the hot path.
- `DamFeatureSource<const N: usize>`
  - Feature writers are generic and monomorphized per caller. No trait objects.
- `DamInferenceModel<const N: usize>`
  - Model execution stays pure and statically dispatched. The swap path can inline it.
- `DamFeePolicy`
  - Fee widening is policy-driven without introducing dynamic dispatch or registry plumbing.
- Account adapters
  - `DamPoolConfig::fee_policy()` and `DamModelWeights::linear_model()` convert account state into pure policy/model values at the edge, so the swap loop can run on plain data.

## Raydium Splice Points

- `instructions/swap_v2.rs`
  - Builds the `RaydiumSwapObservation`, loads DAM remaining accounts, runs the
    canonical 43-slot DAM v1 feature and inference path, and passes the fee add
    into swap math.
- `instructions/swap.rs`
  - Legacy swap entrypoint reuses the same DAM seam and packed ABI.
- `libraries/swap_math.rs`
  - Keeps swap-step math pure; DAM only changes the fee-rate input.
- `instructions/admin/`
  - Hosts DAM init/update handlers for model weights and pool config.

## Remaining Accounts

- `swap_v2` tail accounts:
  - `[dam_pool_config, dam_model, instructions_sysvar]`
- `instructions_sysvar` is required when DAM is enabled so the feature frame
  stays aligned with the canonical DAM CPI structural atomic v1 transaction-
  signal layout.
