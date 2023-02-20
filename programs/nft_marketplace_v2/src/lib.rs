use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount, Transfer},
};

declare_id!("29hLYT9LFzKpf98UHqWU4zxm91tWfimkcPq7uNYcdVsR");
mod error;

#[program]
pub mod nft_marketplace_v2 {
    use super::*;

    pub fn init_state(ctx: Context<InitState>, _owner_cut: u16) -> Result<()> {
        let program_state = &mut ctx.accounts.state_account;
        if program_state.initialized == true {
            return Err(error::Error::StateAlreadyInitialized.into());
        }
        require!(_owner_cut < 10000, error::Error::InvalidOwnerCut);

        program_state.owner = ctx.accounts.user.key();
        program_state.owner_cut = _owner_cut;

        Ok(())
    }

    pub fn create_listing(
        ctx: Context<CreateListing>,
        next_id: String,
        _starting_price: u64,
        _token_mint: Pubkey,
        _end_at: i64,
        _started_at: i64,
        _is_auction: bool,
    ) -> Result<()> {
        ctx.accounts.listing_account.seller = ctx.accounts.user.key();
        ctx.accounts.listing_account.starting_price = _starting_price;
        ctx.accounts.listing_account.token_mint = _token_mint;
        ctx.accounts.listing_account.end_at = _end_at;
        ctx.accounts.listing_account.started_at = _started_at;
        ctx.accounts.listing_account.highest_bidder = None;
        ctx.accounts.listing_account.highest_price = 0;
        ctx.accounts.listing_account.cancel = false;
        ctx.accounts.listing_account.is_auction = _is_auction;

        // TRANSFER NFT //
        let transfer_instruction = Transfer {
            from: ctx.accounts.from.to_account_info(),
            to: ctx.accounts.to.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, transfer_instruction);
        anchor_spl::token::transfer(cpi_ctx, 1)?;

        Ok(())
    }

    pub fn bid(ctx: Context<Bid>, current_id: String, _price: u64) -> Result<()> {
        let listing = &ctx.accounts.listing_account;
        require!(listing.is_auction == true, error::Error::NotAuction);
        require!(
            Clock::get()?.unix_timestamp < listing.end_at
                && Clock::get()?.unix_timestamp > listing.started_at,
            error::Error::ListingNotOn
        );
        require!(_price > listing.highest_price, error::Error::InvalidPrice);
        require!(listing.cancel == false, error::Error::AuctionCanceled);
        require!(
            !ctx.accounts
                .user
                .key()
                .eq(&ctx.accounts.listing_account.seller.key()),
            error::Error::InvalidBid
        );

        ctx.accounts.listing_account.highest_price = _price;
        ctx.accounts.listing_account.highest_bidder = Some(ctx.accounts.user.key());

        Ok(())
    }

    pub fn cancel_listing(
        ctx: Context<CancelListing>,
        current_id: String,
        /*  _item_id: u128, */
        _bump: u8,
    ) -> Result<()> {
        let listing = &ctx.accounts.listing_account;
        // require!(
        //     Clock::get()?
        //         .unix_timestamp
        //         .saturating_sub(listing.started_at)
        //         < listing.duration,
        //     error::Error::ListingNotOn
        // );
        require!(
            ctx.accounts.user.key().eq(&listing.seller),
            error::Error::NotAuthorized
        );

        // transfer back
        let seeds = vec![_bump];
        let seeds = vec![
            ctx.accounts.user.key.as_ref(),
            current_id.as_ref(),
            seeds.as_slice(),
        ];
        let seeds = vec![seeds.as_slice()];
        let seeds = seeds.as_slice();
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::Transfer {
                from: ctx.accounts.from.to_account_info(),
                to: ctx.accounts.to.to_account_info(),
                authority: ctx.accounts.auth.to_account_info(),
            },
            &seeds,
        );
        anchor_spl::token::transfer(cpi_ctx, 1)?;

        ctx.accounts.listing_account.cancel = true;

        Ok(())
    }

    pub fn purchase_nft(
        ctx: Context<Purchase>,
        current_id: String,
        /*  _item_id: u128, */
        _bump: u8,
    ) -> Result<()> {
        let listing = &ctx.accounts.listing_account;
        require!(
            Clock::get()?.unix_timestamp > listing.end_at,
            error::Error::AuctionOn
        );
        require!(
            listing.highest_bidder.unwrap().eq(&ctx.accounts.user.key()),
            error::Error::NotWinner
        );
        require!(listing.is_auction == true, error::Error::NotAuction);

        // transfer nft
        let seeds = vec![_bump];
        let seeds = vec![
            ctx.accounts.seller.key.as_ref(),
            current_id.as_ref(),
            seeds.as_slice(),
        ];
        let seeds = vec![seeds.as_slice()];
        let seeds = seeds.as_slice();

        // create ata for receiver
        anchor_spl::associated_token::create(CpiContext::new(
            ctx.accounts.associated_token_program.to_account_info(),
            anchor_spl::associated_token::Create {
                payer: ctx.accounts.user.to_account_info(),
                associated_token: ctx.accounts.to_token_account.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
            },
        ))?;

        // transfer nft
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::Transfer {
                from: ctx.accounts.from_token_account.to_account_info(),
                to: ctx.accounts.to_token_account.to_account_info(),
                authority: ctx.accounts.auth.to_account_info(),
            },
            &seeds,
        );
        anchor_spl::token::transfer(cpi_ctx, 1)?;

        let owner_cut = ctx
            .accounts
            .listing_account
            .highest_price
            .saturating_mul(ctx.accounts.state_account.owner_cut.into())
            .saturating_div(10000);

        // transfer sol
        anchor_lang::solana_program::program::invoke(
            &anchor_lang::solana_program::system_instruction::transfer(
                &ctx.accounts.user.key(),
                &ctx.accounts.seller.key(),
                listing.highest_price.saturating_sub(owner_cut),
            ),
            &[
                ctx.accounts.user.to_account_info(),
                ctx.accounts.seller.to_account_info(),
            ],
        )?;

        anchor_lang::solana_program::program::invoke(
            &anchor_lang::solana_program::system_instruction::transfer(
                &ctx.accounts.user.key(),
                &ctx.accounts.owner.key(),
                owner_cut,
            ),
            &[
                ctx.accounts.user.to_account_info(),
                ctx.accounts.owner.to_account_info(),
            ],
        )?;

        ctx.accounts.listing_account.cancel = true;
        Ok(())
    }

    pub fn buy_nft(
        ctx: Context<Buy>,
        current_id: String,
        /*  _item_id: u128, */ _bump: u8,
    ) -> Result<()> {
        let listing = &ctx.accounts.listing_account;
        require!(listing.is_auction == false, error::Error::NotOnSell);
        require!(
            listing.seller != ctx.accounts.user.key(),
            error::Error::NotAuthorized
        );

        // create ata for receiver
        let seeds = vec![_bump];
        let seeds = vec![
            ctx.accounts.seller.key.as_ref(),
            current_id.as_ref(),
            seeds.as_slice(),
        ];
        let seeds = vec![seeds.as_slice()];
        let seeds = seeds.as_slice();
        anchor_spl::associated_token::create(CpiContext::new(
            ctx.accounts.associated_token_program.to_account_info(),
            anchor_spl::associated_token::Create {
                payer: ctx.accounts.user.to_account_info(),
                associated_token: ctx.accounts.to_token_account.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
            },
        ))?;

        // transfer nft
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::Transfer {
                from: ctx.accounts.from_token_account.to_account_info(),
                to: ctx.accounts.to_token_account.to_account_info(),
                authority: ctx.accounts.auth.to_account_info(),
            },
            &seeds,
        );
        anchor_spl::token::transfer(cpi_ctx, 1)?;

        let owner_cut = ctx
            .accounts
            .listing_account
            .starting_price
            .saturating_mul(ctx.accounts.state_account.owner_cut.into())
            .saturating_div(10000);

        // transfer sol
        anchor_lang::solana_program::program::invoke(
            &anchor_lang::solana_program::system_instruction::transfer(
                &ctx.accounts.user.key(),
                &ctx.accounts.seller.key(),
                listing.starting_price.saturating_sub(owner_cut),
            ),
            &[
                ctx.accounts.user.to_account_info(),
                ctx.accounts.seller.to_account_info(),
            ],
        )?;

        anchor_lang::solana_program::program::invoke(
            &anchor_lang::solana_program::system_instruction::transfer(
                &ctx.accounts.user.key(),
                &ctx.accounts.owner.key(),
                owner_cut,
            ),
            &[
                ctx.accounts.user.to_account_info(),
                ctx.accounts.owner.to_account_info(),
            ],
        )?;

        ctx.accounts.listing_account.cancel = true;

        Ok(())
    }

    pub fn set_price(
        ctx: Context<SetPrice>,
        current_id: String,
        /*  _item_id: u128, */
        _price: u64,
    ) -> Result<()> {
        let listing = &ctx.accounts.listing_account;
        require!(
            ctx.accounts.user.key().eq(&listing.seller),
            error::Error::NotAuthorized
        );
        require!(listing.is_auction == false, error::Error::NotOnSell);

        ctx.accounts.listing_account.starting_price = _price;

        Ok(())
    }
}

#[account]
pub struct Listing {
    seller: Pubkey,
    starting_price: u64,
    token_mint: Pubkey, // spl token to pay for nft
    end_at: i64,
    started_at: i64,
    highest_bidder: Option<Pubkey>,
    highest_price: u64,
    cancel: bool,
    is_auction: bool,
}

#[account]
pub struct State {
    pub owner: Pubkey,
    pub initialized: bool,
    pub owner_cut: u16,
}

#[derive(Accounts)]
pub struct InitState<'info> {
    #[account(init, payer = user, space = 10240)]
    pub state_account: Account<'info, State>,

    #[account(mut)]
    pub user: Signer<'info>, // admin

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(next_id: String)]
pub struct CreateListing<'info> {
    #[account(init, seeds = [user.to_account_info().key.as_ref(), next_id.as_bytes()], bump, payer = user, space = 10000)]
    pub listing_account: Account<'info, Listing>,

    #[account(mut)]
    pub user: Signer<'info>, // seller

    /// CHECK:
    #[account(mut)]
    pub to: AccountInfo<'info>,

    /// CHECK:
    #[account(mut)]
    pub from: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(current_id: String)]
pub struct Bid<'info> {
    #[account(mut, seeds = [owner_auction.to_account_info().key.as_ref(), current_id.as_bytes()], bump)]
    pub listing_account: Account<'info, Listing>,

    #[account(mut)]
    pub user: Signer<'info>, // seller

    /// CHECK:
    #[account(mut)]
    pub owner_auction: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(current_id: String)]
pub struct CancelListing<'info> {
    #[account(mut, seeds = [user.to_account_info().key.as_ref(), current_id.as_bytes()], bump)]
    pub listing_account: Account<'info, Listing>,

    #[account(mut)]
    pub user: Signer<'info>, // seller

    /// CHECK:
    #[account(mut)]
    pub to: AccountInfo<'info>,

    /// CHECK:
    #[account(mut)]
    pub from: AccountInfo<'info>,

    /// CHECK: token account authority PDA
    #[account()]
    pub auth: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(current_id: String)]
pub struct SetPrice<'info> {
    #[account(mut, seeds = [user.to_account_info().key.as_ref(), current_id.as_bytes()], bump)]
    pub listing_account: Account<'info, Listing>,

    #[account(mut)]
    pub user: Signer<'info>, // seller

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(current_id: String)]
pub struct Purchase<'info> {
    #[account(mut)]
    pub user: Signer<'info>, // seller

    pub token_program: Program<'info, Token>,

    #[account(mut, seeds = [user.to_account_info().key.as_ref(), current_id.as_bytes()], bump)]
    pub listing_account: Account<'info, Listing>,

    #[account(mut)]
    pub state_account: Account<'info, State>,

    #[account(mut)]
    pub from_token_account: Account<'info, TokenAccount>,

    /// CHECK: create new ata for receiver
    #[account(mut)]
    pub to_token_account: UncheckedAccount<'info>,

    /// CHECK: token account authority PDA
    #[account(
        // seeds = ["auth".as_bytes().as_ref()],
        // bump,
    )]
    pub auth: UncheckedAccount<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    /// CHECK:
    #[account(mut)]
    pub seller: AccountInfo<'info>,

    /// CHECK:
    #[account(mut)]
    pub owner: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(current_id: String)]
pub struct Buy<'info> {
    #[account(mut)]
    pub user: Signer<'info>, // seller

    pub token_program: Program<'info, Token>,

    #[account(mut, seeds = [seller.to_account_info().key.as_ref(), current_id.as_bytes()], bump)]
    pub listing_account: Account<'info, Listing>,

    #[account(mut)]
    pub state_account: Account<'info, State>,

    #[account(mut)]
    pub from_token_account: Account<'info, TokenAccount>,

    /// CHECK
    #[account(mut)]
    pub to_token_account: UncheckedAccount<'info>,

    /// CHECK: token account authority PDA
    #[account(
        // seeds = ["auth".as_bytes().as_ref()],
        // bump,
    )]
    pub auth: UncheckedAccount<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    /// CHECK:
    #[account(mut)]
    pub seller: AccountInfo<'info>,

    /// CHECK:
    #[account(mut)]
    pub owner: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}
