use anchor_lang::prelude::*;
use anchor_lang::system_program;
use solana_program::pubkey::Pubkey;
use anchor_spl::(
    associated_token::AssociatedToken, metadata::Metadata, 
    token_interface::{mint_to, Minto, Mint, TokenAccount, TokenInterface}
);
use switchboard_on_demand::accounts::RandomnessAccountData;

use anchor_spl::metadata{
    create_metadata_account_v3, 
    create_master_editon_v3,
    CreateMetadataAccountv3,
    set_and_verify_sized_collection_item, 
    sign_metadata, 
    CreateMasterEditionV3, 
    SetAndVerifySizedCollectionItem, 
    SignMetadata,
    mpl_token_metadata::types::Creator,
    mpl_token_metadata::types::{CollectionDetails, DataV2}
};

use anchor_spl::metadata::{create_master_editon_v3, mpl_token_metadata::types::{CollectionDetails, DataV2}, set_and_verify_sized_collection_item, sign_metadata, CreateMasterEditionV3, SetAndVerifySizedCollectionItem, SignMetadata};

declare_id!("Cr7YoXn8ejm6rPoFwBMuDe88gdPpmQDMNFQtZXepKr31");

#[constant]
pub const NAME: &str = "Token Lottery Ticket #";
#[constant]
pub const SYMBOL: &str = "TLT";
#[constant]
pub const URI: &str = "https://phasionistar.onrender.com";


#[program]
pub mod token_lottery {
    // use anchor_spl::token::{mint_to, Minto};
    // use anchor_spl::token::MintTo;

    use super::*;

    pub fn initialize_config(_ctx: Context<Initialize>, start: u64, end: u64, price: u64) -> Result<()> {
        ctx.accounts.token_lottery.bump = ctx.bumps.token_lottery;
        ctx.accounts.token_lottery.start_time = start;
        ctx.accounts.token_lottery.end_time = end;
        ctx.accounts.token_lottery.ticket_price = price;
        // ctx.accounts.token_lottery.authority = #ctx.accounts.payer.key;
        ctx.accounts.token_lottery.authority = *ctx.accounts.payer.key;
        ctx.accounts.token_lottery.lottery_pot_amount = 0;
        ctx.accounts.token_lottery.randomness_account = Pubkey::default();
        ctx.accounts.token_lottery.winner_chosen = false;
        
        Ok(())
    }

    pub fn initialize_lottery(ctx: Context<InitializeLottery>) -> Result<()> {

        let signer_seeds: &[&[&[u8]]] = &[&[
            b"collection_mint".as_ref(),
            &[ctx.bumps.collection_mint],
        ]];

        msg!("Create Mint account");
        mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.collection_mint.to_account_info(),
                    to: ctx.accounts.payer.to_account_info(),
                    authority: ctx.accounts.collection_mint.to_account_info(),
                },
                signer_seeds,
            )
        )?;

        //.........
        msg!("Creating Metadata account");
        create_metadata_account_v3(
            CpiContext::new_with_signer(
                ctx.accounts.token_metadata_program.to_account_info(),
                CreateMetadataAccountv3 {
                    metadata: ctx.accounts.metadata.to_account_info(),
                    mint: ctx.accounts.collection_mint.to_account_info(),
                    mint_authority: ctx.accounts.collection_mint.to_account_info(),
                    payer: ctx.accounts.payer.to_account_info(),
                    update_authority: ctx.accounts.collection_mint.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
                &signer_seeds
            ),
            DataV2 {
                name: NAME.to_string(),
                symbol: SYMBOL.to_string(),
                uri: URI.to_string(),
                seller_fee_basis_points: 0,
                creators: Some(vec! [Creator{
                    address: ctx.accounts.collection_mint.key(),
                    verified: false,
                    share: 100,
                }]),
                collection: None,
                uses: None,
            },
            true,
            true,
            Some(CollectionDetails::V1 { size: 0})
        )?;

        msg!("Creating Master Edition account");
        create_master_editon_v3(
            CpiContext::new_with_signer(
                ctx.accounts.token_metadata_program.to_account_info(),
                CreateMasterEditionV3{
                    payer: ctx.accounts.payer.to_account_info(),
                    mint: ctx.accounts.collection_mint.to_account_info(),
                    edition: ctx.accounts.master_edition.to_account_info(),
                    mint_authority: ctx.accounts.collection_mint.to_account_info(),
                    update_authority: ctx.accounts.collection_mint.to_account_info(),
                    metadata: ctx.accounts.metadata.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
                &signer_seeds
            ),
            Some(0)
        )?;
//.......

        msg!("verifying collection");
        sign_metadata(
            CpiContext::new_with_signer (
                ctx.accounts.token_metadata_program.to_account_info(),
                SignMetadata (
                    creator: ctx.accounts.collection_gist.to_account_info(),
                    metadata: ctx.accounts.metadata.to_account_info(),
                ),
                signer_seeds
            ))?;

        Ok(())
    }
}

    pub fn buy_ticket(ctx: Context<buyTicket>, ) -> Result<()> {
        let clock: Clock = Clock::get()?;
        let ticket_name: String = NAME.to_owned() + ctx.accounts.token_lottery.total_tickets.to_string().as_str();

        if clock.slot < ctx.accounts.token_lottery.start_time ||
            clock.slot > ctx.accounts.token_lottery.end_time {
                return Err(ErrorCode::LotteryNotOpen.info());
            }

        system_program::transfer{
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.payer.to_account_info(),
                    to: ctx.accounts.token_lottery.to_account_info(),
                }
            ),
            ctx.accounts.token_lottery.ticket_price,
        }?;

        let signer_seeds: &[&[&[u8]]] = &[&[
            b"collection_mint".as_ref(),
            &[ctx.bumps.collection_mint],
        ]];

        mint_to {
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Minto {
                    mint: ctx.accounts.ticket_mint.to_account_info(),
                    to: ctx.accounts.destination.to_account_info(),
                    authority: ctx.accounts.collection_mint.to_account_info(),
                },
                &signer_seeds
            ),
            1,
        }?;


        
        msg!("Creating Metadata account");
        create_metadata_account_v3(
            CpiContext::new_with_signer(
                ctx.accounts.token_metadata_program.to_account_info(),
                CreateMetadataAccountv3 {
                    metadata: ctx.accounts.ticket_metadata.to_account_info(),
                    mint: ctx.accounts.ticket_mint.to_account_info(),
                    mint_authority: ctx.accounts.collection_mint.to_account_info(),
                    payer: ctx.accounts.payer.to_account_info(),
                    update_authority: ctx.accounts.collection_mint.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
                &signer_seeds
            ),
            DataV2 {
                name: ticket_name,
                symbol: SYMBOL.to_string(),
                uri: URI.to_string(),
                seller_fee_basis_points: 0,
                creators: None,
                collection: None,
                uses: None,
            },
            true,
            true,
            None,
        )?;

        msg!("Creating Master Edition account");
        create_master_editon_v3(
            CpiContext::new_with_signer(
                ctx.accounts.token_metadata_program.to_account_info(),
                CreateMasterEditionV3{
                    payer: ctx.accounts.payer.to_account_info(),
                    mint: ctx.accounts.ticket_mint.to_account_info(),
                    edition: ctx.accounts.ticket_master_edition.to_account_info(),
                    mint_authority: ctx.accounts.collection_mint.to_account_info(),
                    update_authority: ctx.accounts.collection_mint.to_account_info(),
                    metadata: ctx.accounts.ticket_metadata.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
                &signer_seeds
            ),
            Some(0)
        )?;

        set_and_verify_sized_collection_item(
            CpiContext::new_with_signer(
                ctx.accounts.token_metadata_program.to_account_info(),
                SetAndVerifySizedCollectionItem {
                    metadata: ctx.accounts.ticket_metadata.to_account_info(),
                    collection_authority: ctx.accounts.collection_mint.to_account_info(),
                    payer = ctx.accounts.payer.to_account_info(),
                    update_authority: ctx.accounts.collection_mint.to_account_info(),
                    collection_mint: ctx.accounts.collection_mint.to_account_info(),
                    collection_metadata: ctx.accounts.collection_metadata.to_account_info(),
                    collection_master_edition: ctx.accounts.collection_master_edition.to_account_info(),
                },
                &signer_seeds
            ),
            None
        )?;

        ctx.accounts.token_lottery_tickets += 1;

        Ok(())
    }

    pub fn commit_randomness(ctx: Context<CommitRandomness>) -> Result<()> {

        let clock: Clock = Clock::get()?;
        let token_lottery = &mut ctx.accounts.token_lottery;

        if ctx.accounts.payer.key() != token_lottery.authority {
            return Err(ErrorCode::NotAuthorized.into());
        }

        let randomness_data = RandomnessAccountData::parse{
            ctx.accounts.randomness_account.data.borrow()}.unwrap(); 

        if randomness_data.seed_slot != clock.slot - 1 (
            return Err(ErrorCode::RandomnessAlreadyRevealed.into());
        )

        token_lottery.randomness_account = ctx.accounts.randomness_account.key();

        Ok(())
    }

    pub fn reveal_winner(ctx: Context<RevealWinner) -> Result<()> {

        let clock: Clock = Clock::get()?;
        let token_lottery: &mut Account<TokenLottery> = &mut ctx.accounts.token_lottery;

        if ctx.accounts.payer.key() != token_lottery.authority {
            return Err(ErrorCode::NotAuthorized.into());
        }

        if ctx.accounts.randomness_account.key() != token_lottery.randomness_account {
            return Err(ErrorCode::IncorrectRandomnessAccount.into());
        }

        if clock.slot < token_lottery.end_time {
            return Err(ErrorCode::LotteryNotCompleted.into());
        }

        require:(!token_lottery.winner_chosen, ErrorCode::winnerChosen);

        let randomness_data: Ref<RandomnessAccountData> = RandomnessAccountData::parse(
            ctx.accounts.randomness_account.data.borrow()).unwrap();

        let reveal_random_value: [u8: 32] = randomness_data.get_value(&clock)
            .map_err(|| ErrorCode::RandonessNotResolved)?;

        let winner: u64 = reveal_random_value[0] as u64 % token_lottery.total_tickets;

        msg!("Winner Chosen {}", winner);

        token_lottery.winner = winner;
        token_lottery.winner_chosen = true;
        
        // ........
        Ok(())
    }

    pub fn claim_winnigs(ctx: Context<ClaimWinnings>) -> Result<()> {
        require!(ctx.accounts.token_lottery.winner_chosen, ErrorCode::winnerNotChosen);

        require!(ctx.accounts.ticket_metadata.collection.as_ref().unwrap().verified, ErrorCode::NotVerified);
        require!(ctx.accounts.ticket_metadata.collection.as_ref().unwrap().key == ctx.accounts.collection_mint.key(), ErrorCode::IncorrectTicket);

        let ticket_name: String = NAME.to_owned() + &ctx.accounts.token_lottery.winner.to_string();
        let metadata_name: String = ctx.accounts.ticket_metadata.name.replace("\u{0}", "");

        require!(metadata_name == ticket_name, ErrorCode::IncorrectTicket);
        require!(ctx.accounts.ticket_account.amount > 0, ErrorCode::NoTicket);

        **ctx.accounts.token_lottery.to_account_info().lamports.borrow_nut() -> ctx.accounts.token_lottery.lottery_pot_amount;
        **ctx.Accounts.payer.to_account_info().lamports.borrow_nut() += ctx.accounts.token_lottery.lottery_pot_amount;

        ctx.accounts.token_lottery.lottery_pot_amount;

        Ok(())
    }



#[derive(Accounts)]
pub struct Initialize<'info> {

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = 8 + TokenLottery::INIT_SPACE,
        seeds = [b"token_lottery".as_ref()],
        bump
    )]
    pub token_lottery: Account<'info, TokenLottery>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeLottery<'info> {
    
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = collection_mint,
        mint::freeze_authority = collection_mint,
        seeds = [b"collection_mint".as_ref()],
        bump,
    )]
    pub collection_mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = payer,
        token::mint = collection_mint,
        token::authority = collection_token_account,
        seeds = [b"collection_associated_token".as_ref()],
        bump,
    )]
    pub collection_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [
            b"metadata", 
            token_metadata_program.key().as_ref(), 
            collection_mint.key().as_ref()
            ],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    // CHECK: This account is checked by the metadata smart contract.
    pub master_edition: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [
            b"metadata", 
            token_metadata_program.key().as_ref(), 
            collection_mint.key().as_ref(),
            b"edition",
            ],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    // CHECK: This account is checked by the metadata smart contract.
    pub master_edition: UncheckedAccount<'info>,

    pub token_metadata_program: Program<'info, Metadata>,
    pub associate_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct BuyTicket<'info> {
    
    #[account(
        mut,
        seeds = [b"collection_mint".as_ref()],
        bump,
    )]
    pub collection_mint: InterfaceAccount<'info, Mint>,

    pub associate_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub token_metadata_program: Program<'info, Metadata>,
    // We need system program because we are going to be creating a new ticket.
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Mint>,
}

#[derive(Accounts)]
pub struct CommitRandomness<'info> {

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"token_lottery".as_ref()],
        bump = token_lottery.bump,
    )]
    pub token_lottery: Account<'info, TokenLottery>,

    // / CHECK: This account is checked by the Switchboard smart contract.
    pub randomness_account: UncheckedAccount<'info>,
}


#[derive(Accounts)]
pub struct RevealWinner<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"token_lottery".as_ref()],
        bump = token_lottery.bump,
    )]
    pub token_lottery: Account<'info, TokenLottery>,

    // CHECK: This account is checked by the Swichboard smart contract.
    pub randomness_account: UncheckedAccount<'info>,

}


#[derive(Accounts)]
pub struct ClaimWinnings<'info> {

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [b'token_lottery".as_ref()],
        bump = token_lottery.bump,
    )]
    pub token_lottery: Account<'info, TokenLottery>,

    #[account(
        seeds = [token_lottery.winner.to_le_bytes().as_ref()],
        bump,
    )]
    pub ticket_mint: InterfaceAccount<'info, Mint>,

    #[account(
        seeds = [b"collection_mint".as_ref()],
        bump,
    )]
    pub collection_mint: InterfaceAccount<'info, Mint>,

    #[account(
        seeds = [b"metadata", token_metadata_program.key().as_ref(), ticket_mint.key().as_ref()],
        bump,
        seed::program = token_metadata_program.key(),
    )]
    pub ticket_metadata: Account<'info, MetadataAccount>,

    #[account(
        associated_token::mint = ticket_mint,
        associated_token::authority = payer,
        associated_token::token_program = token_program,
    )]
    pub ticket_account: InterfaceAccount<'info, TokenAccount>,
    
    #[account(
        seeds = [b"metadata", token_metadata_program.key().as_ref(), collection_mint.key().as_ref()],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    pub collection_metadata: Account<'info, MetadataAccount>,

    pub token_metadata_program: Program<'info, Metadata>
    pub token_program: Interface<'info, TokenInterface>,
}


#[account]
#[derive(InitSpace)]
pub struct TokenLottery {
    pub bump: u8,
    pub winner: u64,
    pub winner_chosen: bool,
    pub start_time: u64,
    pub end_time: u64,
    pub lottery_pot_amount: u64,
    pub total_tickets: u64,
    pub ticket_price: u64,
    pub authority: Pubkey,
    pub randomness_account: Pubkey,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Lottery is not open")]
    LotteryNotOpen,
    #[msg("Not Authorized")]
    NotAuthorized,
    #[msg("Randoness already revealed")]
    RandomnessAlreadyRevealed,
    #[msg(" Winner Not Chosen")]
    winnerNotChosen,
}




