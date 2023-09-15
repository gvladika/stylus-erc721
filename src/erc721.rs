use alloc::{string::String, vec::Vec};
use alloy_primitives::FixedBytes;
use core::marker::PhantomData;
use stylus_sdk::{
    alloy_primitives::{Address, U256},
    alloy_sol_types::{sol, SolError},
    evm, msg,
    prelude::*,
};

pub trait Erc721Params {
    const NAME: &'static str;
    const SYMBOL: &'static str;
}

sol_storage! {
    pub struct Erc721<T> {
        /// NFT id to owner map
        mapping(uint256 => address) _owners;
        /// User to balance map
        mapping(address => uint256) _balances;
        /// Used to allow [`Erc721Params`]
        PhantomData<T> phantom;
    }
}

// Declare events and Solidity error types
sol! {
    event Transfer(address indexed from, address indexed to, uint256 indexed token_id);

    error NotOwner(address from, uint256 token_id);
    error NotAuthorized(address caller);
    error InvalidRecipient(address to);
    error AlreadyMinted(uint256 token_id);
}

pub enum Erc721Error {
    NotOwner(NotOwner),
    NotAuthorized(NotAuthorized),
    InvalidRecipient(InvalidRecipient),
    AlreadyMinted(AlreadyMinted),
}

impl From<Erc721Error> for Vec<u8> {
    fn from(err: Erc721Error) -> Vec<u8> {
        match err {
            Erc721Error::NotOwner(e) => e.encode(),
            Erc721Error::NotAuthorized(e) => e.encode(),
            Erc721Error::InvalidRecipient(e) => e.encode(),
            Erc721Error::AlreadyMinted(e) => e.encode(),
        }
    }
}

// These methods are external to other contracts
#[external]
impl<T: Erc721Params> Erc721<T> {
    pub fn name() -> Result<String, Vec<u8>> {
        Ok(T::NAME.into())
    }

    pub fn symbol() -> Result<String, Vec<u8>> {
        Ok(T::SYMBOL.into())
    }

    pub fn balance_of(&self, owner: Address) -> Result<U256, Erc721Error> {
        Ok(self._balances.get(owner))
    }

    pub fn owner_of(&self, token_id: U256) -> Result<Address, Erc721Error> {
        Ok(self._owners.get(token_id))
    }

    pub fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), Erc721Error> {
        // require(from == _ownerOf[id], "WRONG_FROM");
        let mut owner_of_id = self._owners.setter(token_id);
        if owner_of_id.get() != from {
            return Err(Erc721Error::NotOwner(NotOwner { from, token_id }));
        }

        // require(to != address(0), "INVALID_RECIPIENT");
        let address_zero: Address = Address(FixedBytes([0u8; 20]));
        if to == address_zero {
            return Err(Erc721Error::InvalidRecipient(InvalidRecipient { to }));
        }

        // require(msg.sender == from || isApprovedForAll[from][msg.sender] || msg.sender == getApproved[id], "NOT_AUTHORIZED");
        if msg::sender() != from {
            return Err(Erc721Error::NotAuthorized(NotAuthorized {
                caller: msg::sender(),
            }));
        }

        // _balanceOf[from]--;
        let mut from_balance = self._balances.setter(from);
        let new_from_balance = from_balance.get() - U256::from(1);
        from_balance.set(new_from_balance);

        // _balanceOf[to]++;
        let mut to_balance = self._balances.setter(to);
        let new_to_balance = to_balance.get() + U256::from(1);
        to_balance.set(new_to_balance);

        // _ownerOf[id] = to;
        owner_of_id.set(to);

        evm::log(Transfer { from, to, token_id });

        Ok(())
    }
}

// internal methods
impl<T: Erc721Params> Erc721<T> {
    pub fn _mint(&mut self, to: Address, token_id: U256) -> Result<(), Erc721Error> {
        let address_zero: Address = Address(FixedBytes([0u8; 20]));

        // require(to != address(0), "INVALID_RECIPIENT");
        if to == address_zero {
            return Err(Erc721Error::InvalidRecipient(InvalidRecipient { to }));
        }

        // require(_ownerOf[id] == address(0), "ALREADY_MINTED");
        let mut owner_of_id = self._owners.setter(token_id);
        if owner_of_id.get() != address_zero {
            return Err(Erc721Error::AlreadyMinted(AlreadyMinted { token_id }));
        }

        // _balanceOf[to]++;
        let mut to_balance = self._balances.setter(to);
        let new_to_balance = to_balance.get() + U256::from(1);
        to_balance.set(new_to_balance);

        // _ownerOf[id] = to;
        owner_of_id.set(to);

        // emit Transfer(address(0), to, id);
        evm::log(Transfer {
            from: address_zero,
            to,
            token_id,
        });

        Ok(())
    }
}
