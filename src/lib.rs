#![no_std]

use soroban_sdk::{contract, contractimpl, symbol_short, Address, BytesN, Env, Vec};

pub mod admin;
pub mod errors;
pub mod helpers;
pub mod loan;
pub mod reputation;
pub mod types;
pub mod vouch;

pub use errors::ContractError;
pub use types::*;

use helpers::{config, require_valid_address, validate_admin_config};
use reputation::ReputationNftExternalClient;

#[contract]
pub struct QuorumCreditContract;

#[contractimpl]
impl QuorumCreditContract {
    pub fn initialize(
        env: Env,
        deployer: Address,
        admins: Vec<Address>,
        admin_threshold: u32,
        token: Address,
    ) -> Result<(), ContractError> {
        deployer.require_auth();

        if env.storage().instance().has(&DataKey::Config) {
            return Err(ContractError::AlreadyInitialized);
        }

        validate_admin_config(&env, &admins, admin_threshold)?;
        require_valid_address(&env, &token)?;

        env.storage().instance().set(&DataKey::Deployer, &deployer);
        env.storage().instance().set(
            &DataKey::Config,
            &Config {
                admins: admins.clone(),
                admin_threshold,
                token: token.clone(),
                yield_bps: DEFAULT_YIELD_BPS,
                slash_bps: DEFAULT_SLASH_BPS,
                max_vouchers: DEFAULT_MAX_VOUCHERS,
                min_loan_amount: DEFAULT_MIN_LOAN_AMOUNT,
                loan_duration: DEFAULT_LOAN_DURATION,
                max_loan_to_stake_ratio: DEFAULT_MAX_LOAN_TO_STAKE_RATIO,
                grace_period: 0,
            },
        );

        env.events().publish(
            (symbol_short!("contract"), symbol_short!("init")),
            (deployer, admins, admin_threshold, token),
        );

        Ok(())
    }

    pub fn vouch(
        env: Env,
        voucher: Address,
        borrower: Address,
        stake: i128,
    ) -> Result<(), ContractError> {
        vouch::vouch(env, voucher, borrower, stake)
    }

    pub fn batch_vouch(
        env: Env,
        voucher: Address,
        borrowers: Vec<Address>,
        stakes: Vec<i128>,
    ) -> Result<(), ContractError> {
        vouch::batch_vouch(env, voucher, borrowers, stakes)
    }

    pub fn increase_stake(
        env: Env,
        voucher: Address,
        borrower: Address,
        additional: i128,
    ) -> Result<(), ContractError> {
        vouch::increase_stake(env, voucher, borrower, additional)
    }

    pub fn decrease_stake(
        env: Env,
        voucher: Address,
        borrower: Address,
        amount: i128,
    ) -> Result<(), ContractError> {
        vouch::decrease_stake(env, voucher, borrower, amount)
    }

    pub fn withdraw_vouch(
        env: Env,
        voucher: Address,
        borrower: Address,
    ) -> Result<(), ContractError> {
        vouch::withdraw_vouch(env, voucher, borrower)
    }

    pub fn transfer_vouch(
        env: Env,
        from: Address,
        to: Address,
        borrower: Address,
    ) -> Result<(), ContractError> {
        vouch::transfer_vouch(env, from, to, borrower)
    }

    pub fn request_loan(
        env: Env,
        borrower: Address,
        amount: i128,
        threshold: i128,
    ) -> Result<(), ContractError> {
        loan::request_loan(env, borrower, amount, threshold)
    }

    pub fn repay(env: Env, borrower: Address, payment: i128) -> Result<(), ContractError> {
        loan::repay(env, borrower, payment)
    }

    pub fn add_admin(env: Env, admin_signers: Vec<Address>, new_admin: Address) {
        admin::add_admin(env, admin_signers, new_admin)
    }

    pub fn remove_admin(env: Env, admin_signers: Vec<Address>, admin_to_remove: Address) {
        admin::remove_admin(env, admin_signers, admin_to_remove)
    }

    pub fn rotate_admin(
        env: Env,
        admin_signers: Vec<Address>,
        old_admin: Address,
        new_admin: Address,
    ) {
        admin::rotate_admin(env, admin_signers, old_admin, new_admin)
    }

    pub fn set_admin_threshold(env: Env, admin_signers: Vec<Address>, new_threshold: u32) {
        admin::set_admin_threshold(env, admin_signers, new_threshold)
    }

    pub fn set_protocol_fee(env: Env, admin_signers: Vec<Address>, fee_bps: u32) {
        admin::set_protocol_fee(env, admin_signers, fee_bps)
    }

    pub fn whitelist_voucher(env: Env, admin_signers: Vec<Address>, voucher: Address) {
        admin::whitelist_voucher(env, admin_signers, voucher)
    }

    pub fn set_fee_treasury(env: Env, admin_signers: Vec<Address>, treasury: Address) {
        admin::set_fee_treasury(env, admin_signers, treasury)
    }

    pub fn upgrade(env: Env, admin_signers: Vec<Address>, new_wasm_hash: BytesN<32>) {
        admin::upgrade(env, admin_signers, new_wasm_hash)
    }

    pub fn pause(env: Env, admin_signers: Vec<Address>) {
        admin::pause(env, admin_signers)
    }

    pub fn unpause(env: Env, admin_signers: Vec<Address>) {
        admin::unpause(env, admin_signers)
    }

    pub fn blacklist(env: Env, admin_signers: Vec<Address>, borrower: Address) {
        admin::blacklist(env, admin_signers, borrower)
    }

    pub fn set_config(env: Env, admin_signers: Vec<Address>, config: Config) {
        admin::set_config(env, admin_signers, config)
    }

    pub fn update_config(
        env: Env,
        admin_signers: Vec<Address>,
        yield_bps: Option<i128>,
        slash_bps: Option<i128>,
    ) {
        admin::update_config(env, admin_signers, yield_bps, slash_bps)
    }

    pub fn set_reputation_nft(env: Env, admin_signers: Vec<Address>, nft_contract: Address) {
        admin::set_reputation_nft(env, admin_signers, nft_contract)
    }

    pub fn set_min_stake(env: Env, admin_signers: Vec<Address>, amount: i128) {
        admin::set_min_stake(env, admin_signers, amount)
    }

    pub fn set_max_loan_amount(env: Env, admin_signers: Vec<Address>, amount: i128) {
        admin::set_max_loan_amount(env, admin_signers, amount)
    }

    pub fn set_min_vouchers(env: Env, admin_signers: Vec<Address>, count: u32) {
        admin::set_min_vouchers(env, admin_signers, count)
    }

    pub fn set_max_loan_to_stake_ratio(env: Env, admin_signers: Vec<Address>, ratio: u32) {
        admin::set_max_loan_to_stake_ratio(env, admin_signers, ratio)
    }

    pub fn is_initialized(env: Env) -> bool {
        env.storage().instance().has(&DataKey::Config)
    }

    pub fn get_token(env: Env) -> Address {
        config(&env).token
    }

    pub fn get_admins(env: Env) -> Vec<Address> {
        admin::get_admins(env)
    }

    pub fn get_admin_threshold(env: Env) -> u32 {
        admin::get_admin_threshold(env)
    }

    pub fn get_slash_treasury_balance(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::SlashTreasury)
            .unwrap_or(0)
    }

    pub fn get_paused(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false)
    }

    pub fn loan_status(env: Env, borrower: Address) -> LoanStatus {
        loan::loan_status(env, borrower)
    }

    pub fn vouch_exists(env: Env, voucher: Address, borrower: Address) -> bool {
        vouch::vouch_exists(env, voucher, borrower)
    }

    pub fn is_whitelisted(env: Env, voucher: Address) -> bool {
        admin::is_whitelisted(env, voucher)
    }

    pub fn get_loan(env: Env, borrower: Address) -> Option<LoanRecord> {
        loan::get_loan(env, borrower)
    }

    pub fn get_loan_by_id(env: Env, loan_id: u64) -> Option<LoanRecord> {
        loan::get_loan_by_id(env, loan_id)
    }

    pub fn get_vouches(env: Env, borrower: Address) -> Option<Vec<VouchRecord>> {
        env.storage().persistent().get(&DataKey::Vouches(borrower))
    }

    pub fn is_eligible(env: Env, borrower: Address, threshold: i128) -> bool {
        loan::is_eligible(env, borrower, threshold)
    }

    pub fn get_contract_balance(env: Env) -> i128 {
        helpers::token(&env).balance(&env.current_contract_address())
    }

    pub fn voucher_history(env: Env, voucher: Address) -> Vec<Address> {
        vouch::voucher_history(env, voucher)
    }

    pub fn get_reputation(env: Env, borrower: Address) -> u32 {
        let nft_addr: Address = match env
            .storage()
            .instance()
            .get::<DataKey, Address>(&DataKey::ReputationNft)
        {
            Some(a) => a,
            None => return 0,
        };
        ReputationNftExternalClient::new(&env, &nft_addr).balance(&borrower)
    }

    pub fn total_vouched(env: Env, borrower: Address) -> Result<i128, ContractError> {
        vouch::total_vouched(env, borrower)
    }

    pub fn repayment_count(env: Env, borrower: Address) -> u32 {
        loan::repayment_count(env, borrower)
    }

    pub fn loan_count(env: Env, borrower: Address) -> u32 {
        loan::loan_count(env, borrower)
    }

    pub fn default_count(env: Env, borrower: Address) -> u32 {
        loan::default_count(env, borrower)
    }

    pub fn get_protocol_fee(env: Env) -> u32 {
        admin::get_protocol_fee(env)
    }

    pub fn get_fee_treasury(env: Env) -> Option<Address> {
        admin::get_fee_treasury(env)
    }

    pub fn is_blacklisted(env: Env, borrower: Address) -> bool {
        admin::is_blacklisted(env, borrower)
    }

    pub fn get_min_stake(env: Env) -> i128 {
        admin::get_min_stake(env)
    }

    pub fn get_max_loan_amount(env: Env) -> i128 {
        admin::get_max_loan_amount(env)
    }

    pub fn get_min_vouchers(env: Env) -> u32 {
        admin::get_min_vouchers(env)
    }

    pub fn get_max_loan_to_stake_ratio(env: Env) -> u32 {
        admin::get_max_loan_to_stake_ratio(env)
    }

    pub fn get_config(env: Env) -> Config {
        admin::get_config(env)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{
        testutils::{Address as _, Ledger as _},
        token::StellarAssetClient,
        Address, Env, Vec,
    };

    fn address_vec(env: &Env, addrs: &[Address]) -> Vec<Address> {
        let mut result = Vec::new(env);
        for addr in addrs {
            result.push_back(addr.clone());
        }
        result
    }

    fn setup(env: &Env) -> (Address, Address, Address, Address) {
        env.mock_all_auths();
        env.ledger().set_timestamp(1_000_000);

        let admin = Address::generate(env);
        let borrower = Address::generate(env);
        let voucher = Address::generate(env);
        let admins = address_vec(env, &[admin.clone()]);

        let token_id = env.register_stellar_asset_contract_v2(admin.clone());
        let token_admin = StellarAssetClient::new(env, &token_id.address());
        token_admin.mint(&voucher, &10_000_000);
        token_admin.mint(&borrower, &20_000);

        let contract_id = env.register_contract(None, QuorumCreditContract);
        token_admin.mint(&contract_id, &50_000_000);

        QuorumCreditContractClient::new(env, &contract_id).initialize(
            &admin,
            &admins,
            &1,
            &token_id.address(),
        );

        (contract_id, admin, borrower, voucher)
    }

    #[test]
    fn test_repay_sets_repayment_timestamp_on_full_repayment() {
        let env = Env::default();
        let (contract_id, _admin, borrower, voucher) = setup(&env);
        let client = QuorumCreditContractClient::new(&env, &contract_id);

        client.vouch(&voucher, &borrower, &1_000_000);
        env.ledger().set_timestamp(1_000_000 + MIN_VOUCH_AGE);
        client.request_loan(&borrower, &500_000, &1_000_000);

        let loan_before_repayment = client.get_loan(&borrower).unwrap();
        assert_eq!(loan_before_repayment.repayment_timestamp, None);

        let total_owed = loan_before_repayment.amount + loan_before_repayment.total_yield;
        let token_admin = StellarAssetClient::new(&env, &client.get_token());
        token_admin.mint(&borrower, &(total_owed - loan_before_repayment.amount));

        let partial_payment = total_owed - 1;
        client.repay(&borrower, &partial_payment);

        let loan_after_partial = client.get_loan(&borrower).unwrap();
        assert!(!loan_after_partial.repaid);
        assert_eq!(loan_after_partial.repayment_timestamp, None);

        let repayment_timestamp = 1_000_000 + MIN_VOUCH_AGE + 30;
        env.ledger().set_timestamp(repayment_timestamp);
        client.repay(&borrower, &1);

        let repaid_loan = client.get_loan(&borrower).unwrap();
        assert!(repaid_loan.repaid);
        assert_eq!(repaid_loan.repayment_timestamp, Some(repayment_timestamp));
    }
}
