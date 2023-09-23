# Stylus ERC721

Basic ERC721 contract written in Rust for Arbitrum Stylus. Based on Solmate's implementation.

# Deployment
Check compilation works and deploy it
```
cargo stylus check
cargo stylus deploy --private-key-path="..."
```

# Actions
Checks balance of and owner
```
cast call --rpc-url $STYLUS_RPC $NFT "balanceOf(address) (uint256)" $USER
cast call --rpc-url $STYLUS_RPC $NFT "ownerOf(uint256) (address)" 0
```

Mint
```
cast send --rpc-url $STYLUS_RPC --private-key $USER_PRIV_KEY  $NFT "mint(address)" $USER
```

TransferFrom
```
cast send --rpc-url $STYLUS_RPC --private-key $USER_PRIV_KEY  $NFT "transferFrom(address,address,uint256)" $USER $RECEIVER 0
```

Burn
```
cast send --rpc-url $STYLUS_RPC --private-key $USER_PRIV_KEY  $NFT "burn(uint256)" 1
```

SafeMint
```
cast send --rpc-url $STYLUS_RPC --private-key $USER_PRIV_KEY  $NFT "safeMint(address)" $USER
```

SafeTransferFrom
```
cast send --rpc-url $STYLUS_RPC --private-key $USER_PRIV_KEY  $NFT "safeTransferFrom(address,address,uint256)" $USER $RECEIVER 0
```