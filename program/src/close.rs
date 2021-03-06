use crate::access_control;
use fund::{accounts::fund::Fund, error::FundError};
use serum_common::pack::Pack;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    msg,
    pubkey::Pubkey,
};

pub fn handler(program_id: &Pubkey, accounts: &[AccountInfo]) -> Result<(), FundError> {
    msg!("handler close initiate");
    let acc_info = &mut accounts.iter();

    let fund_acc_info = next_account_info(acc_info)?;
    let fund_owner_acc_info = next_account_info(acc_info)?;

    access_control(AccessControlRequest {
        program_id,
        fund_acc_info,
        fund_owner_acc_info,
    })?;

    Fund::unpack_mut(
        &mut fund_acc_info.try_borrow_mut_data()?,
        &mut |fund_acc: &mut Fund| {
            state_transition(StateTransistionRequest { fund_acc }).map_err(Into::into)
        },
    )?;

    Ok(())
}

fn access_control(req: AccessControlRequest) -> Result<(), FundError> {
    let AccessControlRequest {
        program_id,
        fund_acc_info,
        fund_owner_acc_info,
    } = req;

    let _ = access_control::check_owner(program_id, fund_acc_info, fund_owner_acc_info)?;

    msg!("access control close success");

    Ok(())
}

fn state_transition(req: StateTransistionRequest) -> Result<(), FundError> {
    let StateTransistionRequest { fund_acc } = req;

    fund_acc.close_fund();

    msg!("state transition close success");

    Ok(())
}
struct AccessControlRequest<'a, 'b> {
    program_id: &'a Pubkey,
    fund_acc_info: &'a AccountInfo<'b>,
    fund_owner_acc_info: &'a AccountInfo<'b>,
}

struct StateTransistionRequest<'c> {
    fund_acc: &'c mut Fund,
}
