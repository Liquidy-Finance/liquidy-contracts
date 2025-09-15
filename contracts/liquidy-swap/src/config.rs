use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Api, StdResult, Storage};
use cw_storage_plus::Item;
use liquidy_rs::swap::InstantiateMsg;

use crate::ContractError;

static CONFIG: Item<Config> = Item::new("config");

#[cw_serde]
pub struct Config {
    pub fee_collector: Addr,
    pub fee_bps: u16,
    pub max_affiliate_fee_bps: u16,
}

impl Config {
    pub fn new(api: &dyn Api, msg: InstantiateMsg) -> StdResult<Self> {
        Ok(Self {
            fee_bps: msg.fee_bps,
            max_affiliate_fee_bps: msg.max_affiliate_fee_bps,
            fee_collector: api.addr_validate(&msg.fee_collector)?,
        })
    }

    pub fn load(storage: &dyn Storage) -> StdResult<Self> {
        CONFIG.load(storage)
    }

    pub fn validate(&self) -> Result<(), ContractError> {
        //self.fee_collector.validate()?;
        Ok(())
    }

    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        CONFIG.save(storage, self)
    }
}

#[cfg(test)]
mod tests {
    //use super::*;

    // #[test]
    // fn validation() {
    //     let mut app = App::default();
    //     let fee_collector = app.api().addr_make("fee_collector");
    //     Config {fee_collector:fee_collector, fee_bps:10u16}.validate().unwrap();
    // }
}
