use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    msg,
    program::invoke,
};
use spl_token::{instruction::initialize_mint, instruction::mint_to};
use solana_program::{
    program_pack::Pack,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};
use mpl_token_metadata::{
    instruction::{create_metadata_accounts_v2},
    state::Creator,
};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct MintConfig {
    pub initial_supply: u64,
    pub token_name: String,
    pub token_symbol: String,
    pub token_uri: String,
}

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let config = MintConfig::try_from_slice(instruction_data)?;
    let mint_pubkey = &accounts[0].key;
    let owner_pubkey = &accounts[1].key;
    let rent_sysvar = &accounts[2];
    let token_program = &accounts[3];

    msg!("Creating MLINK token with symbol {} and supply {}", config.token_symbol, config.initial_supply);

    // Initialize the mint (token creation)
    let mint_instruction = initialize_mint(
        &spl_token::ID,
        mint_pubkey,
        owner_pubkey,
        Some(owner_pubkey),
        9, // 9 decimals like SOL
    )?;
    invoke(
        &mint_instruction,
        &[accounts[0].clone(), accounts[1].clone(), rent_sysvar.clone(), token_program.clone()],
    )?;

    // Mint the initial supply to the owner's account
    let mint_to_instruction = mint_to(
        &spl_token::ID,
        mint_pubkey,
        &accounts[4].key, // Recipient token account
        owner_pubkey,
        &[], // No multisignature required
        config.initial_supply,
    )?;
    invoke(
        &mint_to_instruction,
        &[accounts[0].clone(), accounts[1].clone(), accounts[4].clone()],
    )?;

    // Create token metadata (name, symbol, image)
    create_token_metadata(
        mint_pubkey,
        owner_pubkey,
        &accounts[5].key, // payer
        config.token_name,
        config.token_symbol,
        config.token_uri, // Link to image metadata (from Pinata)
    )?;

    Ok(())
}

pub fn create_token_metadata(
    mint_pubkey: &Pubkey,
    mint_authority: &Pubkey,
    payer: &Pubkey,
    token_name: String,
    token_symbol: String,
    token_uri: String, // Link to image or metadata
) -> ProgramResult {
    let metadata_accounts = create_metadata_accounts_v2(
        mpl_token_metadata::ID,
        *mint_pubkey,
        *mint_authority,
        *mint_authority,
        *payer,
        *payer,
        token_name,
        token_symbol,
        token_uri,
        None,  // No creators (optional)
        0,     // No seller fees for MLINK
        true,  // Update authority is signer
        true,  // Is mutable
    );

    solana_program::program::invoke(
        &metadata_accounts,
        &[
            mint_pubkey.clone(),
            mint_authority.clone(),
            payer.clone(),
        ],
    )?;
    Ok(())
}
