use cosmwasm_std::{ensure, Order, StdResult, Storage};
use cw_storage_plus::{Bound, Map};
use liquidy_rs::swap::Affiliate;

use crate::config::Config;
use crate::ContractError;

pub struct Status<'a> {
    pub affiliates: Map<&'a str, Affiliate>,
}

impl<'a> Status<'a> {
    pub const fn new() -> Self {
        Self {
            affiliates: Map::new("affiliates"),
        }
    }
    pub fn add_affiliate(
        &self,
        storage: &mut dyn Storage,
        affiliate: Affiliate,
    ) -> Result<(), ContractError> {
        let config = Config::load(storage)?;
        ensure!(
            affiliate.affiliate_fee_bps <= config.max_affiliate_fee_bps,
            ContractError::InvalidAffiliateFee {
                max: config.max_affiliate_fee_bps
            }
        );

        ensure!(
            affiliate.referral_fee_share_bps <= 10000,
            ContractError::InvalidReferralFee { max: 10000 }
        );

        self.affiliates
            .save(storage, &affiliate.affiliate_code.clone(), &affiliate)?;
        Ok(())
    }

    pub fn remove_affiliate(&self, storage: &mut dyn Storage, key: &'a str) -> StdResult<()> {
        self.affiliates.remove(storage, key);
        Ok(())
    }

    pub fn get_affiliates(
        &self,
        storage: &dyn Storage,
        limit: u8,
        start_after: Option<String>,
    ) -> StdResult<Vec<(String, Affiliate)>> {
        let start = start_after.as_deref().map(Bound::exclusive);

        let affiliates = self
            .affiliates
            .range(storage, start, None, Order::Ascending)
            .take(limit as usize)
            .collect::<StdResult<Vec<_>>>()?;

        Ok(affiliates)
    }

    pub fn get(&self, storage: &dyn Storage, key: &'a str) -> StdResult<Option<Affiliate>> {
        self.affiliates.may_load(storage, key)
    }
}
