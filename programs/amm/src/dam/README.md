# DAM Layout

This folder is the direct Raydium-side DAM integration slice.

## Structure

- `accounts.rs`
  - Anchor account/state for DAM pool config and shared model weights.
- `features/`
  - Raydium swap feature extraction and feature-frame adapters.
- `model/`
  - Pure inference code. No account loading and no instruction context.
- `policy/`
  - Risk-threshold and fee-upcharge policy.
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

## Planned Raydium Splice Points

- `instructions/swap_v2.rs`
  - Build the `RaydiumSwapObservation`, load DAM remaining accounts, run the legacy-compatible DAM feature and inference path, and pass the fee add into swap math.
- `instructions/swap.rs`
  - Legacy path can reuse the same engine through a narrower feature adapter.
- `libraries/swap_math.rs`
  - Keep swap-step math pure; DAM only changes the fee-rate input.
- `instructions/admin/`
  - Add DAM init/update handlers for model weights and pool config after the swap-side seam is wired.

## Remaining Accounts

- `swap_v2` tail accounts:
  - `[dam_pool_config, dam_model, instructions_sysvar]`
- `instructions_sysvar` is required when DAM is enabled so the feature frame stays aligned with the original DAM transaction-signal model layout.
