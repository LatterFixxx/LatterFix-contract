use soroban_sdk::{Address, Env, String, symbol_short};

/// Event module for the LatterFix TaskManager Soroban contract.
///
/// Every public state-changing action emits a structured event via
/// `env.events().publish()`. Events are indexed off-chain by:
///   - Stellar Expert contract event viewer
///   - Soroban RPC `getEvents` (filtered by contractId + topic)
///   - The LatterFix frontend via `fetchContractEvents()` in transactionHistory.ts
///
/// Topic layout:  (symbol, primary_id)
/// Data  layout:  tuple of relevant fields

// ── Task Events ────────────────────────────────────────────────────────────

pub fn emit_task_created(
    env: &Env,
    task_id: u32,
    creator: Address,
    title: String,
    reward: i128,
) {
    let ledger_ts = env.ledger().timestamp();
    env.events().publish(
        (symbol_short!("task_cre"), task_id),
        (creator, title, reward, ledger_ts),
    );
}

pub fn emit_task_assigned(
    env: &Env,
    task_id: u32,
    assignee: Address,
) {
    env.events().publish(
        (symbol_short!("task_assg"), task_id),
        (assignee, env.ledger().timestamp()),
    );
}

pub fn emit_task_submitted(
    env: &Env,
    task_id: u32,
    assignee: Address,
    delivery_url: String,
) {
    env.events().publish(
        (symbol_short!("task_subm"), task_id),
        (assignee, delivery_url, env.ledger().timestamp()),
    );
}

pub fn emit_task_completed(
    env: &Env,
    task_id: u32,
    assignee: Address,
    payout: i128,
    fee: i128,
) {
    env.events().publish(
        (symbol_short!("task_comp"), task_id),
        (assignee, payout, fee, env.ledger().timestamp()),
    );
}

pub fn emit_task_cancelled(
    env: &Env,
    task_id: u32,
    creator: Address,
    refund: i128,
) {
    env.events().publish(
        (symbol_short!("task_canc"), task_id),
        (creator, refund, env.ledger().timestamp()),
    );
}

pub fn emit_task_disputed(
    env: &Env,
    task_id: u32,
    caller: Address,
) {
    env.events().publish(
        (symbol_short!("task_disp"), task_id),
        (caller, env.ledger().timestamp()),
    );
}

pub fn emit_dispute_resolved(
    env: &Env,
    task_id: u32,
    creator_refund: i128,
    assignee_payout: i128,
) {
    env.events().publish(
        (symbol_short!("disp_resl"), task_id),
        (creator_refund, assignee_payout, env.ledger().timestamp()),
    );
}

// ── Profile Events ─────────────────────────────────────────────────────────

pub fn emit_profile_created(
    env: &Env,
    user: Address,
    username: String,
) {
    env.events().publish(
        (symbol_short!("prof_cre"), user),
        (username, env.ledger().timestamp()),
    );
}

pub fn emit_profile_updated(
    env: &Env,
    user: Address,
    field: String,
) {
    env.events().publish(
        (symbol_short!("prof_upd"), user),
        (field, env.ledger().timestamp()),
    );
}

pub fn emit_reputation_awarded(
    env: &Env,
    user: Address,
    points: u32,
    new_total: u32,
) {
    env.events().publish(
        (symbol_short!("rep_award"), user),
        (points, new_total, env.ledger().timestamp()),
    );
}

// ── Milestone Events ───────────────────────────────────────────────────────

pub fn emit_milestone_created(
    env: &Env,
    task_id: u32,
    milestone_id: u32,
    amount: i128,
) {
    env.events().publish(
        (symbol_short!("mile_cre"), (task_id, milestone_id)),
        (amount, env.ledger().timestamp()),
    );
}

pub fn emit_milestone_submitted(
    env: &Env,
    task_id: u32,
    milestone_id: u32,
    assignee: Address,
) {
    env.events().publish(
        (symbol_short!("mile_subm"), (task_id, milestone_id)),
        (assignee, env.ledger().timestamp()),
    );
}

pub fn emit_milestone_approved(
    env: &Env,
    task_id: u32,
    milestone_id: u32,
    amount: i128,
) {
    env.events().publish(
        (symbol_short!("mile_appr"), (task_id, milestone_id)),
        (amount, env.ledger().timestamp()),
    );
}

pub fn emit_milestone_rejected(
    env: &Env,
    task_id: u32,
    milestone_id: u32,
    feedback: String,
) {
    env.events().publish(
        (symbol_short!("mile_rej"), (task_id, milestone_id)),
        (feedback, env.ledger().timestamp()),
    );
}

// ── Governance Events ──────────────────────────────────────────────────────

pub fn emit_proposal_created(
    env: &Env,
    proposal_id: u32,
    proposer: Address,
    title: String,
) {
    env.events().publish(
        (symbol_short!("prop_cre"), proposal_id),
        (proposer, title, env.ledger().timestamp()),
    );
}

pub fn emit_vote_cast(
    env: &Env,
    proposal_id: u32,
    voter: Address,
    vote_type: String,
    weight: u32,
) {
    env.events().publish(
        (symbol_short!("vote_cast"), (proposal_id, voter)),
        (vote_type, weight, env.ledger().timestamp()),
    );
}

pub fn emit_proposal_executed(
    env: &Env,
    proposal_id: u32,
    passed: bool,
) {
    env.events().publish(
        (symbol_short!("prop_exec"), proposal_id),
        (passed, env.ledger().timestamp()),
    );
}

// ── Access Control Events ──────────────────────────────────────────────────

pub fn emit_role_granted(
    env: &Env,
    user: Address,
    role: String,
    granted_by: Address,
) {
    env.events().publish(
        (symbol_short!("role_gr"), user),
        (role, granted_by, env.ledger().timestamp()),
    );
}

pub fn emit_role_revoked(
    env: &Env,
    user: Address,
    role: String,
    revoked_by: Address,
) {
    env.events().publish(
        (symbol_short!("role_rev"), user),
        (role, revoked_by, env.ledger().timestamp()),
    );
}

// ── Pause Events ───────────────────────────────────────────────────────────

pub fn emit_paused(
    env: &Env,
    action: String,
    admin: Address,
) {
    env.events().publish(
        (symbol_short!("paused"), action),
        (admin, env.ledger().timestamp()),
    );
}

pub fn emit_unpaused(
    env: &Env,
    action: String,
    admin: Address,
) {
    env.events().publish(
        (symbol_short!("unpaused"), action),
        (admin, env.ledger().timestamp()),
    );
}

// ── Transfer Events ────────────────────────────────────────────────────────

pub fn emit_tokens_locked(
    env: &Env,
    task_id: u32,
    from: Address,
    amount: i128,
) {
    env.events().publish(
        (symbol_short!("lock"), task_id),
        (from, amount, env.ledger().timestamp()),
    );
}

pub fn emit_tokens_released(
    env: &Env,
    task_id: u32,
    to: Address,
    amount: i128,
) {
    env.events().publish(
        (symbol_short!("release"), task_id),
        (to, amount, env.ledger().timestamp()),
    );
}

// ── Platform Events ────────────────────────────────────────────────────────

/// Emitted when platform fee basis points are updated by an admin.
pub fn emit_fee_updated(
    env: &Env,
    old_fee_bps: u32,
    new_fee_bps: u32,
    updated_by: Address,
) {
    env.events().publish(
        (symbol_short!("fee_upd"), updated_by),
        (old_fee_bps, new_fee_bps, env.ledger().timestamp()),
    );
}

/// Emitted when the contract is first initialized.
pub fn emit_contract_initialized(
    env: &Env,
    admin: Address,
    fee_bps: u32,
) {
    env.events().publish(
        (symbol_short!("init"), admin),
        (fee_bps, env.ledger().timestamp()),
    );
}

/// Emitted when an authorized administrator executes an asset clawback operation.
pub fn emit_clawback_executed(
    env: &Env,
    admin: Address,
    from: Address,
    amount: i128,
) {
    env.events().publish(
        (symbol_short!("clawback"), admin),
        (from, amount, env.ledger().timestamp()),
    );
}

/// Emitted when a user initiates an anchor SEP-24 fiat withdrawal flow.
pub fn emit_sep24_session_created(
    env: &Env,
    account: Address,
    anchor_id: String,
) {
    env.events().publish(
        (symbol_short!("sep24_in"), account),
        (anchor_id, env.ledger().timestamp()),
    );
}
