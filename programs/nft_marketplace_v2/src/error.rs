use anchor_lang::error_code;

#[error_code]
pub enum Error {
    #[msg("Price must be at least 1 lamports")]
    InvalidPrice,
    #[msg("Invalid quantity")]
    InvalidQuantity,
    #[msg("Cash back should lower than 1")]
    CashbackMax,
    #[msg("Please submit the asking price in order to complete the purchase")]
    InvalidPayment,
    #[msg("Invalid account")]
    InvalidStateAccount,
    #[msg("State already has been initialized")]
    StateAlreadyInitialized,
    #[msg("Item list is not available for gacha")]
    ItemsUnavailableForGacha,
    #[msg("Listing not on")]
    ListingNotOn,
    #[msg("Auction on")]
    AuctionOn,
    #[msg("Auction canceled")]
    AuctionCanceled,
    #[msg("Not authorized")]
    NotAuthorized,
    #[msg("Not winner")]
    NotWinner,
    #[msg("Not auction")]
    NotAuction,
    #[msg("Not On Sell")]
    NotOnSell,
    #[msg("Owner cut from 0 to 10000")]
    InvalidOwnerCut,
    #[msg("Cannot bid own auction")]
    InvalidBid,
}