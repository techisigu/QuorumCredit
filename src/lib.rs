#![no_std]

use soroban_sdk::{
    contract, contractimpl, panic_with_error, symbol_short, Address, BytesN, Env, Vec,
};

// Module declarations
pub mod admin;
pub mod errors;
pub mod helpers;
pub mod loan;
pub mod reputation;
pub mod types;
pub mod vouch;

// Re-exports for external use
pub use errors::ContractError;
pub use types::*;

use helpers::{config, require_valid_address, validate_admin_config};
use reputation::ReputationNftExternalClient;

#[contract]
pub struct QuorumCreditContract;

#[contractimpl]
impl QuorumCreditContract {
    /// One-time initialisation: set admins, XLM token address, and default config.
    pub fn initialize(
        env: Env,
        deployer: Address,
        admins: Vec<Address>,
        admin_threshold: u32,
        token: Address,
    ) -> Result<(), ContractError> {
        deployer.require_auth();

        if env.storage().instance().has(&DataKey::Config) {
            panic_with_error!(&env, ContractError::AlreadyInitialized);
        }

        // Validate admin addresses and configuration
        validate_admin_config(&env, &admins, admin_threshold)?;

        // Validate token address
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
            (deployer.clone(), admins, admin_threshold, token),
        );

        Ok(())
    }

    // ── Vouch Functions ───────────────────────────────────────────────────────

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

    // ── Loan Functions ────────────────────────────────────────────────────────

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

    // ── Admin Functions ───────────────────────────────────────────────────────

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

    // ── View Functions ────────────────────────────────────────────────────────

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

    pub fn total_vouched(env: Env, borrower: Address) -> i128 {
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
    use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};

    fn create_test_token(env: &Env) -> Address {
        Address::generate(env)
    }

    fn create_test_admin(env: &Env) -> Address {
        Address::generate(env)
    }

    fn create_zero_account_address(env: &Env) -> Address {
        Address::from_string(&String::from_str(
            env,
            "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
        ))
    }

    fn create_zero_contract_address(env: &Env) -> Address {
        Address::from_string(&String::from_str(
            env,
            "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4",
        ))
    }

    #[test]
    fn test_initialize_rejects_zero_admin_address() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, QuorumCreditContract);
        let client = QuorumCreditContractClient::new(&env, &contract_id);

        let deployer = Address::generate(&env);
        let zero_admin = create_zero_account_address(&env);
        let valid_admin = create_test_admin(&env);
        let admins = Vec::from_array(&env, [zero_admin, valid_admin]);
        let token = create_test_token(&env);

        let result = client.try_initialize(&deployer, &admins, &1, &token);
        assert!(result.is_err());

        // Verify the specific error
        match result.err().unwrap() {
            Ok(err) => assert_eq!(err, ContractError::ZeroAddress),
            Err(_) => panic!("Expected ContractError::ZeroAddress"),
        }
    }

    #[test]
    fn test_initialize_rejects_zero_contract_admin_address() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, QuorumCreditContract);
        let client = QuorumCreditContractClient::new(&env, &contract_id);

        let deployer = Address::generate(&env);
        let zero_admin = create_zero_contract_address(&env);
        let admins = Vec::from_array(&env, [zero_admin]);
        let token = create_test_token(&env);

        let result = client.try_initialize(&deployer, &admins, &1, &token);
        assert!(result.is_err());

        // Verify the specific error
        match result.err().unwrap() {
            Ok(err) => assert_eq!(err, ContractError::ZeroAddress),
            Err(_) => panic!("Expected ContractError::ZeroAddress"),
        }
    }

    #[test]
    fn test_initialize_rejects_zero_token_address() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, QuorumCreditContract);
        let client = QuorumCreditContractClient::new(&env, &contract_id);

        let deployer = Address::generate(&env);
        let admin = create_test_admin(&env);
        let admins = Vec::from_array(&env, [admin]);
        let zero_token = create_zero_account_address(&env);

        let result = client.try_initialize(&deployer, &admins, &1, &zero_token);
        assert!(result.is_err());

        // Verify the specific error
        match result.err().unwrap() {
            Ok(err) => assert_eq!(err, ContractError::ZeroAddress),
            Err(_) => panic!("Expected ContractError::ZeroAddress"),
        }
    }

    #[test]
    fn test_initialize_succeeds_with_valid_addresses() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, QuorumCreditContract);
        let client = QuorumCreditContractClient::new(&env, &contract_id);

        let deployer = Address::generate(&env);
        let admin = create_test_admin(&env);
        let admins = Vec::from_array(&env, [admin]);
        let token = create_test_token(&env);

        let result = client.try_initialize(&deployer, &admins, &1, &token);
        assert!(result.is_ok());

        // Verify contract is initialized
        assert!(client.is_initialized());
    }

    #[test]
    fn test_get_config_returns_defaults_after_initialization() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, QuorumCreditContract);
        let client = QuorumCreditContractClient::new(&env, &contract_id);

        let deployer = Address::generate(&env);
        let admin = create_test_admin(&env);
        let admins = Vec::from_array(&env, [admin]);
        let token = create_test_token(&env);

        client.initialize(&deployer, &admins, &1, &token);

        let config = client.get_config();
        assert_eq!(config.yield_bps, DEFAULT_YIELD_BPS);
        assert_eq!(config.slash_bps, DEFAULT_SLASH_BPS);
        assert_eq!(config.max_vouchers, DEFAULT_MAX_VOUCHERS);
        assert_eq!(config.min_loan_amount, DEFAULT_MIN_LOAN_AMOUNT);
        assert_eq!(config.loan_duration, DEFAULT_LOAN_DURATION);
        assert_eq!(
            config.max_loan_to_stake_ratio,
            DEFAULT_MAX_LOAN_TO_STAKE_RATIO
        );
    }

    #[test]
    fn test_update_config_yield_bps_only() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, QuorumCreditContract);
        let client = QuorumCreditContractClient::new(&env, &contract_id);

        let deployer = Address::generate(&env);
        let admin = create_test_admin(&env);
        let admins = Vec::from_array(&env, [admin.clone()]);
        let token = create_test_token(&env);

        client.initialize(&deployer, &admins, &1, &token);

        // Update only yield_bps
        let new_yield_bps = 300i128;
        client.update_config(&admins, &Some(new_yield_bps), &None);

        let config = client.get_config();
        assert_eq!(config.yield_bps, new_yield_bps);
        assert_eq!(config.slash_bps, DEFAULT_SLASH_BPS); // Should remain unchanged
    }

    #[test]
    fn test_update_config_slash_bps_only() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, QuorumCreditContract);
        let client = QuorumCreditContractClient::new(&env, &contract_id);

        let deployer = Address::generate(&env);
        let admin = create_test_admin(&env);
        let admins = Vec::from_array(&env, [admin.clone()]);
        let token = create_test_token(&env);

        client.initialize(&deployer, &admins, &1, &token);

        // Update only slash_bps
        let new_slash_bps = 6000i128;
        client.update_config(&admins, &None, &Some(new_slash_bps));

        let config = client.get_config();
        assert_eq!(config.yield_bps, DEFAULT_YIELD_BPS); // Should remain unchanged
        assert_eq!(config.slash_bps, new_slash_bps);
    }

    #[test]
    fn test_update_config_both_yield_and_slash_bps() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, QuorumCreditContract);
        let client = QuorumCreditContractClient::new(&env, &contract_id);

        let deployer = Address::generate(&env);
        let admin = create_test_admin(&env);
        let admins = Vec::from_array(&env, [admin.clone()]);
        let token = create_test_token(&env);

        client.initialize(&deployer, &admins, &1, &token);

        // Update both values
        let new_yield_bps = 400i128;
        let new_slash_bps = 7000i128;
        client.update_config(&admins, &Some(new_yield_bps), &Some(new_slash_bps));

        let config = client.get_config();
        assert_eq!(config.yield_bps, new_yield_bps);
        assert_eq!(config.slash_bps, new_slash_bps);
    }

    #[test]
    fn test_update_config_no_changes() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, QuorumCreditContract);
        let client = QuorumCreditContractClient::new(&env, &contract_id);

        let deployer = Address::generate(&env);
        let admin = create_test_admin(&env);
        let admins = Vec::from_array(&env, [admin.clone()]);
        let token = create_test_token(&env);

        client.initialize(&deployer, &admins, &1, &token);

        // Update with None values (no changes)
        client.update_config(&admins, &None, &None);

        let config = client.get_config();
        assert_eq!(config.yield_bps, DEFAULT_YIELD_BPS);
        assert_eq!(config.slash_bps, DEFAULT_SLASH_BPS);
    }

    #[test]
    #[should_panic(expected = "yield_bps must be non-negative")]
    fn test_update_config_rejects_negative_yield_bps() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, QuorumCreditContract);
        let client = QuorumCreditContractClient::new(&env, &contract_id);

        let deployer = Address::generate(&env);
        let admin = create_test_admin(&env);
        let admins = Vec::from_array(&env, [admin.clone()]);
        let token = create_test_token(&env);

        client.initialize(&deployer, &admins, &1, &token);

        // Try to set negative yield_bps
        client.update_config(&admins, &Some(-100i128), &None);
    }

    #[test]
    #[should_panic(expected = "slash_bps must be 1-10000")]
    fn test_update_config_rejects_zero_slash_bps() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, QuorumCreditContract);
        let client = QuorumCreditContractClient::new(&env, &contract_id);

        let deployer = Address::generate(&env);
        let admin = create_test_admin(&env);
        let admins = Vec::from_array(&env, [admin.clone()]);
        let token = create_test_token(&env);

        client.initialize(&deployer, &admins, &1, &token);

        // Try to set zero slash_bps
        client.update_config(&admins, &None, &Some(0i128));
    }

    #[test]
    #[should_panic(expected = "slash_bps must be 1-10000")]
    fn test_update_config_rejects_slash_bps_above_10000() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, QuorumCreditContract);
        let client = QuorumCreditContractClient::new(&env, &contract_id);

        let deployer = Address::generate(&env);
        let admin = create_test_admin(&env);
        let admins = Vec::from_array(&env, [admin.clone()]);
        let token = create_test_token(&env);

        client.initialize(&deployer, &admins, &1, &token);

        // Try to set slash_bps above 10000
        client.update_config(&admins, &None, &Some(10001i128));
    }

    #[test]
    fn test_update_config_accepts_boundary_values() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, QuorumCreditContract);
        let client = QuorumCreditContractClient::new(&env, &contract_id);

        let deployer = Address::generate(&env);
        let admin = create_test_admin(&env);
        let admins = Vec::from_array(&env, [admin.clone()]);
        let token = create_test_token(&env);

        client.initialize(&deployer, &admins, &1, &token);

        // Test boundary values
        client.update_config(&admins, &Some(0i128), &Some(1i128)); // Min values
        let config = client.get_config();
        assert_eq!(config.yield_bps, 0);
        assert_eq!(config.slash_bps, 1);

        client.update_config(&admins, &None, &Some(10000i128)); // Max slash_bps
        let config = client.get_config();
        assert_eq!(config.slash_bps, 10000);
    }
}
