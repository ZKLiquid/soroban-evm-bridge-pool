use crate::storage_types::{DataKey, BALANCE_BUMP_AMOUNT, BALANCE_LIFETIME_THRESHOLD};
use soroban_sdk::{Address, Env};

pub fn read_user_balance(e: &Env, addr: Address) -> i128 {
    let key = DataKey::DepositBalance(addr);
    if let Some(balance) = e.storage().persistent().get::<DataKey, i128>(&key) {
        e.storage()
            .persistent()
            .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
        balance
    } else {
        0
    }
}

fn write_user_balance(e: &Env, addr: Address, amount: i128) {
    let key = DataKey::DepositBalance(addr);
    e.storage().persistent().set(&key, &amount);
    e.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
}

pub fn read_pool_balance(e: &Env) -> i128 {
    let key = DataKey::TotalPoolBalance;
    if let Some(balance) = e.storage().persistent().get::<DataKey, i128>(&key) {
        e.storage()
            .persistent()
            .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
        balance
    } else {
        0
    }
}

pub fn write_pool_balance(e: &Env, amount: i128) {
    if amount < 0 {
        panic!("Amount must be non-negative");
    }
    let key = DataKey::TotalPoolBalance;
    let cur_balance=read_pool_balance(e);
    let new_balance=amount+cur_balance;
    e.storage().persistent().set(&key, &new_balance);
    e.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
}

pub fn read_depositors(e: &Env) -> i128 {
    let key = DataKey::TotalDepositors;
    if let Some(count) = e.storage().persistent().get::<DataKey, i128>(&key) {
        e.storage()
            .persistent()
            .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
        count
    } else {
        0
    }
}

fn write_depositors(e: &Env, inc: i128) {
    let key = DataKey::TotalDepositors;
    let cur_count=read_depositors(e);
    let new_count=cur_count+inc;
    e.storage().persistent().set(&key, &new_count);
    e.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
}


pub fn add_balance(e: &Env, addr: Address, amount: i128) {
    let balance = read_user_balance(e, addr.clone());
    write_user_balance(e, addr, balance + amount);
    write_pool_balance(e, amount);
    if balance==0 {
        write_depositors(e, 1);
    }
}

pub fn subtract_balance(e: &Env, addr: Address, amount: i128) {
    let balance = read_user_balance(e, addr.clone());
    if balance < amount {
        panic!("insufficient balance");
    }
    write_user_balance(e, addr.clone(), balance - amount);
    write_pool_balance(e, -amount);
    let balance = read_user_balance(e, addr);
    if balance == 0 {
        write_depositors(e, -1)
    }
}
