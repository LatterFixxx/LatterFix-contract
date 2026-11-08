#![cfg(test)]

use crate::{TaskManagerContract, TaskManagerContractClient};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env, String, Vec};

// ── Shared setup helper ────────────────────────────────────────────────────

fn setup_initialized_contract(env: &Env, fee_bps: u32) -> (
    TaskManagerContractClient,
    Address, // contract_id
    Address, // admin
    Address, // token_contract
    Address, // fee_recipient
) {
    let contract_id = env.register_contract(None, TaskManagerContract);
    let client = TaskManagerContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token_contract = env.register_stellar_asset_contract(token_admin);
    let fee_recipient = Address::generate(env);

    client.initialize(&admin, &fee_bps, &token_contract, &fee_recipient);
    (client, contract_id, admin, token_contract, fee_recipient)
}

// ── Test 1: Contract initialization ────────────────────────────────────────

#[test]
fn test_initialization() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _, admin, token_contract, fee_recipient) =
        setup_initialized_contract(&env, 100);

    // Double-initialization must fail
    let res = client.try_initialize(&admin, &100, &token_contract, &fee_recipient);
    assert!(res.is_err(), "re-initialization should be rejected");
}

// ── Test 2: Full task lifecycle with fee assertion ─────────────────────────

#[test]
fn test_create_and_complete_task_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, contract_id, _, token_contract, fee_recipient) =
        setup_initialized_contract(&env, 250); // 2.5% fee

    let creator = Address::generate(&env);
    let assignee = Address::generate(&env);

    StellarAssetClient::new(&env, &token_contract).mint(&creator, &1000);
    let token = soroban_sdk::token::Client::new(&env, &token_contract);
    assert_eq!(token.balance(&creator), 1000);

    let mut tags = Vec::new(&env);
    tags.push_back(String::from_str(&env, "rust"));

    let task_id = client.create_task(
        &creator,
        &String::from_str(&env, "Test Task"),
        &String::from_str(&env, "Task Description"),
        &1000,
        &tags,
    );
    assert_eq!(task_id, 1);
    assert_eq!(token.balance(&creator), 0);
    assert_eq!(token.balance(&contract_id), 1000);

    client.assign_task(&assignee, &task_id);
    client.submit_work(
        &assignee,
        &task_id,
        &String::from_str(&env, "https://github.com/LatterFixxx/LatterFix-Smart-contract"),
    );
    client.complete_task(&creator, &task_id);

    // 2.5% of 1000 = 25 fee; assignee receives 975
    assert_eq!(token.balance(&fee_recipient), 25);
    assert_eq!(token.balance(&assignee), 975);
    assert_eq!(token.balance(&contract_id), 0);
}

// ── Test 3: Cancel refunds full amount ────────────────────────────────────

#[test]
fn test_cancel_task_refund() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, contract_id, _, token_contract, _) =
        setup_initialized_contract(&env, 100);

    let creator = Address::generate(&env);
    StellarAssetClient::new(&env, &token_contract).mint(&creator, &500);

    let task_id = client.create_task(
        &creator,
        &String::from_str(&env, "Cancel Task"),
        &String::from_str(&env, "Will cancel this"),
        &500,
        &Vec::new(&env),
    );

    let token = soroban_sdk::token::Client::new(&env, &token_contract);
    assert_eq!(token.balance(&creator), 0);
    assert_eq!(token.balance(&contract_id), 500);

    client.cancel_task(&creator, &task_id);
    assert_eq!(token.balance(&creator), 500, "creator must be fully refunded");
    assert_eq!(token.balance(&contract_id), 0);
}

// ── Test 4: Dispute and 50/50 split resolution ────────────────────────────

#[test]
fn test_dispute_and_resolution() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, contract_id, admin, token_contract, _) =
        setup_initialized_contract(&env, 100);

    let creator = Address::generate(&env);
    let assignee = Address::generate(&env);
    StellarAssetClient::new(&env, &token_contract).mint(&creator, &1000);

    let task_id = client.create_task(
        &creator,
        &String::from_str(&env, "Dispute Task"),
        &String::from_str(&env, "Dispute test"),
        &1000,
        &Vec::new(&env),
    );
    client.assign_task(&assignee, &task_id);
    client.dispute_task(&creator, &task_id);
    client.resolve_dispute(&admin, &task_id, &500, &500);

    let token = soroban_sdk::token::Client::new(&env, &token_contract);
    assert_eq!(token.balance(&creator), 500, "creator gets 50%");
    assert_eq!(token.balance(&assignee), 500, "assignee gets 50%");
    assert_eq!(token.balance(&contract_id), 0);
}

// ── Test 5: User profile lifecycle ────────────────────────────────────────

#[test]
fn test_user_profile_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _, admin, _, _) = setup_initialized_contract(&env, 100);

    let user = Address::generate(&env);
    let username = String::from_str(&env, "john_doe");
    let bio = String::from_str(&env, "Rust developer");

    client.create_profile(&user, &username, &bio);

    let profile = client.get_profile(&user).expect("profile must exist after creation");
    assert_eq!(profile.address, user);
    assert_eq!(profile.username, username);
    assert_eq!(profile.reputation, 100, "starting reputation is 100");
    assert_eq!(profile.completed_tasks, 0);
    assert_eq!(profile.bio, bio);

    let new_bio = String::from_str(&env, "Soroban developer");
    client.update_bio(&user, &new_bio);
    assert_eq!(client.get_profile(&user).unwrap().bio, new_bio);

    client.reward_contribution(&admin, &user, &25);
    let updated = client.get_profile(&user).unwrap();
    assert_eq!(updated.reputation, 125, "reputation must increase by 25");
    assert_eq!(updated.completed_tasks, 1);
}

// ── Test 6: Dispute resolved fully in favour of assignee ─────────────────

#[test]
fn test_dispute_full_assignee_payout() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, contract_id, admin, token_contract, _) =
        setup_initialized_contract(&env, 0); // 0% fee for clean assertions

    let creator = Address::generate(&env);
    let assignee = Address::generate(&env);
    StellarAssetClient::new(&env, &token_contract).mint(&creator, &800);

    let task_id = client.create_task(
        &creator,
        &String::from_str(&env, "Full Assignee Payout"),
        &String::from_str(&env, "Admin rules in contributor favour"),
        &800,
        &Vec::new(&env),
    );
    client.assign_task(&assignee, &task_id);
    client.dispute_task(&creator, &task_id);
    // Admin awards 100% to assignee
    client.resolve_dispute(&admin, &task_id, &0, &800);

    let token = soroban_sdk::token::Client::new(&env, &token_contract);
    assert_eq!(token.balance(&assignee), 800, "assignee gets full payout");
    assert_eq!(token.balance(&creator), 0);
    assert_eq!(token.balance(&contract_id), 0);
}

// ── Test 7: Multiple tasks share the same contract instance ───────────────

#[test]
fn test_multiple_concurrent_tasks() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, contract_id, _, token_contract, fee_recipient) =
        setup_initialized_contract(&env, 100); // 1% fee

    let creator = Address::generate(&env);
    let a1 = Address::generate(&env);
    let a2 = Address::generate(&env);

    StellarAssetClient::new(&env, &token_contract).mint(&creator, &3000);

    let mut tags = Vec::new(&env);
    tags.push_back(String::from_str(&env, "frontend"));

    let t1 = client.create_task(
        &creator,
        &String::from_str(&env, "Task Alpha"),
        &String::from_str(&env, "First concurrent task"),
        &1000,
        &tags,
    );
    let t2 = client.create_task(
        &creator,
        &String::from_str(&env, "Task Beta"),
        &String::from_str(&env, "Second concurrent task"),
        &2000,
        &tags,
    );

    let token = soroban_sdk::token::Client::new(&env, &token_contract);
    assert_eq!(token.balance(&contract_id), 3000, "both task rewards locked");
    assert_eq!(token.balance(&creator), 0);

    // Complete task 1 → a1
    client.assign_task(&a1, &t1);
    client.submit_work(&a1, &t1, &String::from_str(&env, "https://github.com/pr/1"));
    client.complete_task(&creator, &t1);

    // Complete task 2 → a2
    client.assign_task(&a2, &t2);
    client.submit_work(&a2, &t2, &String::from_str(&env, "https://github.com/pr/2"));
    client.complete_task(&creator, &t2);

    // 1% of 1000 = 10, 1% of 2000 = 20 → fee_recipient gets 30
    assert_eq!(token.balance(&fee_recipient), 30, "combined fees correct");
    assert_eq!(token.balance(&a1), 990);
    assert_eq!(token.balance(&a2), 1980);
    assert_eq!(token.balance(&contract_id), 0);
}

// ── Test 8: Cannot assign to an already-assigned task ────────────────────

#[test]
fn test_cannot_double_assign() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _, _, token_contract, _) = setup_initialized_contract(&env, 100);

    let creator = Address::generate(&env);
    let a1 = Address::generate(&env);
    let a2 = Address::generate(&env);

    StellarAssetClient::new(&env, &token_contract).mint(&creator, &500);

    let task_id = client.create_task(
        &creator,
        &String::from_str(&env, "Single Assign Task"),
        &String::from_str(&env, "Only one assignee allowed"),
        &500,
        &Vec::new(&env),
    );

    client.assign_task(&a1, &task_id);
    // Second assignment to same task must fail
    let result = client.try_assign_task(&a2, &task_id);
    assert!(result.is_err(), "double-assigning a task should be rejected");
}

// ── Test 9: State Archival TTL Extension (Issue #25) ────────────────────────

#[test]
fn test_state_archival_ttl_extension() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _, _, _, _) = setup_initialized_contract(&env, 100);

    // Call instance TTL bump and archival refresh
    client.bump_contract_instance_ttl();
    client.refresh_archival_ttl();
}

// ── Test 10: Administrative Asset Clawback (Issue #13) ──────────────────────

#[test]
fn test_clawback_asset_recovery() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _, admin, _, _) = setup_initialized_contract(&env, 100);
    let target_user = Address::generate(&env);

    // Verify clawback initially disabled
    assert!(!client.is_account_clawback_enabled(&target_user));

    // Admin enables clawback for target_user
    client.set_account_clawback_enabled(&admin, &target_user, &true);
    assert!(client.is_account_clawback_enabled(&target_user));

    // Admin executes clawback for 400 units
    client.clawback_asset(&admin, &target_user, &400);

    // Verify clawback audit record
    let history = client.get_clawback_history(&target_user);
    assert_eq!(history.len(), 1);
    let record = history.get(0).unwrap();
    assert_eq!(record.admin, admin);
    assert_eq!(record.from, target_user);
    assert_eq!(record.amount, 400);

    // Non-admin clawback must fail
    let non_admin = Address::generate(&env);
    let result = client.try_clawback_asset(&non_admin, &target_user, &100);
    assert!(result.is_err(), "non-admin clawback must be rejected");

    // Admin disables clawback for target_user
    client.set_account_clawback_enabled(&admin, &target_user, &false);
    assert!(!client.is_account_clawback_enabled(&target_user));

    // Clawback execution on disabled account must fail
    let disabled_result = client.try_clawback_asset(&admin, &target_user, &50);
    assert!(disabled_result.is_err(), "clawback on disabled account must fail");
}

// ── Test 11: Anchor SEP-24 Fiat Withdrawal Session (Issue #6) ───────────────

#[test]
fn test_sep24_withdrawal_session_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _, _, _, _) = setup_initialized_contract(&env, 100);
    let user = Address::generate(&env);

    let anchor_id = String::from_str(&env, "ANCHOR_USDC_OFFRAMP");
    let redirect_url = String::from_str(&env, "https://anchor.stellar.org/sep24/interactive/flow123");

    // Initially no session exists
    assert!(client.get_sep24_withdrawal_session(&user).is_none());

    // Initialize session
    client.init_sep24_withdrawal(&user, &anchor_id, &redirect_url);

    // Verify session retrieval
    let session = client.get_sep24_withdrawal_session(&user).unwrap();
    assert_eq!(session.account, user);
    assert_eq!(session.external_anchor_id, anchor_id);
    assert_eq!(session.interactive_url, redirect_url);
    assert_eq!(session.status, String::from_str(&env, "pending_interactive"));
}
