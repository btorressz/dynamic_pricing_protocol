use anchor_lang::prelude::*;
use pyth_sdk_solana::state::load_price_account;

declare_id!("5QxHEDXwD6yBCdcxZ9daqUJLgCyjVuA6YXq58nPLHy1n");

#[program]
pub mod dynamic_pricing_protocol {
    use super::*;

    // Initialize price data for a product/service
    pub fn initialize_price_data(ctx: Context<InitializePriceData>, initial_price: u64) -> Result<()> {
        let price_data = &mut ctx.accounts.price_data;
        price_data.price = initial_price;
        price_data.demand = 0;
        price_data.supply = 100; // Default supply
        price_data.total_liquidity = 0;
        Ok(())
    }

    // Update price manually (could be used by admin or authorized signer)
    pub fn update_price(ctx: Context<UpdatePrice>, new_price: u64) -> Result<()> {
        let price_data = &mut ctx.accounts.price_data;
        require!(new_price > 0, PricingError::InvalidPrice);
        price_data.price = new_price;
        Ok(())
    }

    // Fetch price from an external oracle (e.g., Pyth)
    pub fn fetch_price_from_oracle(ctx: Context<FetchPrice>) -> Result<()> {
        let price_account_info = &ctx.accounts.price_account;
        let price_data_ref = price_account_info.try_borrow_data().map_err(|_| CustomError::OracleError)?;
        let price_account = load_price_account(&price_data_ref).map_err(|_| CustomError::OracleError)?;

        let oracle_price = price_account.agg.price;
        let price_data = &mut ctx.accounts.price_data;
        price_data.price = oracle_price as u64;

        Ok(())
    }

    // Liquidity providers can contribute liquidity and earn rewards
    pub fn contribute_liquidity(ctx: Context<ContributeLiquidity>, amount: u64) -> Result<()> {
        let liquidity_provider = &mut ctx.accounts.liquidity_provider;
        let price_data = &mut ctx.accounts.price_data;

        liquidity_provider.liquidity += amount;
        liquidity_provider.rewards += amount / 100;
        price_data.total_liquidity += amount;

        Ok(())
    }

    // Vesting of rewards for liquidity providers
    pub fn vest_rewards(ctx: Context<VestRewards>, amount: u64) -> Result<()> {
        let liquidity_provider = &mut ctx.accounts.liquidity_provider;
        let current_time = Clock::get()?.unix_timestamp;

        require!(current_time > liquidity_provider.last_reward_claim + 86400 * 7, PricingError::VestingPeriodNotMet);
        require!(liquidity_provider.rewards >= amount, PricingError::InsufficientRewards);
        liquidity_provider.rewards -= amount;
        liquidity_provider.last_reward_claim = current_time;

        Ok(())
    }

    // Distribute revenue to liquidity providers
    pub fn distribute_revenue(ctx: Context<DistributeRevenue>, total_revenue: u64) -> Result<()> {
        let liquidity_provider = &mut ctx.accounts.liquidity_provider;
        let price_data = &ctx.accounts.price_data;

        let share = (liquidity_provider.liquidity as f64 / price_data.total_liquidity as f64) * total_revenue as f64;
        liquidity_provider.rewards += share as u64;

        Ok(())
    }

    // Governance token distribution to liquidity providers
    pub fn distribute_governance_tokens(ctx: Context<DistributeGovernanceTokens>, amount: u64) -> Result<()> {
        let governance_token_account = &mut ctx.accounts.governance_token_account;
        governance_token_account.balance += amount;
        Ok(())
    }

    // Adjust price automatically based on supply-demand ratio
    pub fn adjust_price_based_on_supply_demand(ctx: Context<UpdatePrice>) -> Result<()> {
        let price_data = &mut ctx.accounts.price_data;

        if price_data.demand > price_data.supply {
            price_data.price += (price_data.demand - price_data.supply) / 10;
        } else {
            price_data.price -= (price_data.supply - price_data.demand) / 10;
        }

        Ok(())
    }

    // Buy an asset (optional buy/sell mechanism)
    pub fn buy_asset(ctx: Context<BuyAsset>, amount: u64) -> Result<()> {
        let price_data = &mut ctx.accounts.price_data;
        let buyer = &ctx.accounts.buyer;

        let total_cost = price_data.price * amount;
        require!(buyer.lamports() >= total_cost, PricingError::InsufficientBalance);

        price_data.supply -= amount;
        price_data.demand += amount;

        Ok(())
    }

    // Dynamic fee mechanism based on liquidity depth
    pub fn apply_dynamic_fee(ctx: Context<ApplyFee>, transaction_amount: u64) -> Result<u64> {
        let price_data = &ctx.accounts.price_data;

        let fee_percentage = if price_data.total_liquidity < 1000 {
            5 // 5% fee for low liquidity
        } else if price_data.total_liquidity < 5000 {
            2 // 2% fee for medium liquidity
        } else {
            1 // 1% fee for high liquidity
        };

        let fee = (transaction_amount * fee_percentage) / 100;
        Ok(fee)
    }

    // Slash rewards for inactive liquidity providers
    pub fn slash_inactive_provider(ctx: Context<SlashInactive>, inactivity_period: i64) -> Result<()> {
        let liquidity_provider = &mut ctx.accounts.liquidity_provider;
        let current_time = Clock::get()?.unix_timestamp;

        if current_time - liquidity_provider.last_reward_claim > inactivity_period {
            liquidity_provider.rewards /= 2; // Slash 50% of the rewards
        }

        Ok(())
    }

    // Smooth price changes for volatile markets
    pub fn smooth_price(ctx: Context<SmoothPrice>, new_price: u64, smoothing_factor: u64) -> Result<()> {
        let price_data = &mut ctx.accounts.price_data;
        price_data.price = ((price_data.price * smoothing_factor) + new_price) / (smoothing_factor + 1);

        Ok(())
    }

    // Propose a fee change (governance mechanism)
    pub fn propose_fee_change(ctx: Context<ProposeFeeChange>, new_fee_percentage: u64) -> Result<()> {
        let governance = &mut ctx.accounts.governance_data;
        require!(new_fee_percentage <= 10, GovernanceError::InvalidFeePercentage);

        governance.proposed_fee_percentage = new_fee_percentage;
        governance.votes_for = 0;
        governance.votes_against = 0;

        Ok(())
    }

    // Cast a vote for a governance proposal
    pub fn vote_on_proposal(ctx: Context<Vote>, vote_for: bool) -> Result<()> {
        let governance = &mut ctx.accounts.governance_data;
        let voter = &mut ctx.accounts.governance_token_account;

        if vote_for {
            governance.votes_for += voter.balance;
        } else {
            governance.votes_against += voter.balance;
        }

        Ok(())
    }

    // Contribute to the insurance pool for liquidity providers
    pub fn contribute_to_insurance_pool(ctx: Context<ContributeToPool>, amount: u64) -> Result<()> {
        let insurance_pool = &mut ctx.accounts.insurance_pool;
        insurance_pool.total_funds += amount;

        Ok(())
    }

    // Claim from the insurance pool
    pub fn claim_insurance(ctx: Context<ClaimInsurance>, amount: u64) -> Result<()> {
        let insurance_pool = &mut ctx.accounts.insurance_pool;
        let liquidity_provider = &mut ctx.accounts.liquidity_provider;

        require!(insurance_pool.total_funds >= amount, InsuranceError::InsufficientFunds);

        insurance_pool.total_funds -= amount;
        liquidity_provider.rewards += amount;

        Ok(())
    }

    // Register a referral
    pub fn register_referral(ctx: Context<RegisterReferral>, referrer: Pubkey) -> Result<()> {
        let user_account = &mut ctx.accounts.user_account;
        user_account.referrer = Some(referrer);
        Ok(())
    }

    // Reward a referrer
    pub fn reward_referrer(ctx: Context<RewardReferrer>, amount: u64) -> Result<()> {
        let user_account = &ctx.accounts.user_account;
        let referrer_account = &mut ctx.accounts.referrer_account;

        if let Some(referrer) = user_account.referrer {
            referrer_account.rewards += amount / 10; // 10% referral reward
        }

        Ok(())
    }
}

// Structs for enhanced features

#[account]
pub struct InsurancePool {
    pub total_funds: u64,
}

#[account]
pub struct GovernanceData {
    pub proposed_fee_percentage: u64,
    pub votes_for: u64,
    pub votes_against: u64,
}

#[account]
pub struct UserAccount {
    pub referrer: Option<Pubkey>,
}

// Core price data and liquidity provider structs

#[account]
pub struct PriceData {
    pub price: u64,           // Current price of the asset
    pub demand: u64,          // Demand data (updated based on actual transactions)
    pub supply: u64,          // Supply of the asset
    pub total_liquidity: u64, // Total liquidity provided
}

#[account]
pub struct LiquidityProvider {
    pub liquidity: u64,          // Liquidity contributed by the provider
    pub rewards: u64,            // Rewards earned by the provider
    pub last_reward_claim: i64,  // Last time rewards were claimed (for vesting)
}

#[account]
pub struct GovernanceTokenAccount {
    pub balance: u64,  // Number of governance tokens held by the provider
}

// Contexts for all actions

#[derive(Accounts)]
pub struct InitializePriceData<'info> {
    #[account(init, payer = user, space = 8 + 8 + 8 + 8 + 8)] // 8 bytes for discriminator, 4 fields each u64
    pub price_data: Account<'info, PriceData>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdatePrice<'info> {
    #[account(mut)]
    pub price_data: Account<'info, PriceData>,
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct FetchPrice<'info> {
    #[account(mut)]
    pub price_data: Account<'info, PriceData>,
    #[account(mut)]
    pub price_account: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct ContributeLiquidity<'info> {
    #[account(mut)]
    pub liquidity_provider: Account<'info, LiquidityProvider>,
    #[account(mut)]
    pub price_data: Account<'info, PriceData>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct VestRewards<'info> {
    #[account(mut)]
    pub liquidity_provider: Account<'info, LiquidityProvider>,
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct DistributeRevenue<'info> {
    #[account(mut)]
    pub liquidity_provider: Account<'info, LiquidityProvider>,
    pub price_data: Account<'info, PriceData>,
}

#[derive(Accounts)]
pub struct DistributeGovernanceTokens<'info> {
    #[account(mut)]
    pub governance_token_account: Account<'info, GovernanceTokenAccount>,
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct ApplyFee<'info> {
    pub price_data: Account<'info, PriceData>,
}

#[derive(Accounts)]
pub struct SlashInactive<'info> {
    #[account(mut)]
    pub liquidity_provider: Account<'info, LiquidityProvider>,
}

#[derive(Accounts)]
pub struct SmoothPrice<'info> {
    #[account(mut)]
    pub price_data: Account<'info, PriceData>,
}

#[derive(Accounts)]
pub struct ProposeFeeChange<'info> {
    #[account(mut)]
    pub governance_data: Account<'info, GovernanceData>,
}

#[derive(Accounts)]
pub struct Vote<'info> {
    #[account(mut)]
    pub governance_data: Account<'info, GovernanceData>,
    #[account(mut)]
    pub governance_token_account: Account<'info, GovernanceTokenAccount>,
}

#[derive(Accounts)]
pub struct ContributeToPool<'info> {
    #[account(mut)]
    pub insurance_pool: Account<'info, InsurancePool>,
}

#[derive(Accounts)]
pub struct ClaimInsurance<'info> {
    #[account(mut)]
    pub insurance_pool: Account<'info, InsurancePool>,
    #[account(mut)]
    pub liquidity_provider: Account<'info, LiquidityProvider>,
}

#[derive(Accounts)]
pub struct RegisterReferral<'info> {
    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,
}

#[derive(Accounts)]
pub struct RewardReferrer<'info> {
    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,
    #[account(mut)]
    pub referrer_account: Account<'info, LiquidityProvider>,
}

#[derive(Accounts)]
pub struct BuyAsset<'info> {
    #[account(mut)]
    pub price_data: Account<'info, PriceData>,
    #[account(mut)]
    pub buyer: Signer<'info>,
}

// Custom error handling

#[error_code]
pub enum InsuranceError {
    #[msg("Insufficient funds in the insurance pool.")]
    InsufficientFunds,
}

#[error_code]
pub enum GovernanceError {
    #[msg("Invalid fee percentage proposed.")]
    InvalidFeePercentage,
}

#[error_code]
pub enum PricingError {
    #[msg("Price cannot be negative.")]
    InvalidPrice,
    #[msg("Insufficient balance to complete the purchase.")]
    InsufficientBalance,
    #[msg("Vesting period has not been met.")]
    VestingPeriodNotMet,
    #[msg("Not enough rewards to claim.")]
    InsufficientRewards,
}

#[error_code]
pub enum CustomError {
    #[msg("Failed to load data from Oracle.")]
    OracleError,
}
