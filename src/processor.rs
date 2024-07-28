//! Program state processor

use {

    borsh::{BorshDeserialize, BorshSerialize}, solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult, instruction::{AccountMeta, Instruction}, 
        msg, program::{get_return_data,  invoke_signed}, program_error::ProgramError, pubkey::Pubkey, rent::Rent, 
        system_instruction
    }, spl_tlv_account_resolution::{account::ExtraAccountMeta, state::ExtraAccountMetaList}, spl_token_2022::{
        extension::{
            transfer_hook::TransferHookAccount, BaseStateWithExtensions, StateWithExtensions,
        },
        state::{Account, Mint},
    }, spl_transfer_hook_interface::{
        collect_extra_account_metas_signer_seeds,
        error::TransferHookError,
        get_extra_account_metas_address, get_extra_account_metas_address_and_bump_seed,
        instruction::{ExecuteInstruction, TransferHookInstruction}
    }
};


#[derive(BorshDeserialize)]
/// Random number
pub struct RandomNumber {
    /// random number
    pub random_number: u64,
}

#[derive(BorshDeserialize, BorshSerialize)]
/// Data account
pub struct Counter {
    /// total calls
    pub total_calls: u64,
}



fn check_token_account_is_transferring(account_info: &AccountInfo) -> Result<(), ProgramError> {
    let account_data = account_info.try_borrow_data()?;
    let token_account = StateWithExtensions::<Account>::unpack(&account_data)?;
    let extension = token_account.get_extension::<TransferHookAccount>()?;
    if bool::from(extension.transferring) {
        Ok(())
    } else {
        Err(TransferHookError::ProgramCalledOutsideOfTransfer.into())
    }
}

/// Processes an [Execute](enum.TransferHookInstruction.html) instruction.
pub fn process_execute(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    msg!("instruction data valid");

    let accounts_iter: &mut std::slice::Iter<AccountInfo> = &mut accounts.iter();

    let source_account_info: &AccountInfo = next_account_info(accounts_iter)?; //0
    let mint_info: &AccountInfo = next_account_info(accounts_iter)?; //1
    let destination_account_info: &AccountInfo = next_account_info(accounts_iter)?; //2
    let authority_info: &AccountInfo = next_account_info(accounts_iter)?; //3
    let extra_account_metas_info: &AccountInfo = next_account_info(accounts_iter)?; //4
    let hook_counter_info: &AccountInfo<'_> = next_account_info(accounts_iter)?; //specific to program. derived from seeds. Stores how many times at a given time 5
    let payer_pda: &AccountInfo<'_> = next_account_info(accounts_iter)?; //specific to user derived from mint(1) and authority(3) 6
    let price_feed_account_1: &AccountInfo<'_> = next_account_info(accounts_iter)?; //7
    let price_feed_account_2: &AccountInfo<'_> = next_account_info(accounts_iter)?; //8
    let price_feed_account_3: &AccountInfo<'_> = next_account_info(accounts_iter)?; //9
    let fallback_account: &AccountInfo<'_> = next_account_info(accounts_iter)?; //10
    let current_feed_accounts: &AccountInfo<'_> = next_account_info(accounts_iter)?; //11
    let temp: &AccountInfo<'_> = next_account_info(accounts_iter)?; //derived from mint(1) and authority_info(3) and hook_counter_info account data(5).(test if you can create as an account belonging to other program)
    let rng_program: &AccountInfo<'_> = next_account_info(accounts_iter)?; //13
    let system_program: &AccountInfo<'_> = next_account_info(accounts_iter)?; //14
    let compute_budget_program: &AccountInfo<'_> = next_account_info(accounts_iter)?; //14
    
    msg!("accounts iterated");

    
    //this mothafukas spent waay too much compute unit!!!
    //Check that the accounts are properly in "transferring" mode
    check_token_account_is_transferring(source_account_info)?;
    check_token_account_is_transferring(destination_account_info)?;

    msg!("accounts are transferring");
    


    // Only check that the correct pda and account are provided
    let expected_validation_address: Pubkey =
        get_extra_account_metas_address(mint_info.key, program_id);
    if expected_validation_address != *extra_account_metas_info.key {
        return Err(ProgramError::InvalidSeeds);
    }

    msg!("expected_validation_address provided");
    


    // Load the extra required accounts from the validation account
    let data = extra_account_metas_info.try_borrow_data()?;

    msg!("Load the extra required accounts");
    


    // Check the provided accounts against the validation data
    ExtraAccountMetaList::check_account_infos::<ExecuteInstruction>(
        accounts,
        &TransferHookInstruction::Execute { amount }.pack(),
        program_id,
        &data,
    )?;

    msg!("Check the provided accounts against the validation data");
    


    let zero:u64 = 0;
    let account_balance: u64 = **payer_pda.lamports.borrow();

    if account_balance != zero {
        msg!("Payer exists");
        

        let fee: u64 = u64::from_le_bytes(data[1..9].try_into().unwrap());
        let rent: Rent = Rent::default();
        let temp_rent_amount: u64 = rent.minimum_balance(50);
        let payer_rent_amount: u64 = rent.minimum_balance(8);

        let rents: u64 = temp_rent_amount
            .checked_add(payer_rent_amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        let total_required_balance: u64 = rents
            .checked_add(fee)
            .ok_or(ProgramError::ArithmeticOverflow)?;


        if total_required_balance > account_balance {
            msg!("Payer has enough lamports");

            let counter: Counter = Counter::try_from_slice(&hook_counter_info.data.borrow())?;
        
            msg!("payer_data deserialized");
            

            get_random_number(
                hook_counter_info,
                payer_pda,
                price_feed_account_1,
                price_feed_account_2,
                price_feed_account_3,
                fallback_account,
                current_feed_accounts,
                temp,
                rng_program,
                system_program,
                compute_budget_program,
                mint_info.key,
                authority_info.key,
                program_id,
                counter
            )?;
        }
    }



    Ok(())
}

fn get_random_number<'info>(
    hook_counter_info: &AccountInfo<'info>,
    payer_pda: &AccountInfo<'info>,
    price_feed_account_1: &AccountInfo<'info>,
    price_feed_account_2: &AccountInfo<'info>,
    price_feed_account_3: &AccountInfo<'info>,
    fallback_account: &AccountInfo<'info>,
    current_feed_accounts: &AccountInfo<'info>,
    temp: &AccountInfo<'info>,
    rng_program: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
    compute_budget_program: &AccountInfo<'info>,
    mint: &Pubkey,
    authority: &Pubkey,
    program_id: &Pubkey,
    mut counter:Counter
) -> ProgramResult {

    

    let (payer_address, payer_bump) =
        Pubkey::find_program_address(&[&mint.to_bytes(), &authority.to_bytes()], program_id);
    let (temp_address, temp_bump) =
        Pubkey::find_program_address(&[&mint.to_bytes(), &authority.to_bytes(), &counter.total_calls.to_le_bytes()], program_id);
 
    msg!("pda addresses derived");
    

    let payer_meta: AccountMeta = AccountMeta {
        pubkey: payer_address,
        is_signer: true,
        is_writable: true,
    };    
    let payer_meta2: AccountMeta = AccountMeta {
        pubkey: payer_address,
        is_signer: true,
        is_writable: true,
    };
    let price_feed_account_1_meta: AccountMeta = AccountMeta {
        pubkey: *price_feed_account_1.key,
        is_signer: false,
        is_writable: false,
    };
    let price_feed_account_2_meta: AccountMeta = AccountMeta {
        pubkey: *price_feed_account_2.key,
        is_signer: false,
        is_writable: false,
    };
    let price_feed_account_3_meta: AccountMeta = AccountMeta {
        pubkey: *price_feed_account_3.key,
        is_signer: false,
        is_writable: false,
    };
    let fallback_account_meta: AccountMeta = AccountMeta {
        pubkey: *fallback_account.key,
        is_signer: false,
        is_writable: false,
    };
    let current_feed_accounts_meta: AccountMeta = AccountMeta {
        pubkey: *current_feed_accounts.key,
        is_signer: false,
        is_writable: true,
    };
    let temp_meta: AccountMeta = AccountMeta {
        pubkey: temp_address,
        is_signer: true,
        is_writable: true,
    };
    let system_program_meta: AccountMeta = AccountMeta {
        pubkey: *system_program.key,
        is_signer: false,
        is_writable: false,
    };

    // Creating instruction to cpi RNG PROGRAM
    let ix: Instruction = Instruction {
        program_id: *rng_program.key,
        accounts: [
            payer_meta,
            price_feed_account_1_meta,
            price_feed_account_2_meta,
            price_feed_account_3_meta,
            fallback_account_meta,
            current_feed_accounts_meta,
            temp_meta,
            system_program_meta,
        ]
        .to_vec(),
        data: [0].to_vec(),
    };

    let ix2: Instruction = Instruction {
        program_id: *compute_budget_program.key,
        accounts: [
            payer_meta2
        ]
        .to_vec(),
        data: [2, 32, 161, 7, 0].to_vec(),
    };

    msg!("instruction screated");

    invoke_signed(
        &ix2,
        &[
            payer_pda.clone(),
        ],
        &[
            &[&mint.to_bytes(), &authority.to_bytes(), &[payer_bump]],
        ],
    )?;
    

    invoke_signed(
        &ix,
        &[
            payer_pda.clone(),
            temp.clone(),
            price_feed_account_1.clone(),
            price_feed_account_2.clone(),
            price_feed_account_3.clone(),
            fallback_account.clone(),
            current_feed_accounts.clone(),
            temp.clone(),
            system_program.clone(),
        ],
        &[
            &[&mint.to_bytes(), &authority.to_bytes(), &[payer_bump]],
            &[&mint.to_bytes(), &authority.to_bytes(), &counter.total_calls.to_le_bytes(), &[temp_bump]],
        ],
    )?;

    msg!("rng program CPI");
    

    let returned_data: (Pubkey, Vec<u8>) = get_return_data().unwrap();

    // Random number is returned from the RNG_PROGRAM
    let random_number: RandomNumber;
    if &returned_data.0 == rng_program.key {
        random_number = RandomNumber::try_from_slice(&returned_data.1)?;
        msg!("{}", random_number.random_number);
    

    } else {
        panic!();
    }


    counter.total_calls = counter
        .total_calls
        .checked_add(1)
        .ok_or(TransferHookError::IncorrectAccount)?;

    counter.serialize(&mut &mut hook_counter_info.data.borrow_mut()[..])?;

    msg!("payer_data serialized");
    


    Ok(())
}


/// Processes a
/// [InitializeExtraAccountMetaList](enum.TransferHookInstruction.html)
/// instruction.
pub fn process_initialize_extra_account_meta_list(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    extra_account_metas: &[ExtraAccountMeta],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let extra_account_metas_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let authority_info = next_account_info(account_info_iter)?;
    let _system_program_info = next_account_info(account_info_iter)?;

    // check that the one mint we want to target is trying to create extra
    // account metas
    #[cfg(feature = "forbid-additional-mints")]
    if *mint_info.key != crate::mint::id() {
        return Err(ProgramError::InvalidArgument);
    }

    // check that the mint authority is valid without fully deserializing
    let mint_data = mint_info.try_borrow_data()?;
    let mint = StateWithExtensions::<Mint>::unpack(&mint_data)?;
    let mint_authority = mint
        .base
        .mint_authority
        .ok_or(TransferHookError::MintHasNoMintAuthority)?;

    // Check signers
    if !authority_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if *authority_info.key != mint_authority {
        return Err(TransferHookError::IncorrectMintAuthority.into());
    }

    // Check validation account
    let (expected_validation_address, bump_seed) =
        get_extra_account_metas_address_and_bump_seed(mint_info.key, program_id);
    if expected_validation_address != *extra_account_metas_info.key {
        return Err(ProgramError::InvalidSeeds);
    }

    // Create the account
    let bump_seed = [bump_seed];
    let signer_seeds = collect_extra_account_metas_signer_seeds(mint_info.key, &bump_seed);
    let length = extra_account_metas.len();
    let account_size = ExtraAccountMetaList::size_of(length)?;
    invoke_signed(
        &system_instruction::allocate(extra_account_metas_info.key, account_size as u64),
        &[extra_account_metas_info.clone()],
        &[&signer_seeds],
    )?;
    invoke_signed(
        &system_instruction::assign(extra_account_metas_info.key, program_id),
        &[extra_account_metas_info.clone()],
        &[&signer_seeds],
    )?;

    // Write the data
    let mut data = extra_account_metas_info.try_borrow_mut_data()?;
    ExtraAccountMetaList::init::<ExecuteInstruction>(&mut data, extra_account_metas)?;

    Ok(())
}

/// Processes a
/// [UpdateExtraAccountMetaList](enum.TransferHookInstruction.html)
/// instruction.
pub fn process_update_extra_account_meta_list(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    extra_account_metas: &[ExtraAccountMeta],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let extra_account_metas_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let authority_info = next_account_info(account_info_iter)?;

    // check that the mint authority is valid without fully deserializing
    let mint_data = mint_info.try_borrow_data()?;
    let mint = StateWithExtensions::<Mint>::unpack(&mint_data)?;
    let mint_authority = mint
        .base
        .mint_authority
        .ok_or(TransferHookError::MintHasNoMintAuthority)?;

    // Check signers
    if !authority_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if *authority_info.key != mint_authority {
        return Err(TransferHookError::IncorrectMintAuthority.into());
    }

    // Check validation account
    let expected_validation_address = get_extra_account_metas_address(mint_info.key, program_id);
    if expected_validation_address != *extra_account_metas_info.key {
        return Err(ProgramError::InvalidSeeds);
    }

    // Check if the extra metas have been initialized
    let min_account_size = ExtraAccountMetaList::size_of(0)?;
    let original_account_size = extra_account_metas_info.data_len();
    if program_id != extra_account_metas_info.owner || original_account_size < min_account_size {
        return Err(ProgramError::UninitializedAccount);
    }

    // If the new extra_account_metas length is different, resize the account and
    // update
    let length = extra_account_metas.len();
    let account_size = ExtraAccountMetaList::size_of(length)?;
    if account_size >= original_account_size {
        extra_account_metas_info.realloc(account_size, false)?;
        let mut data = extra_account_metas_info.try_borrow_mut_data()?;
        ExtraAccountMetaList::update::<ExecuteInstruction>(&mut data, extra_account_metas)?;
    } else {
        {
            let mut data = extra_account_metas_info.try_borrow_mut_data()?;
            ExtraAccountMetaList::update::<ExecuteInstruction>(&mut data, extra_account_metas)?;
        }
        extra_account_metas_info.realloc(account_size, false)?;
    }

    Ok(())
}



/// Processes an [Instruction](enum.Instruction.html).
pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
    let instruction = TransferHookInstruction::unpack(input)?;

    match instruction {
        TransferHookInstruction::Execute { amount } => {
            msg!("Instruction: Execute");
            process_execute(program_id, accounts, amount)
        }
        TransferHookInstruction::InitializeExtraAccountMetaList {
            extra_account_metas,
        } => {
            msg!("Instruction: InitializeExtraAccountMetaList");
            process_initialize_extra_account_meta_list(program_id, accounts, &extra_account_metas)
        }
        TransferHookInstruction::UpdateExtraAccountMetaList {
            extra_account_metas,
        } => {
            msg!("Instruction: UpdateExtraAccountMetaList");
            process_update_extra_account_meta_list(program_id, accounts, &extra_account_metas)
        }

    }
}

/*

//call this once after initializing the hook
async function create_hook_counter() {

  const hook_program = new PublicKey("54GNE9AuT5juYGVbYBTMakgo1ACgf65sZaCq32AVSHSj");

  const newAccount = Keypair.generate()

  const ix = SystemProgram.createAccount({
    fromPubkey:payer.publicKey,
    newAccountPubkey:newAccount.publicKey,
    space:8,
    lamports:LAMPORTS_PER_SOL*0.01,
    programId:hook_program
  })

  const message = new TransactionMessage({
    instructions: [ix],
      payerKey: payer.publicKey,
      recentBlockhash : (await connection.getLatestBlockhash()).blockhash
    }).compileToV0Message();

    const tx = new VersionedTransaction(message);
    tx.sign([payer,newAccount]);

    console.log(newAccount.publicKey.toBase58())

  const sig = await connection.sendTransaction(tx);


}

//users need to add lamports to this account for keeping calling the rng
async function get_user_account() {

  const mint = new PublicKey("f4xD9KagBKJfJM8f8WFj6pz2E2YJUoYLggac1Kg7Cc5");
  const authority = new PublicKey("Frfz5jf4mR7QFNqrYKAMKCjRbCGycX1by6r26UmHHCoL");

  const hook_program = new PublicKey("54GNE9AuT5juYGVbYBTMakgo1ACgf65sZaCq32AVSHSj");
  const current_feed = PublicKey.findProgramAddressSync([mint.toBytes(),authority.toBytes()],hook_program);

  console.log(current_feed[0].toBase58())

}

*/
//100000
//  5000