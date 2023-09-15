// Only run this as a WASM if the export-abi feature is not set.
#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

/// Initializes a custom, global allocator for Rust programs compiled to WASM.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use crate::erc721::{Erc721, Erc721Params};
use alloy_primitives::{Address, U256};
use erc721::Erc721Error;
/// Import the Stylus SDK along with alloy primitive types for use in our program.
use stylus_sdk::prelude::*;

/// import module
mod erc721;

struct StylusNFTParams;

/// Immutable definitions
impl Erc721Params for StylusNFTParams {
    const NAME: &'static str = "StylusNFT";
    const SYMBOL: &'static str = "SNFT";
}

// Define the entrypoint as a Solidity storage object, in this case a struct
// called `Counter` with a single uint256 value called `number`. The sol_storage! macro
// will generate Rust-equivalent structs with all fields mapped to Solidity-equivalent
// storage slots and types.
sol_storage! {
    #[entrypoint]
    struct StylusNFT {
        #[borrow] // Allows erc721 to access MyToken's storage and make calls
        Erc721<StylusNFTParams> erc721;
    }
}

#[external]
#[inherit(Erc721<StylusNFTParams>)]
impl StylusNFT {
    pub fn mint(&mut self, to: Address, token_id: U256) -> Result<(), Erc721Error> {
        self.erc721._mint(to, token_id)?;
        Ok(())
    }

    pub fn burn(&mut self, token_id: U256) -> Result<(), Erc721Error> {
        self.erc721._burn(token_id)?;
        Ok(())
    }
}
