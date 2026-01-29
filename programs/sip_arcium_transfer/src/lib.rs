use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;

/// SIP Private Transfer Program
///
/// Solana program that queues encrypted computations to Arcium MXE.
/// Works with the `encrypted-ixs` circuits for:
/// - Private transfers (balance hidden)
/// - Balance checks (threshold validation)
/// - Confidential swaps (DEX with hidden amounts)
///
/// @see https://github.com/sip-protocol/sip-mobile/issues/73

const COMP_DEF_OFFSET_PRIVATE_TRANSFER: u32 = comp_def_offset("private_transfer");
const COMP_DEF_OFFSET_CHECK_BALANCE: u32 = comp_def_offset("check_balance");
const COMP_DEF_OFFSET_VALIDATE_SWAP: u32 = comp_def_offset("validate_swap");

declare_id!("S1P5q5497A6oRCUutUFb12LkNQynTNoEyRyUvotmcX9");

#[arcium_program]
pub mod sip_arcium_transfer {
    use super::*;

    // =========================================================================
    // INITIALIZATION
    // =========================================================================

    /// Initialize the private_transfer computation definition
    pub fn init_private_transfer_comp_def(
        ctx: Context<InitPrivateTransferCompDef>,
    ) -> Result<()> {
        init_comp_def(ctx.accounts, None, None)?;
        Ok(())
    }

    /// Initialize the check_balance computation definition
    pub fn init_check_balance_comp_def(ctx: Context<InitCheckBalanceCompDef>) -> Result<()> {
        init_comp_def(ctx.accounts, None, None)?;
        Ok(())
    }

    /// Initialize the validate_swap computation definition
    pub fn init_validate_swap_comp_def(ctx: Context<InitValidateSwapCompDef>) -> Result<()> {
        init_comp_def(ctx.accounts, None, None)?;
        Ok(())
    }

    // =========================================================================
    // PRIVATE TRANSFER
    // =========================================================================

    /// Queue a private transfer computation
    ///
    /// # Arguments
    /// * `computation_offset` - Unique offset for this computation
    /// * `encrypted_sender_balance` - Encrypted u64 (32 bytes)
    /// * `encrypted_amount` - Encrypted u64 (32 bytes)
    /// * `encrypted_min_balance` - Encrypted u64 (32 bytes)
    /// * `pubkey` - X25519 public key for result encryption
    /// * `nonce` - Encryption nonce
    pub fn private_transfer(
        ctx: Context<PrivateTransfer>,
        computation_offset: u64,
        encrypted_sender_balance: [u8; 32],
        encrypted_amount: [u8; 32],
        encrypted_min_balance: [u8; 32],
        pubkey: [u8; 32],
        nonce: u128,
    ) -> Result<()> {
        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;

        // Build args matching PrivateTransferInput struct order
        let args = ArgBuilder::new()
            .x25519_pubkey(pubkey)
            .plaintext_u128(nonce)
            .encrypted_u64(encrypted_sender_balance)
            .encrypted_u64(encrypted_amount)
            .encrypted_u64(encrypted_min_balance)
            .build();

        queue_computation(
            ctx.accounts,
            computation_offset,
            args,
            None,
            vec![PrivateTransferCallback::callback_ix(
                computation_offset,
                &ctx.accounts.mxe_account,
                &[],
            )?],
            1,
            0,
        )?;

        Ok(())
    }

    /// Callback for private transfer result
    #[arcium_callback(encrypted_ix = "private_transfer")]
    pub fn private_transfer_callback(
        ctx: Context<PrivateTransferCallback>,
        output: SignedComputationOutputs<PrivateTransferOutput>,
    ) -> Result<()> {
        let verified = output.verify_output(
            &ctx.accounts.cluster_account,
            &ctx.accounts.computation_account,
        );

        match verified {
            Ok(PrivateTransferOutput { field_0 }) => {
                // field_0.ciphertexts[0] = is_valid (bool)
                // field_0.ciphertexts[1] = new_sender_balance (u64)
                emit!(PrivateTransferEvent {
                    is_valid: field_0.ciphertexts[0],
                    new_sender_balance: field_0.ciphertexts[1],
                    nonce: field_0.nonce.to_le_bytes(),
                });
            }
            Err(_) => return Err(ErrorCode::AbortedComputation.into()),
        }

        Ok(())
    }

    // =========================================================================
    // BALANCE CHECK
    // =========================================================================

    /// Queue a balance check computation
    pub fn check_balance(
        ctx: Context<CheckBalance>,
        computation_offset: u64,
        encrypted_balance: [u8; 32],
        encrypted_minimum: [u8; 32],
        pubkey: [u8; 32],
        nonce: u128,
    ) -> Result<()> {
        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;

        let args = ArgBuilder::new()
            .x25519_pubkey(pubkey)
            .plaintext_u128(nonce)
            .encrypted_u64(encrypted_balance)
            .encrypted_u64(encrypted_minimum)
            .build();

        queue_computation(
            ctx.accounts,
            computation_offset,
            args,
            None,
            vec![CheckBalanceCallback::callback_ix(
                computation_offset,
                &ctx.accounts.mxe_account,
                &[],
            )?],
            1,
            0,
        )?;

        Ok(())
    }

    #[arcium_callback(encrypted_ix = "check_balance")]
    pub fn check_balance_callback(
        ctx: Context<CheckBalanceCallback>,
        output: SignedComputationOutputs<CheckBalanceOutput>,
    ) -> Result<()> {
        let verified = output.verify_output(
            &ctx.accounts.cluster_account,
            &ctx.accounts.computation_account,
        );

        match verified {
            Ok(CheckBalanceOutput { field_0 }) => {
                emit!(BalanceCheckEvent {
                    meets_minimum: field_0.ciphertexts[0],
                    nonce: field_0.nonce.to_le_bytes(),
                });
            }
            Err(_) => return Err(ErrorCode::AbortedComputation.into()),
        }

        Ok(())
    }

    // =========================================================================
    // VALIDATE SWAP
    // =========================================================================

    /// Queue a confidential swap validation
    pub fn validate_swap(
        ctx: Context<ValidateSwap>,
        computation_offset: u64,
        encrypted_input_balance: [u8; 32],
        encrypted_input_amount: [u8; 32],
        encrypted_min_output: [u8; 32],
        encrypted_actual_output: [u8; 32],
        pubkey: [u8; 32],
        nonce: u128,
    ) -> Result<()> {
        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;

        let args = ArgBuilder::new()
            .x25519_pubkey(pubkey)
            .plaintext_u128(nonce)
            .encrypted_u64(encrypted_input_balance)
            .encrypted_u64(encrypted_input_amount)
            .encrypted_u64(encrypted_min_output)
            .encrypted_u64(encrypted_actual_output)
            .build();

        queue_computation(
            ctx.accounts,
            computation_offset,
            args,
            None,
            vec![ValidateSwapCallback::callback_ix(
                computation_offset,
                &ctx.accounts.mxe_account,
                &[],
            )?],
            1,
            0,
        )?;

        Ok(())
    }

    #[arcium_callback(encrypted_ix = "validate_swap")]
    pub fn validate_swap_callback(
        ctx: Context<ValidateSwapCallback>,
        output: SignedComputationOutputs<ValidateSwapOutput>,
    ) -> Result<()> {
        let verified = output.verify_output(
            &ctx.accounts.cluster_account,
            &ctx.accounts.computation_account,
        );

        match verified {
            Ok(ValidateSwapOutput { field_0 }) => {
                // field_0.ciphertexts[0] = is_valid (bool)
                // field_0.ciphertexts[1] = new_input_balance (u64)
                // field_0.ciphertexts[2] = slippage_ok (bool)
                emit!(SwapValidationEvent {
                    is_valid: field_0.ciphertexts[0],
                    new_input_balance: field_0.ciphertexts[1],
                    slippage_ok: field_0.ciphertexts[2],
                    nonce: field_0.nonce.to_le_bytes(),
                });
            }
            Err(_) => return Err(ErrorCode::AbortedComputation.into()),
        }

        Ok(())
    }
}

// =============================================================================
// ACCOUNT STRUCTURES
// =============================================================================

// Private Transfer Accounts
#[queue_computation_accounts("private_transfer", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct PrivateTransfer<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init_if_needed,
        space = 9,
        payer = payer,
        seeds = [&SIGN_PDA_SEED],
        bump,
        address = derive_sign_pda!(),
    )]
    pub sign_pda_account: Account<'info, ArciumSignerAccount>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Account<'info, MXEAccount>,
    #[account(mut, address = derive_mempool_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: mempool_account
    pub mempool_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_execpool_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: executing_pool
    pub executing_pool: UncheckedAccount<'info>,
    #[account(mut, address = derive_comp_pda!(computation_offset, mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: computation_account
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_PRIVATE_TRANSFER))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(mut, address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(mut, address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS)]
    pub pool_account: Account<'info, FeePool>,
    #[account(mut, address = ARCIUM_CLOCK_ACCOUNT_ADDRESS)]
    pub clock_account: Account<'info, ClockAccount>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
}

#[callback_accounts("private_transfer")]
#[derive(Accounts)]
pub struct PrivateTransferCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_PRIVATE_TRANSFER))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Account<'info, MXEAccount>,
    /// CHECK: computation_account
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(address = ::anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: instructions_sysvar
    pub instructions_sysvar: AccountInfo<'info>,
}

#[init_computation_definition_accounts("private_transfer", payer)]
#[derive(Accounts)]
pub struct InitPrivateTransferCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)]
    /// CHECK: comp_def_account
    pub comp_def_account: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

// Check Balance Accounts
#[queue_computation_accounts("check_balance", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct CheckBalance<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init_if_needed,
        space = 9,
        payer = payer,
        seeds = [&SIGN_PDA_SEED],
        bump,
        address = derive_sign_pda!(),
    )]
    pub sign_pda_account: Account<'info, ArciumSignerAccount>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Account<'info, MXEAccount>,
    #[account(mut, address = derive_mempool_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: mempool_account
    pub mempool_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_execpool_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: executing_pool
    pub executing_pool: UncheckedAccount<'info>,
    #[account(mut, address = derive_comp_pda!(computation_offset, mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: computation_account
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_CHECK_BALANCE))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(mut, address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(mut, address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS)]
    pub pool_account: Account<'info, FeePool>,
    #[account(mut, address = ARCIUM_CLOCK_ACCOUNT_ADDRESS)]
    pub clock_account: Account<'info, ClockAccount>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
}

#[callback_accounts("check_balance")]
#[derive(Accounts)]
pub struct CheckBalanceCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_CHECK_BALANCE))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Account<'info, MXEAccount>,
    /// CHECK: computation_account
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(address = ::anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: instructions_sysvar
    pub instructions_sysvar: AccountInfo<'info>,
}

#[init_computation_definition_accounts("check_balance", payer)]
#[derive(Accounts)]
pub struct InitCheckBalanceCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)]
    /// CHECK: comp_def_account
    pub comp_def_account: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

// Validate Swap Accounts
#[queue_computation_accounts("validate_swap", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct ValidateSwap<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init_if_needed,
        space = 9,
        payer = payer,
        seeds = [&SIGN_PDA_SEED],
        bump,
        address = derive_sign_pda!(),
    )]
    pub sign_pda_account: Account<'info, ArciumSignerAccount>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Account<'info, MXEAccount>,
    #[account(mut, address = derive_mempool_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: mempool_account
    pub mempool_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_execpool_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: executing_pool
    pub executing_pool: UncheckedAccount<'info>,
    #[account(mut, address = derive_comp_pda!(computation_offset, mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: computation_account
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_VALIDATE_SWAP))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(mut, address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(mut, address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS)]
    pub pool_account: Account<'info, FeePool>,
    #[account(mut, address = ARCIUM_CLOCK_ACCOUNT_ADDRESS)]
    pub clock_account: Account<'info, ClockAccount>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
}

#[callback_accounts("validate_swap")]
#[derive(Accounts)]
pub struct ValidateSwapCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_VALIDATE_SWAP))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Account<'info, MXEAccount>,
    /// CHECK: computation_account
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(address = ::anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: instructions_sysvar
    pub instructions_sysvar: AccountInfo<'info>,
}

#[init_computation_definition_accounts("validate_swap", payer)]
#[derive(Accounts)]
pub struct InitValidateSwapCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)]
    /// CHECK: comp_def_account
    pub comp_def_account: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

// =============================================================================
// EVENTS
// =============================================================================

#[event]
pub struct PrivateTransferEvent {
    /// Encrypted boolean - was transfer valid?
    pub is_valid: [u8; 32],
    /// Encrypted u64 - new sender balance
    pub new_sender_balance: [u8; 32],
    /// Nonce for decryption
    pub nonce: [u8; 16],
}

#[event]
pub struct BalanceCheckEvent {
    /// Encrypted boolean - does balance meet minimum?
    pub meets_minimum: [u8; 32],
    /// Nonce for decryption
    pub nonce: [u8; 16],
}

#[event]
pub struct SwapValidationEvent {
    /// Encrypted boolean - is swap valid?
    pub is_valid: [u8; 32],
    /// Encrypted u64 - new input balance
    pub new_input_balance: [u8; 32],
    /// Encrypted boolean - is slippage acceptable?
    pub slippage_ok: [u8; 32],
    /// Nonce for decryption
    pub nonce: [u8; 16],
}

// =============================================================================
// ERRORS
// =============================================================================

#[error_code]
pub enum ErrorCode {
    #[msg("The computation was aborted")]
    AbortedComputation,
    #[msg("Cluster not set")]
    ClusterNotSet,
}
