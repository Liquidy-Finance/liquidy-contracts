use cosmwasm_std::{Addr, Coin, StdResult};
use cw_multi_test::{AppResponse, ContractWrapper, Executor};
use liquidy_rs::swap::{
    Affiliate, AffiliateResponse, AffiliatesResponse, ConfigResponse, ExecuteMsg, InstantiateMsg,
    QueryMsg, Stage, SudoMsg,
};
use liquidy_swap::contract::{execute, instantiate, query, sudo};
use rujira_rs::CallbackData;
use rujira_rs_testing::RujiraApp;

pub struct MockLiquidySwap {
    pub address: Addr,
}

impl MockLiquidySwap {
    pub fn new(app: &mut RujiraApp, msg: InstantiateMsg) -> Self {
        let code = Box::new(ContractWrapper::new(execute, instantiate, query).with_sudo(sudo));
        let code_id = app.store_code(code);
        let owner = app.api().addr_make("owner");
        let swap = app
            .instantiate_contract(code_id, owner.clone(), &msg, &[], "Swap", None)
            .unwrap();

        MockLiquidySwap { address: swap }
    }

    pub fn execute_swap(
        &self,
        app: &mut RujiraApp,
        user: &str,
        funds: Vec<Coin>,
        min_return: Coin,
        stages: Vec<Stage>,
        affiliate_code: Option<String>,
        callback: Option<CallbackData>,
    ) -> anyhow::Result<AppResponse> {
        let user_addr = app.api().addr_make(user);
        app.execute_contract(
            user_addr.clone(),
            self.address.clone(),
            &ExecuteMsg::Swap {
                min_return,
                stages,
                recipient: Some(app.api().addr_make(user)),
                affiliate_code,
                callback,
            },
            &funds,
        )
    }

    pub fn sudo_execute_config_fee_collector_update(
        &self,
        app: &mut RujiraApp,
        fee_collector: String,
    ) -> anyhow::Result<AppResponse> {
        app.wasm_sudo(
            self.address.clone(),
            &SudoMsg::UpdateFeeCollectorConfig { fee_collector },
        )
    }
    pub fn sudo_execute_config_fee_bps_update(
        &self,
        app: &mut RujiraApp,
        fee_bps: u16,
    ) -> anyhow::Result<AppResponse> {
        app.wasm_sudo(
            self.address.clone(),
            &SudoMsg::UpdateFeeBbsConfig { fee_bps },
        )
    }
    pub fn sudo_execute_add_affiliate(
        &self,
        app: &mut RujiraApp,
        affiliate: Affiliate,
    ) -> anyhow::Result<AppResponse> {
        app.wasm_sudo(self.address.clone(), &SudoMsg::AddAffiliate { affiliate })
    }
    pub fn sudo_execute_remove_affiliate(
        &self,
        app: &mut RujiraApp,
        affiliate_code: String,
    ) -> anyhow::Result<AppResponse> {
        app.wasm_sudo(
            self.address.clone(),
            &SudoMsg::RemoveAffiliate { affiliate_code },
        )
    }

    pub fn query_config(&self, app: &mut RujiraApp) -> StdResult<ConfigResponse> {
        app.wrap()
            .query_wasm_smart(self.address.clone(), &QueryMsg::Config {})
    }

    pub fn query_affiliates(
        &self,
        app: &mut RujiraApp,
        limit: Option<u8>,
        start_after: Option<String>,
    ) -> StdResult<AffiliatesResponse> {
        app.wrap().query_wasm_smart(
            self.address.clone(),
            &QueryMsg::Affiliates { limit, start_after },
        )
    }
    pub fn query_affiliate(
        &self,
        app: &mut RujiraApp,
        affiliate_code: String,
    ) -> StdResult<AffiliateResponse> {
        app.wrap().query_wasm_smart(
            self.address.clone(),
            &QueryMsg::Affiliate { affiliate_code },
        )
    }
}
