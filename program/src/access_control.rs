use fund::{
    accounts::{fund::Fund, vault::TokenVault, whitelist::Whitelist},
    error::{FundError, FundErrorCode},
};
use serum_common::pack::Pack;
use solana_program::sysvar::Sysvar;
use solana_program::{
    account_info::AccountInfo, program_pack::Pack as TokenPack, pubkey::Pubkey, sysvar::rent::Rent,
};

use spl_token::state::{Account as TokenAccount, Mint};

pub fn token(acc_info: &AccountInfo) -> Result<TokenAccount, FundError> {
    if *acc_info.owner != spl_token::ID {
        return Err(FundErrorCode::InvalidAccountOwner.into());
    }

    let token = TokenAccount::unpack(&acc_info.try_borrow_data()?)?;
    if token.state != spl_token::state::AccountState::Initialized {
        return Err(FundErrorCode::NotInitialized.into());
    }

    Ok(token)
}

pub fn fund(acc_info: &AccountInfo, program_id: &Pubkey) -> Result<Fund, FundError> {
    if acc_info.owner != program_id {
        return Err(FundErrorCode::InvalidAccountOwner.into());
    }

    let fund = Fund::unpack(&acc_info.try_borrow_data()?)?;
    if !fund.initialized {
        return Err(FundErrorCode::NotInitialized.into());
    }

    Ok(fund)
}

pub fn whitelist<'a>(
    wl_acc_info: AccountInfo<'a>,
    fund: &Fund,
    program_id: &Pubkey,
) -> Result<Whitelist<'a>, FundError> {
    if program_id != wl_acc_info.owner {
        return Err(FundErrorCode::InvalidAccountOwner.into());
    }

    if fund.whitelist != *wl_acc_info.key {
        return Err(FundErrorCode::InvalidWhitelist.into());
    }
    Whitelist::new(wl_acc_info).map_err(Into::into)
}

pub fn check_owner(
    program_id: &Pubkey,
    acc_info: &AccountInfo,
    owner_acc_info: &AccountInfo,
) -> Result<(), FundError> {
    if !owner_acc_info.is_signer {
        return Err(FundErrorCode::Unauthorized.into());
    }

    let fund = fund(acc_info, program_id)?;

    if !fund.owner.eq(owner_acc_info.key) {
        return Err(FundErrorCode::InvalidAccountOwner.into());
    }

    Ok(())
}

pub fn fund_open(acc_info: &AccountInfo, program_id: &Pubkey) -> Result<(), FundError> {
    let fund = fund(acc_info, program_id)?;

    if !fund.open {
        return Err(FundErrorCode::FundClosed.into());
    }

    Ok(())
}

pub fn mint(acc_info: &AccountInfo) -> Result<Mint, FundError> {
    if *acc_info.owner != spl_token::ID {
        return Err(FundErrorCode::InvalidMint.into());
    }

    let mint = Mint::unpack(&acc_info.try_borrow_data()?)?;
    if !mint.is_initialized {
        return Err(FundErrorCode::UnitializedTokenMint.into());
    }

    Ok(mint)
}

pub fn rent(acc_info: &AccountInfo) -> Result<Rent, FundError> {
    if *acc_info.key != solana_program::sysvar::rent::id() {
        return Err(FundErrorCode::InvalidRentSysvar.into());
    }
    Rent::from_account_info(acc_info).map_err(Into::into)
}

pub fn vault(
    acc_info: &AccountInfo,
    vault_authority_acc_info: &AccountInfo,
    fund_acc_info: &AccountInfo,
    program_id: &Pubkey,
) -> Result<TokenAccount, FundError> {
    let fund = fund(fund_acc_info, program_id)?;
    let vault = token(acc_info)?;
    if *acc_info.key != fund.vault {
        return Err(FundErrorCode::InvalidVault.into());
    }

    let va = vault_authority(
        vault_authority_acc_info,
        fund_acc_info.key,
        &fund,
        program_id,
    )?;

    if va != vault.owner {
        return Err(FundErrorCode::InvalidVault.into());
    }
    if va != *vault_authority_acc_info.key {
        return Err(FundErrorCode::InvalidVault.into());
    }

    Ok(vault)
}

pub fn vault_join(
    acc_info: &AccountInfo,
    vault_authority_acc_info: &AccountInfo,
    fund_acc_info: &AccountInfo,
    program_id: &Pubkey,
) -> Result<TokenAccount, FundError> {
    let fund = fund(fund_acc_info, program_id)?;
    let vault = vault(
        acc_info,
        vault_authority_acc_info,
        fund_acc_info,
        program_id,
    )?;
    let va = vault_authority(
        vault_authority_acc_info,
        fund_acc_info.key,
        &fund,
        program_id,
    )?;

    if va != vault.owner {
        return Err(FundErrorCode::InvalidVault.into());
    }
    if va != *vault_authority_acc_info.key {
        return Err(FundErrorCode::InvalidVault.into());
    }

    Ok(vault)
}

pub fn vault_authority(
    vault_authority_acc_info: &AccountInfo,
    fund_addr: &Pubkey,
    fund: &Fund,
    program_id: &Pubkey,
) -> Result<Pubkey, FundError> {
    let va = Pubkey::create_program_address(
        &TokenVault::signer_seeds(fund_addr, &fund.nonce),
        program_id,
    )
    .map_err(|_| FundErrorCode::InvalidVaultNonce)?;
    if va != *vault_authority_acc_info.key {
        return Err(FundErrorCode::InvalidVault.into());
    }

    Ok(va)
}

pub fn withdraw(
    program_id: &Pubkey,
    fund_acc_info: &AccountInfo,
    withdraw_acc_beneficiary_info: &AccountInfo,
) -> Result<Fund, FundError> {
    let fund = Fund::unpack(&fund_acc_info.try_borrow_data()?)?;

    if fund_acc_info.owner != program_id {
        return Err(FundErrorCode::InvalidAccount.into());
    }
    if !fund.initialized {
        return Err(FundErrorCode::NotInitialized.into());
    }
    if fund.owner != *withdraw_acc_beneficiary_info.key {
        return Err(FundErrorCode::Unauthorized.into());
    }

    Ok(fund)
}

pub fn check_balance(fund_acc_info: &AccountInfo, amount: u64) -> Result<(), FundError> {
    let fund = Fund::unpack(&fund_acc_info.try_borrow_data()?)?;

    if fund.balance + amount > fund.max_balance {
        return Err(FundErrorCode::FundBalanceOverflow.into());
    }

    Ok(())
}

pub fn check_depositor<'a>(
    program_id: &Pubkey,
    wl_acc_info: AccountInfo<'a>,
    fund: &Fund,
    depositor_acc_info: &AccountInfo<'a>,
) -> Result<(), FundError> {
    if program_id != wl_acc_info.owner {
        return Err(FundErrorCode::InvalidAccountOwner.into());
    }

    if fund.whitelist != *wl_acc_info.key {
        return Err(FundErrorCode::InvalidWhitelist.into());
    }

    let wl: Result<Whitelist, FundError> = Whitelist::new(wl_acc_info).map_err(Into::into);

    let _ = wl.unwrap().index_of(depositor_acc_info.key)?;

    Ok(())
}

pub fn check_nft<'a>(
    fund: &Fund,
    mint_acc_info: &AccountInfo<'a>,
    token_acc_info: &AccountInfo<'a>,
) -> Result<(), FundError> {
    let token_acc = token(token_acc_info)?;
    if token_acc.mint != fund.nft_mint {
        return Err(FundErrorCode::InvalidTokenAccountMint.into());
    }
    if token_acc.mint != *mint_acc_info.key {
        return Err(FundErrorCode::InvalidMint.into());
    }
    Ok(())
}
