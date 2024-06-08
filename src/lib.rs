#![no_std]
mod admin;
mod balance;
mod storage_types;
use crate::storage_types::DataKey;
use crate::admin::{has_administrator, read_administrator, write_administrator};
use crate::balance::{read_user_balance, add_balance, subtract_balance, read_pool_balance, read_depositors};
use soroban_sdk::{contract, contractimpl, token, Address, Env, String};
// use soroban_sdk::String;

pub trait MultiChainTransferTrait {
    fn initialize(e: Env, admin: Address);
    fn transfer_to_evm(env: Env, from: Address, to: String, token_address: Address, amount: i128);
    fn transfer_payout(env: Env, oracle: Address, to: Address, token_address: Address, amount: i128);

    fn deposit_token( env: Env, from: Address, token_address: Address, amount: i128);
    fn withdraw_token( env: Env, to: Address, token_address: Address, amount: i128);
    fn get_user_balance(e: &Env, user: Address) -> i128;
    fn get_pool_balance(e: &Env)-> i128 ;
    fn get_users_count(e: &Env) -> i128;
    

    fn get_token_evm_address(e: &Env, soroban_address: Address) -> Option<String>;
    fn set_token_evm_address(e: &Env, soroban_address: Address, evm_address_string: String);
}

#[contract]
pub struct MultiChainBridge;

#[contractimpl]
impl MultiChainTransferTrait for MultiChainBridge {

    fn initialize(e: Env, admin: Address) {
        if has_administrator(&e) {
            panic!("already has an admin")
        }
        write_administrator(&e, &admin);
       
    }

    fn transfer_to_evm(
        env: Env,
        from: Address,
        to: String,
        token_address: Address,
        amount: i128,
    ) {
        // Verify preconditions on the minimum price for both parties.
        if amount < 0 {
            panic!("not enough token A for token B");
        }
        from.require_auth();
        // Perform the swap by moving tokens from a to b and from b to a.
        take_token(&env, &token_address, &from,  amount);

        env.events().publish((from.clone(), "multichain_transfer_sent"),(amount, Self::get_token_evm_address(&env, token_address.clone()), to.clone()))
    }

    //Send receieved transfer from evm blockchain to the recipient 
    fn transfer_payout(
        env: Env,
        oracle: Address,
        to: Address,
        token_address: Address,
        amount: i128,
    ) {
        // Verify preconditions on the minimum price for both parties.
        if amount < 0 {
            panic!("not enough token A for token B");
        }
        oracle.require_auth();
        // Perform the swap by moving tokens from a to b and from b to a.
        send_token(&env, &token_address, &to,  amount);

        env.events().publish((to.clone(), "multichain_transfer_received"),(amount, Self::get_token_evm_address(&env, token_address.clone()), to.clone()))
    }


    fn deposit_token( env: Env,
        from: Address,
        token_address: Address,
        amount: i128,){
            from.require_auth();
            if amount <= 0 {
                panic!("Amount must be positive");
            }

            take_token(&env, &token_address, &from,  amount);
            add_balance(&env, from.clone(), amount);
        }

    fn withdraw_token( env: Env,
        to: Address,
        token_address: Address,
        amount: i128,){
            to.require_auth();
            if amount <= 0 {
                panic!("Amount must be positive");
            }
            let cur_balance = read_user_balance(&env, to.clone());

            if amount > cur_balance {
                panic!("You cannot withdraw an amount greater than your balance");
            }
            subtract_balance(&env, to.clone(), amount);
            send_token(&env, &token_address, &to,  amount);
        }


        fn get_user_balance(e: &Env, user: Address)-> i128 {
            let balance=read_user_balance(e, user);
            return  balance;
        }

        fn get_pool_balance(e: &Env)-> i128 {
            let balance=read_pool_balance(e);
            return  balance;
        }

        fn get_users_count(e: &Env) -> i128 {
            let count = read_depositors(e);
            return  count;
        }


    fn get_token_evm_address(e: &Env, soroban_address: Address) -> Option<String>{
        let key =DataKey::SorobanEth(soroban_address);
        let soroban_eth=e.storage().persistent().get::<DataKey,String>(&key);
        return  soroban_eth;
    }
    
    fn set_token_evm_address(e: &Env, soroban_address: Address, evm_address_string: String) {
        let admin = read_administrator(&e);
        admin.require_auth();
        let key =DataKey::SorobanEth(soroban_address);
        e.storage().persistent().set(&key, &evm_address_string);        
    }
}

fn take_token(
    env: &Env,
    token_address: &Address,
    from: &Address,
    transfer_amount: i128,
) {
    let token = token::Client::new(env, token_address);
    let contract_address = env.current_contract_address();
   
    token.transfer(from, &contract_address, &transfer_amount);

}

fn send_token(
    env: &Env,
    token_address: &Address,
    to: &Address,
    transfer_amount: i128,
) {
    let token = token::Client::new(env, token_address);
    let contract_address = env.current_contract_address();
   
    token.transfer(&contract_address, to, &transfer_amount);

}

mod test;
