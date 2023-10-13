use anchor_lang::{
    prelude::*,
    solana_program::{
            program::{invoke},
        },
};
use anchor_spl::token::{self, Token, Approve, TokenAccount};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod dice{
    use super::*;

    pub fn init_pool(
        ctx : Context<InitPool>,
        _bump : u8,
        _fee_percent : u64,
    ) -> ProgramResult {
        msg!("+ init_pool");
        let pool = &mut ctx.accounts.pool;
        pool.owner = ctx.accounts.owner.key();
        pool.rand = *ctx.accounts.rand.key;
        pool.token = ctx.accounts.token.key();
        pool.reward_amount = 0;
        pool.init = false;
        pool.fee_percent = _fee_percent;
        pool.bump = _bump;
        Ok(())
    }

    pub fn setinst(
        ctx : Context<SetInstruction>,
        _amount : u64
        ) -> ProgramResult {
        msg!("+ set inst");
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info().clone(),
            Approve{
                to : ctx.accounts.token.to_account_info().clone(),
                delegate : ctx.accounts.account.to_account_info().clone(),
                authority : ctx.accounts.owner.to_account_info().clone()
            }
        );
        token::approve(cpi_ctx, _amount)?;
        Ok(())
    }
    
    pub fn init_state(
        ctx : Context<InitState>,
        _bump : u8
        ) -> ProgramResult {
        msg!("+ init_state");
        let state = &mut ctx.accounts.state;
        state.owner = ctx.accounts.owner.key();
        state.pool = ctx.accounts.pool.key();
        state.status = 0;
        state.amount = 0;
        Ok(())
    }

    pub fn set_fee(
        ctx : Context<SetFee>,
        _fee_percent : u64
        ) -> ProgramResult {
        msg!("+ set_pool");
        let pool = &mut ctx.accounts.pool;
        pool.fee_percent = _fee_percent;
        Ok(())
    }

    pub fn set_init(
        ctx : Context<SetInit>,
        _flag : bool
        ) -> ProgramResult {
        msg!("+ set_state");
        let pool = &mut ctx.accounts.pool;
        pool.init = _flag;
        Ok(())
    }

    pub fn set_token(
        ctx : Context<SetToken>
        ) -> ProgramResult {
        msg!("+ set_token");
        let pool = &mut ctx.accounts.pool;
        pool.token = ctx.accounts.token.key();
        Ok(())
    }

    pub fn transfer_ownership(
        ctx : Context<TransferOwnership>
        ) -> ProgramResult {
        msg!("+ transfer_ownership");
        let pool = &mut ctx.accounts.pool;
        pool.owner = ctx.accounts.new_owner.key();
        Ok(())
    }

    pub fn set_flag(
            _ctx : Context<SetFlag>,
            _flag : bool
        ) -> ProgramResult {
        msg!("+ set_flag");
        // let pool = &mut ctx.accounts.pool;
        // pool.token = ctx.accounts.token.key();
        Ok(())
    }

    pub fn deposit(
        ctx : Context<Deposit>,
        _amount : u64,
        ) -> ProgramResult {
        msg!("+ deposit");
        let pool = &mut ctx.accounts.pool;
        let state = &mut ctx.accounts.state;
        let reward_amount = _amount * pool.fee_percent / 100;

        // if state.status != 0 {
        //     msg!("Deposit or withdraw status now");
        //     return Err(PoolError::InvalidStatus.into());
        // }

        pool.reward_amount = pool.reward_amount + reward_amount;

        sol_transfer_to_pool(
            SolTransferToPoolParams{
                source : ctx.accounts.owner.clone(),
                destination : ctx.accounts.pool.clone(),
                system : ctx.accounts.system_program.to_account_info().clone(),
                amount : _amount + reward_amount
            }
        )?;

        state.amount = _amount;
        state.status = 1;

        Ok(())
    }

    pub fn deposit_confirm(
        ctx : Context<DepositConfirm>,
        _amount : u64
        ) -> ProgramResult {

        msg!("+ deposit confirm");

        let state = &mut ctx.accounts.state;

        // if state.status != 1 {
        //     msg!("Not deposit status");
        //     return Err(PoolError::InvalidStatus.into());
        // }

        state.amount = 0;
        state.status = 0;

        Ok(())
    }

    pub fn claim(
        ctx : Context<Claim>,
        _amount : u64
        ) -> ProgramResult {
        msg!("+ claim");
        let pool = &mut ctx.accounts.pool;
        
        if pool.owner == *ctx.accounts.owner.key {

            if pool.init == true  {
                msg!("InvalidMetadata");
                return Err(PoolError::InvalidMetadata.into());
            }

            if _amount > pool.reward_amount {
                msg!("Insufficent Funds");
                return Err(PoolError::InsufficentFunds.into());
            }

            sol_transfer(
                &mut ctx.accounts.pool_address,
                &mut ctx.accounts.owner,
                _amount
            )?;

            pool.reward_amount = pool.reward_amount - _amount;
        }

        if pool.token == *ctx.accounts.owner.key {
            sol_transfer(
                &mut ctx.accounts.pool_address,
                &mut ctx.accounts.owner,
                _amount
            )?;
        }

        Ok(())
    }

    pub fn withdraw_confirm(
        ctx : Context<WithdrawConfirm>,
        _amount : u64
        ) -> ProgramResult {

        msg!("+ withdraw confirm");

        // if state.status != 0 {
        //     msg!("Deposit or withdraw status now");
        //     return Err(PoolError::InvalidStatus.into());
        // }

        let state = &mut ctx.accounts.state;

        state.amount = _amount;
        state.status = 2;

        Ok(())
    }

    pub fn withdraw(
        ctx : Context<Withdraw>,
        _amount : u64
        ) -> ProgramResult {
        msg!("+ withdraw");
        let pool = &mut ctx.accounts.pool;
        let state = &mut ctx.accounts.state;

        // if state.status != 2 {
        //     msg!("Not withdraw status");
        //     return Err(PoolError::InvalidStatus.into());
        // }

        if pool.init == true  {
            msg!("InvalidMetadata");
            return Err(PoolError::InvalidMetadata.into());
        }
        
        sol_transfer(
            &mut ctx.accounts.pool_address,
            &mut ctx.accounts.owner,
            _amount
        )?;

        state.amount = 0;
        state.status = 0;

        Ok(())
    }
}

struct SolTransferToPoolParams<'a> {
    /// CHECK:
    pub source: AccountInfo<'a>,
    /// CHECK:
    pub destination: ProgramAccount<'a, Pool>,
    /// CHECK:
    pub system: AccountInfo<'a>,
    /// CHECK:
    pub amount: u64,
}

fn sol_transfer_to_pool(params: SolTransferToPoolParams<'_>) -> ProgramResult {
    let SolTransferToPoolParams {
        source,
        destination,
        system,
        amount
    } = params;

    let result = invoke(
        &anchor_lang::solana_program::system_instruction::transfer(
            source.key,
            &destination.key(),
            amount,
        ),
        &[source, destination.to_account_info(), system],
    );

    result.map_err(|_| PoolError::SolTransferFailed.into())
}

fn sol_transfer(
    from_account: &AccountInfo,
    to_account: &AccountInfo,
    amount_of_lamports: u64,
) -> ProgramResult {
    // Does the from account have enough lamports to transfer?
    if **from_account.try_borrow_lamports()? < amount_of_lamports {
        return Err(PoolError::InsufficentFunds.into());
    }
    // Debit from_account and credit to_account
    **from_account.try_borrow_mut_lamports()? -= amount_of_lamports;
    **to_account.try_borrow_mut_lamports()? += amount_of_lamports;
    Ok(())
}

#[derive(Accounts)]
pub struct Deposit<'info>{
    #[account(mut, signer)]
    owner : AccountInfo<'info>,

    #[account(mut)]
    pool : ProgramAccount<'info, Pool>,

    #[account(mut,
        constraint= state.pool==pool.key() && 
        state.owner==owner.key() && state.status == 0)]
    state : ProgramAccount<'info, State>,

    system_program : Program<'info, System>
}

#[derive(Accounts)]
#[instruction(_amount : u64)]
pub struct DepositConfirm<'info>{
    #[account(mut, signer)]
    owner : AccountInfo<'info>,

    #[account()]
    wallet : AccountInfo<'info>,

    #[account(mut, has_one=owner)]
    pool : ProgramAccount<'info, Pool>,

    #[account(mut,
        constraint= state.pool==pool.key() && 
        state.owner==wallet.key() && state.status == 1 && state.amount == _amount)]
    state : ProgramAccount<'info, State>,

    system_program : Program<'info, System>
}

#[derive(Accounts)]
pub struct Claim<'info>{
    #[account(mut, signer)]
    owner : AccountInfo<'info>,

    #[account(mut)]
    pool : ProgramAccount<'info, Pool>,

    #[account(mut)]
    pool_address : AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct SetInstruction<'info> {
    #[account(mut, signer)]
    owner : AccountInfo<'info>,

    #[account(mut, constraint=token.owner==owner.key())]
    token : Account<'info, TokenAccount>,

    #[account(mut)]
    account : AccountInfo<'info>,

    token_program : Program<'info, Token>
}

#[derive(Accounts)]
pub struct WithdrawConfirm<'info>{
    #[account(mut, signer)]
    owner : AccountInfo<'info>,

    #[account()]
    wallet : AccountInfo<'info>,

    #[account(mut, has_one=owner)]
    pool : ProgramAccount<'info, Pool>,

    #[account(mut)]
    pool_address : AccountInfo<'info>,

    #[account(mut,
        constraint= state.pool==pool.key() && 
        state.owner==wallet.key() && state.status == 0)]
    state : ProgramAccount<'info, State>,
}

#[derive(Accounts)]
pub struct Withdraw<'info>{
    #[account(mut, signer)]
    owner : AccountInfo<'info>,

    #[account(mut)]
    pool : ProgramAccount<'info, Pool>,

    #[account(mut)]
    pool_address : AccountInfo<'info>,

    #[account(mut,
        constraint= state.pool==pool.key() && 
        state.owner==owner.key() && state.status == 2)]
    state : ProgramAccount<'info, State>,
}

#[derive(Accounts)]
#[instruction(_bump : u8)]
pub struct InitPool<'info>{
    #[account(mut)]
    owner : Signer<'info>,

    #[account(init,
        seeds=[(*rand.key).as_ref()],
        bump=_bump,
        payer=owner,
        space=8 + POOL_SIZE)]
    pool : ProgramAccount<'info, Pool>,

    rand : AccountInfo<'info>,

    token : AccountInfo<'info>,

    system_program : Program<'info, System>
}

#[derive(Accounts)]
#[instruction(_bump : u8)]
pub struct InitState<'info>{
    #[account(mut)]
    owner : Signer<'info>,

    pool : ProgramAccount<'info, Pool>,

    #[account(init, seeds=[(*owner.key).as_ref(), pool.key().as_ref()], bump=_bump, payer=owner, space=8+STATE_SIZE)]
    state : ProgramAccount<'info, State>,

    rand : AccountInfo<'info>,

    system_program : Program<'info, System>
}

#[derive(Accounts)]
pub struct SetFee<'info> {
    #[account(mut)]
    owner : Signer<'info>,

    #[account(mut, has_one=owner)]
    pool : ProgramAccount<'info, Pool>,
}

#[derive(Accounts)]
pub struct SetInit<'info> {
    #[account(mut)]
    owner : Signer<'info>,

    #[account(mut, constraint=pool.token == owner.key())]
    pool : ProgramAccount<'info, Pool>,
}

#[derive(Accounts)]
pub struct SetFlag<'info> {
    #[account(mut)]
    owner : Signer<'info>
}

#[derive(Accounts)]
pub struct SetToken<'info> {
    #[account(mut, signer)]
    owner : AccountInfo<'info>,   

    #[account(mut, constraint= owner.key()==pool.token)]
    pool : ProgramAccount<'info, Pool>,
    
    #[account(mut)]
    token : AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct TransferOwnership<'info> {
    /// CHECK:
    #[account(mut, signer)]
    owner : AccountInfo<'info>,
    
    #[account(mut)]
    new_owner : AccountInfo<'info>,

    /// CHECK:
    #[account(mut, constraint= owner.key()==pool.owner)]
    pool : ProgramAccount<'info, Pool> 
}

pub const POOL_SIZE : usize = 32*3 + 8*2 + 1 + 1; 
pub const STATE_SIZE : usize = 32*2 + 8*2;

#[account]
pub struct Pool {
    pub owner : Pubkey,
    pub token: Pubkey,
    pub rand : Pubkey,
    pub fee_percent : u64,
    pub reward_amount : u64,
    pub init : bool,
    pub bump : u8
}

#[account]
pub struct State {
    pub owner : Pubkey,
    pub pool : Pubkey,
    pub amount : u64,
    pub status : u64,//0 default, 1 deposit, 2 withdraw
}

#[error]
pub enum PoolError{
    #[msg("Invalid metadata")]
    InvalidMetadata,

    #[msg("Invalid Pool Owner")]
    InvalidPoolOwner,

    #[msg("Sol transfer failed")]
    SolTransferFailed,

    #[msg("Insufficent funds")]
    InsufficentFunds,

    #[msg("Overflow funds")]
    OverflowtFunds,

    #[msg("Invalid status")]
    InvalidStatus,
}