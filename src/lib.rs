#![no_std]
mod admin;
mod storage_types;
use crate::storage_types::DataKey;
use crate::admin::{has_administrator, read_administrator, write_administrator};

use soroban_sdk::{contract, contractimpl, token, Address, Env, String};
// use soroban_sdk::String;


pub trait MultiChainTransferTrait {
    fn initialize(e: Env, admin: Address);
    fn multichain_transfer(env: Env, from: Address, to: Address, token_address: Address, amount: i128);
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

    fn multichain_transfer(
        env: Env,
        from: Address,
        to: Address,
        token_address: Address,
        amount: i128,
    ) {
        // Verify preconditions on the minimum price for both parties.
        if amount < 0 {
            panic!("not enough token A for token B");
        }
        from.require_auth();
        // Perform the swap by moving tokens from a to b and from b to a.
        move_token(&env, &token_address, &from,  amount);

        env.events().publish((from.clone(), "multichain_transfer"),(amount, Self::get_token_evm_address(&env, token_address.clone()), to.clone()))
    }


    // fn get_token_evm_address(e: &Env, soroban_address: Address) -> Option<String> {
    //     let address_map: Map<Address, String> = e.storage().persistent().get(&DataKey::AddressMap).unwrap();
    //     address_map.get(soroban_address).clone()
    // }

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

fn move_token(
    env: &Env,
    token_address: &Address,
    from: &Address,
    transfer_amount: i128,
) {
    let token = token::Client::new(env, token_address);
    let contract_address = env.current_contract_address();
   
    token.transfer(from, &contract_address, &transfer_amount);

}



mod test;
