mod utils;

use solana_program::{
    pubkey::Pubkey,
    account_info::AccountInfo,
    program_error::ProgramError,
    entrypoint::ProgramResult,
    entrypoint,
    rent::Rent,
    sysvar::Sysvar,
    program,
    msg,
    system_instruction,
    system_program
};
use utils::Image;

entrypoint!(process_transaction);

pub fn process_transaction(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let mut accounts_iter = accounts.iter();
    let signer = accounts_iter.next()
        .unwrap();
    let pda_account = accounts_iter.next()
        .unwrap();
    let system_program = accounts_iter.next()
        .unwrap();

    if !system_program::check_id(system_program.key) {
        return Err(ProgramError::InvalidArgument);
    }

    let account_len = data.len();

    if account_len == 0 {
        return close_account(signer, pda_account, program_id);
    }

    let rent = Rent::get()
        .unwrap();
    let rent_lamports = rent.minimum_balance(account_len);

    let image: Image = bincode::deserialize(data)
        .unwrap();

    let (pda, bump_seed) = Pubkey::find_program_address(
        &[signer.key.as_ref(), image.title.as_bytes()],
        program_id
    );

    if pda.ne(pda_account.key) {
        return Err(ProgramError::InvalidArgument);
    }

    program::invoke_signed(
        &system_instruction::create_account(
            signer.key,
            pda_account.key,
            rent_lamports,
            account_len as u64,
            program_id
        ),
        &[signer.clone(), pda_account.clone(), system_program.clone()],
        &[&[signer.key.as_ref(), image.title.as_bytes(), &[bump_seed]]]
    ).unwrap();
    msg!("PDA Created: {}", pda);

    pda_account.serialize_data(&image)
        .unwrap();

    Ok(())
}

fn close_account(signer: &AccountInfo, pda: &AccountInfo, program_id: &Pubkey) -> Result<(), ProgramError> {
    let image: Image = bincode::deserialize(&pda.data.borrow()).unwrap();

    let (derived_pda, _bump_seed) = Pubkey::find_program_address(
        &[signer.key.as_ref(), image.title.as_bytes()],
        program_id
    );

    if derived_pda.to_string() != pda.key.to_string() {
        msg!("Error: Invalid Signer");
        msg!("Input PDA: {}", pda.key);
        msg!("Derived PDA: {}", derived_pda);

        return Err(ProgramError::IllegalOwner);
    }

    **signer.try_borrow_mut_lamports()? += pda.lamports();
    **pda.try_borrow_mut_lamports()? = 0;

    pda.data.try_borrow_mut()
        .unwrap()
        .fill(0);

    msg!("PDA Closed: {}", pda.key);

    Ok(())
}
