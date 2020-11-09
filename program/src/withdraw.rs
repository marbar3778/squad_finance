use crate::access_control;
use fund::{
  accounts::fund::Fund,
  error::{FundError, FundErrorCode},
};
use serum_common::pack::Pack;
use serum_lockup::accounts::token_vault::TokenVault;
use solana_program::{
  account_info::{next_account_info, AccountInfo},
  info,
  program::invoke_signed,
  pubkey::Pubkey,
};
use spl_token::{instruction::transfer, ID};
use std::convert::Into;

pub fn handler(
  program_id: &Pubkey,
  accounts: &[AccountInfo],
  amount: u64,
) -> Result<(), FundError> {
  info!("handler: withdraw");

  let acc_infos = &mut accounts.iter();
  let vault_acc_info = next_account_info(acc_infos)?;
  let fund_acc_info = next_account_info(acc_infos)?;
  let withdraw_acc_info = next_account_info(acc_infos)?;
  let vault_authority_acc_info = next_account_info(acc_infos)?;
  let token_program_acc_info = next_account_info(acc_infos)?;

  access_control(AccessControlRequest {
    program_id,
    amount,
    fund_acc_info,
    withdraw_acc_info,
    vault_acc_info,
    vault_authority_acc_info,
  })?;

  Fund::unpack_mut(
    &mut fund_acc_info.try_borrow_mut_data()?,
    &mut |fund_acc: &mut Fund| {
      state_transistion(StateTransistionRequest {
        fund_acc,
        fund_acc_info,
        withdraw_acc_info,
        vault_acc_info,
        vault_authority_acc_info,
        token_program_acc_info,
        amount,
      })
      .map_err(Into::into)
    },
  )?;

  Ok(())
}

fn access_control(req: AccessControlRequest) -> Result<(), FundError> {
  let AccessControlRequest {
    program_id,
    amount,
    fund_acc_info,
    withdraw_acc_info,
    vault_acc_info,
    vault_authority_acc_info,
  } = req;
  // todo: check balance

  if !withdraw_acc_info.is_signer {
    return Err(FundErrorCode::Unauthorized)?;
  }

  let _ = access_control::fund(fund_acc_info, program_id)?;
  let _ = access_control::vault_join(
    vault_acc_info,
    vault_authority_acc_info,
    fund_acc_info,
    program_id,
  )?;

  let _ = access_control::withdraw(program_id, fund_acc_info, withdraw_acc_info);

  Ok(())
}

fn state_transistion(req: StateTransistionRequest) -> Result<(), FundError> {
  let StateTransistionRequest {
    fund_acc,
    fund_acc_info,
    withdraw_acc_info,
    vault_acc_info,
    vault_authority_acc_info,
    token_program_acc_info,
    amount,
  } = req;

  fund_acc.deduct(amount);
  // transfer from program account to owner of fund
  info!("invoking token transfer");
  let withdraw_instruction = transfer(
    &ID,
    vault_acc_info.key,
    withdraw_acc_info.key,
    &vault_authority_acc_info.key,
    &[],
    amount as u64,
  )?;

  let signer_seeds = TokenVault::signer_seeds(fund_acc_info.key, &fund_acc.nonce);

  invoke_signed(
    &withdraw_instruction,
    &[
      vault_acc_info.clone(),
      withdraw_acc_info.clone(),
      vault_authority_acc_info.clone(),
      token_program_acc_info.clone(),
    ],
    &[&signer_seeds],
  )?;

  Ok(())
}

struct AccessControlRequest<'a, 'b> {
  program_id: &'a Pubkey,
  amount: u64,
  fund_acc_info: &'a AccountInfo<'b>,
  withdraw_acc_info: &'a AccountInfo<'b>,
  vault_acc_info: &'a AccountInfo<'b>,
  vault_authority_acc_info: &'a AccountInfo<'b>,
}

struct StateTransistionRequest<'a, 'b, 'c> {
  fund_acc: &'c mut Fund,
  fund_acc_info: &'a AccountInfo<'b>,
  withdraw_acc_info: &'a AccountInfo<'b>,
  vault_acc_info: &'a AccountInfo<'b>,
  vault_authority_acc_info: &'a AccountInfo<'b>,
  token_program_acc_info: &'a AccountInfo<'b>,
  amount: u64,
}
