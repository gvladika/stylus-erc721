use alloc::{string::String, vec::Vec};
use alloy_primitives::FixedBytes;
use core::marker::PhantomData;
use stylus_sdk::{
    alloy_primitives::{Address, U256},
    alloy_sol_types::{sol, SolError},
    call::Call,
    evm, msg,
    prelude::*,
};

pub trait Erc721Params {
    const NAME: &'static str;
    const SYMBOL: &'static str;
}

sol_storage! {
    pub struct Erc721<T> {
        /// Token id to owner map
        mapping(uint256 => address) _owners;
        /// User to balance map
        mapping(address => uint256) _balances;
        /// Token id to approved user map
        mapping(uint256 => address) _approvals;
        /// User to operator map (the operator can manage all NFTs of the owner.)
        mapping(address => mapping(address => bool)) _approvals_for_all;
        /// Used to allow [`Erc721Params`]
        PhantomData<T> phantom;
    }
}

// Declare events and Solidity error types
sol! {
    event Transfer(address indexed from, address indexed to, uint256 indexed token_id);
    event Approval(address indexed owner, address indexed spender, uint256 indexed token_id);
    event ApprovalForAll(address indexed owner, address indexed operator, bool approved);

    error NotOwner(address account, uint256 token_id);
    error NotAuthorized(address caller);
    error InvalidRecipient(address to);
    error AlreadyMinted(uint256 token_id);
    error NotMinted(uint256 token_id);
    error UnsafeRecipient(address recipient);
    error CallFailed();
}

sol_interface! {
    interface IERC721TokenReceiver {
        function onERC721Received(address operator, address from, uint256 token_id, bytes data) external returns(bytes4);
    }
}

pub enum Erc721Error {
    NotOwner(NotOwner),
    NotAuthorized(NotAuthorized),
    InvalidRecipient(InvalidRecipient),
    AlreadyMinted(AlreadyMinted),
    NotMinted(NotMinted),
    UnsafeRecipient(UnsafeRecipient),
    CallFailed(CallFailed),
}

impl From<Erc721Error> for Vec<u8> {
    fn from(err: Erc721Error) -> Vec<u8> {
        match err {
            Erc721Error::NotOwner(e) => e.encode(),
            Erc721Error::NotAuthorized(e) => e.encode(),
            Erc721Error::InvalidRecipient(e) => e.encode(),
            Erc721Error::AlreadyMinted(e) => e.encode(),
            Erc721Error::NotMinted(e) => e.encode(),
            Erc721Error::UnsafeRecipient(e) => e.encode(),
            Erc721Error::CallFailed(e) => e.encode(),
        }
    }
}

const ADDRESS_ZERO: Address = Address(FixedBytes([0u8; 20]));
const ERC721_TOKEN_RECEIVER_ID: u32 = 0x150b7a02;

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

    pub fn get_approved(&self, token_id: U256) -> Result<Address, Erc721Error> {
        Ok(self._approvals.get(token_id))
    }

    pub fn is_approved_for_all(
        &self,
        owner: Address,
        operator: Address,
    ) -> Result<bool, Erc721Error> {
        Ok(self._approvals_for_all.get(owner).get(operator))
    }

    pub fn approve(&mut self, spender: Address, token_id: U256) -> Result<(), Erc721Error> {
        // address owner = _ownerOf[id];
        let owner = self._owners.getter(token_id).get();

        // require(msg.sender == owner || isApprovedForAll[owner][msg.sender], "NOT_AUTHORIZED");
        if msg::sender() != owner && !self._approvals_for_all.get(owner).get(msg::sender()) {
            return Err(Erc721Error::NotOwner(NotOwner {
                account: owner,
                token_id,
            }));
        }

        // getApproved[id] = spender;
        let mut spender_of = self._approvals.setter(token_id);
        spender_of.set(spender);

        // emit Approval(owner, spender, id);
        evm::log(Approval {
            owner,
            spender,
            token_id,
        });

        Ok(())
    }

    pub fn set_approval_for_all(
        &mut self,
        operator: Address,
        approved: bool,
    ) -> Result<(), Erc721Error> {
        // isApprovedForAll[msg.sender][operator] = approved;
        let mut operator_setter = self._approvals_for_all.setter(msg::sender());
        let mut approval_setter = operator_setter.setter(operator);
        approval_setter.set(approved);

        // emit ApprovalForAll(msg.sender, operator, approved);
        evm::log(ApprovalForAll {
            owner: msg::sender(),
            operator,
            approved,
        });

        Ok(())
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
            return Err(Erc721Error::NotOwner(NotOwner {
                account: from,
                token_id,
            }));
        }

        // require(to != address(0), "INVALID_RECIPIENT");
        if to == ADDRESS_ZERO {
            return Err(Erc721Error::InvalidRecipient(InvalidRecipient { to }));
        }

        // require(msg.sender == from || isApprovedForAll[from][msg.sender] || msg.sender == getApproved[id], "NOT_AUTHORIZED");
        if msg::sender() != from
            && self._approvals_for_all.get(from).get(msg::sender())
            && msg::sender() != self._approvals.get(token_id)
        {
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

        // delete getApproved[id];
        self._approvals.setter(token_id).set(ADDRESS_ZERO);

        evm::log(Transfer { from, to, token_id });

        Ok(())
    }

    pub fn safe_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), Erc721Error> {
        // transferFrom(from, to, id);
        self.transfer_from(from, to, token_id)?;

        self._check_recipient_is_valid(from, to, token_id)?;

        Ok(())
    }
}

// internal mint+burn methods
impl<T: Erc721Params> Erc721<T> {
    pub fn _mint(&mut self, to: Address, token_id: U256) -> Result<(), Erc721Error> {
        // require(to != address(0), "INVALID_RECIPIENT");
        if to == ADDRESS_ZERO {
            return Err(Erc721Error::InvalidRecipient(InvalidRecipient { to }));
        }

        // require(_ownerOf[id] == address(0), "ALREADY_MINTED");
        let mut owner_of_id = self._owners.setter(token_id);
        if owner_of_id.get() != ADDRESS_ZERO {
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
            from: ADDRESS_ZERO,
            to,
            token_id,
        });

        Ok(())
    }

    pub fn _burn(&mut self, token_id: U256) -> Result<(), Erc721Error> {
        // address owner = _ownerOf[id];
        let mut owner = self._owners.setter(token_id);

        // require(owner != address(0), "NOT_MINTED");
        if owner.get() == ADDRESS_ZERO {
            return Err(Erc721Error::NotMinted(NotMinted { token_id }));
        }

        // _balanceOf[owner]--;
        let mut owner_balance = self._balances.setter(owner.get());
        let new_owner_balance = owner_balance.get() - U256::from(1);
        owner_balance.set(new_owner_balance);

        // delete _ownerOf[id];
        owner.set(ADDRESS_ZERO);

        // delete getApproved[id];
        self._approvals.setter(token_id).set(ADDRESS_ZERO);

        // emit Transfer(owner, address(0), id);
        evm::log(Transfer {
            from: owner.get(),
            to: ADDRESS_ZERO,
            token_id,
        });

        Ok(())
    }

    pub fn _safe_mint(&mut self, to: Address, token_id: U256) -> Result<(), Erc721Error> {
        // _mint(to, id);
        self._mint(to, token_id)?;

        self._check_recipient_is_valid(ADDRESS_ZERO, to, token_id)?;

        Ok(())
    }

    pub fn _check_recipient_is_valid(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), Erc721Error> {
        let receiver = IERC721TokenReceiver::new(to);
        let config = Call::new();
        let hook_result = receiver
            .on_erc_721_received(config, msg::sender(), from, token_id, vec![])
            .map_err(|_e| Erc721Error::CallFailed(CallFailed {}))?;

        // require(to.code.length == 0 || ERC721TokenReceiver(to).onERC721Received(msg.sender, from, id, "") == ERC721TokenReceiver.onERC721Received.selector, "UNSAFE_RECIPIENT");
        if to.has_code() && u32::from_be_bytes(hook_result.0) != ERC721_TOKEN_RECEIVER_ID {
            return Err(Erc721Error::UnsafeRecipient(UnsafeRecipient {
                recipient: to,
            }));
        }

        Ok(())
    }
}
