use cosmwasm_std::{coin, Coin, Decimal, Uint128};
use liquidy_rs::swap::InstantiateMsg;
use liquidy_rs_testing::mock_fin::MockFin;
use liquidy_rs_testing::mock_liquidy_app::LiquidyApp;
use liquidy_rs_testing::mock_liquidy_swap::MockLiquidySwap;
//use rujira_rs::TokenMetadata;
use rujira_rs_testing::mock_rujira_app;
use std::str::FromStr;

pub struct TestEnv {
    pub app: LiquidyApp,
    pub swap: MockLiquidySwap,
    pub fin_pairs: Vec<(String, MockFin)>,
}

impl TestEnv {
    /// Populates orderbooks for all fin pairs with standard test parameters
    /// This eliminates the repetitive orderbook setup code across tests
    pub fn populate_orderbooks(&mut self, owner: &str) {
        let owner_addr = self.app.api().addr_make(owner);

        self.fin_pairs.iter().for_each(|(denom, swap_mock)| {
            swap_mock
                .populate_orderbook(
                    &mut self.app,
                    &owner_addr,
                    vec![
                        coin(100_000_000_000, "eth-usdc"),
                        coin(100_000_000_000, denom.clone()),
                    ],
                    Decimal::from_str("100").unwrap(),
                    &[1u64, 2u64, 3u64],
                    Uint128::from(1_000_000_000u128),
                )
                .unwrap();
        });
    }
}

pub fn setup(balances: Vec<(&str, Vec<Coin>)>, mocked_fin_pairs: Vec<String>) -> TestEnv {
    let mut liquidy_app = LiquidyApp::new(mock_rujira_app());

    for (addr, coins) in balances {
        liquidy_app.add_balance(addr, coins, true);
    }
    let fee_collector = liquidy_app.api().addr_make("fee_collector");

    let swap_mocks = get_mock_fin_pairs(&mut liquidy_app, mocked_fin_pairs);

    // let entry_adapter = MockNamiIndexEntryAdapter::new(
    //     &mut nami_app,
    //     InstantiateMsg {
    //         quote_denom: quote_denom.clone(),
    //         swap_contracts: swap_mocks
    //             .iter()
    //             .map(|(denom, swap_mock)| (denom.clone(), swap_mock.address.to_string()))
    //             .collect(),
    //     },
    // );
    let swap = MockLiquidySwap::new(
        &mut liquidy_app,
        InstantiateMsg {
            fee_collector: fee_collector.to_string(),
            fee_bps: 10u16,
            max_affiliate_fee_bps: 500u16,
        },
    );

    TestEnv {
        app: liquidy_app,
        swap,
        fin_pairs: swap_mocks,
    }
}

fn get_mock_fin_pairs(app: &mut LiquidyApp, fin_tokens: Vec<String>) -> Vec<(String, MockFin)> {
    let mut swap_mocks: Vec<(String, MockFin)> = Vec::new();
    for denom in fin_tokens {
        let swap_mock = MockFin::new_app_layer(app, denom.as_str(), "eth-usdc");

        swap_mocks.push((denom.clone(), swap_mock));
    }
    swap_mocks
}
