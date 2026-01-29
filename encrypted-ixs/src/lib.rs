use arcis::*;

/// SIP Private Transfer Circuit
///
/// Enables confidential token transfers where:
/// - Sender's balance is encrypted
/// - Transfer amount is encrypted
/// - Recipient address is encrypted
/// - Balance validation happens in MPC (no one sees actual values)
///
/// This is the core of SIP's Arcium integration for the $10K bounty.
///
/// @see https://github.com/sip-protocol/sip-mobile/issues/73
#[encrypted]
mod circuits {
    use arcis::*;

    /// Input for private transfer validation
    /// All values are encrypted - MXE nodes compute without seeing plaintext
    pub struct PrivateTransferInput {
        /// Sender's current balance (in smallest units, e.g., lamports)
        sender_balance: u64,
        /// Amount to transfer
        amount: u64,
        /// Minimum balance to maintain (for rent exemption)
        min_balance: u64,
    }

    /// Output of private transfer
    pub struct PrivateTransferOutput {
        /// Whether the transfer is valid (sufficient balance)
        is_valid: bool,
        /// New sender balance after transfer
        new_sender_balance: u64,
    }

    /// Validate and compute a private transfer
    ///
    /// MPC guarantees:
    /// - No single node sees the actual balance or amount
    /// - Computation is verifiable via threshold signatures
    /// - Result is encrypted with requester's key
    #[instruction]
    pub fn private_transfer(
        input_ctxt: Enc<Shared, PrivateTransferInput>,
    ) -> Enc<Shared, PrivateTransferOutput> {
        let input = input_ctxt.to_arcis();

        // Check if sender has sufficient balance (encrypted comparison)
        // Both branches execute - MPC selects result without leaking which
        let available = input.sender_balance - input.min_balance;
        let is_valid = available >= input.amount;

        // Compute new balance (only meaningful if valid)
        let new_balance = if is_valid {
            input.sender_balance - input.amount
        } else {
            input.sender_balance // No change if invalid
        };

        let output = PrivateTransferOutput {
            is_valid,
            new_sender_balance: new_balance,
        };

        input_ctxt.owner.from_arcis(output)
    }

    /// Input for encrypted balance check (simpler use case)
    pub struct BalanceCheckInput {
        /// Balance to check
        balance: u64,
        /// Minimum required
        minimum: u64,
    }

    /// Check if encrypted balance meets minimum threshold
    /// Useful for pre-validation before transfer
    #[instruction]
    pub fn check_balance(input_ctxt: Enc<Shared, BalanceCheckInput>) -> Enc<Shared, bool> {
        let input = input_ctxt.to_arcis();
        let result = input.balance >= input.minimum;
        input_ctxt.owner.from_arcis(result)
    }

    /// Input for confidential swap validation
    pub struct ConfidentialSwapInput {
        /// Input token balance
        input_balance: u64,
        /// Input amount for swap
        input_amount: u64,
        /// Expected minimum output (slippage protection)
        min_output: u64,
        /// Actual output from DEX quote
        actual_output: u64,
    }

    /// Output of swap validation
    pub struct ConfidentialSwapOutput {
        /// Whether swap is valid
        is_valid: bool,
        /// New input balance
        new_input_balance: u64,
        /// Whether slippage is acceptable
        slippage_ok: bool,
    }

    /// Validate a confidential DEX swap
    ///
    /// Checks:
    /// 1. Sufficient input balance
    /// 2. Slippage within tolerance
    #[instruction]
    pub fn validate_swap(
        input_ctxt: Enc<Shared, ConfidentialSwapInput>,
    ) -> Enc<Shared, ConfidentialSwapOutput> {
        let input = input_ctxt.to_arcis();

        // Check sufficient balance
        let has_balance = input.input_balance >= input.input_amount;

        // Check slippage
        let slippage_ok = input.actual_output >= input.min_output;

        // Both must be true for valid swap
        let is_valid = has_balance && slippage_ok;

        let new_balance = if has_balance {
            input.input_balance - input.input_amount
        } else {
            input.input_balance
        };

        let output = ConfidentialSwapOutput {
            is_valid,
            new_input_balance: new_balance,
            slippage_ok,
        };

        input_ctxt.owner.from_arcis(output)
    }
}
