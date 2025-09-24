use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin, Uint128};
use rujira_rs::fin::SimulationResponse;
use rujira_rs::CallbackData;

#[cw_serde]
pub struct InstantiateMsg {
    pub fee_collector: String,
    pub fee_bps: u16,
    pub max_affiliate_fee_bps: u16,
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    Swap {
        min_return: Coin,
        stages: Vec<Stage>,
        recipient: Option<Addr>,
        affiliate_code: Option<String>,
        callback: Option<CallbackData>,
    },
}

#[cw_serde]
pub enum SudoMsg {
    UpdateFeeCollectorConfig { fee_collector: String },
    UpdateFeeBbsConfig { fee_bps: u16 },
    UpdateMaxAffiliateFeeBbsConfig { max_affiliate_fee_bps: u16 },
    AddAffiliate { affiliate: Affiliate },
    RemoveAffiliate { affiliate_code: String },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},

    #[returns(SimulationResponse)]
    Simulate { coin: Coin, stages: Vec<Stage> },

    #[returns(AffiliatesResponse)]
    Affiliates {
        limit: Option<u8>,
        start_after: Option<String>,
    },

    #[returns(AffiliateResponse)]
    Affiliate { affiliate_code: String },
}

#[cw_serde]
pub struct Affiliate {
    pub affiliate_code: String,
    pub referral_fee_share_bps: u16,
    pub affiliate_fee_bps: u16,
    pub referral_payment_address: String,
    pub affiliate_payment_address: String,
}

#[cw_serde]
pub struct Stage {
    pub address: Addr,
    pub denom: String,
}
#[cw_serde]
pub struct SwapEntry {
    pub denom: String,
    pub amount: Uint128,
    pub min_return: Option<Uint128>,
}

#[cw_serde]
pub struct ConfigResponse {
    pub fee_collector: Addr,
    pub fee_bps: u16,
}

#[cw_serde]
pub struct AffiliatesResponse {
    pub affiliates: Vec<(String, Affiliate)>,
}

#[cw_serde]
pub struct AffiliateResponse {
    pub affiliate: Affiliate,
}
