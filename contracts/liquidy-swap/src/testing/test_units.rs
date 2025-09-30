use std::str::FromStr;

use crate::testing::env;
use cosmwasm_std::Event;
use cosmwasm_std::{coin, coins, Decimal, Uint128};
use liquidy_rs::swap::{Affiliate, Stage};

#[test]
fn base_test() {
    // Initialize user balances
    let balances = vec![
        (
            "user",
            vec![
                coin(10_000_000_000, "nami"),
                coin(10_000_000_000, "auto"),
                coin(10_000_000_000, "lqdy"),
                coin(10_000_000_000, "eth-usdc"),
            ],
        ),
        (
            "owner",
            vec![
                coin(100_000_000_000, "nami"),
                coin(100_000_000_000, "auto"),
                coin(100_000_000_000, "lqdy"),
                coin(500_000_000_000, "eth-usdc"),
            ],
        ),
        ("fee_collector", vec![]),
    ];
    let mut test_env = env::setup(
        balances,
        vec!["nami".to_string(), "auto".to_string(), "lqdy".to_string()],
    );

    // populate swap mocks so that fair price is 1 for everyone
    let owner = test_env.app.api().addr_make("owner");

    //let fee_collector = test_env.app.api().addr_make("fee_collector");

    test_env.fin_pairs.iter().for_each(|(denom, swap_mock)| {
        // if denom=="lqdy"{
        //     lqdy_contract=swap_mock.address.clone();
        // }
        swap_mock
            .populate_orderbook(
                &mut test_env.app,
                &owner.clone(),
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

    let (_lqdy_token, lqdy_mocked_fin) = &test_env.fin_pairs[2];

    //Successful swap
    test_env
        .swap
        .execute_swap(
            &mut test_env.app,
            "user",
            coins(1_700_000_000u128, "eth-usdc"),
            coin(16_983_000u128, "lqdy"),
            vec![Stage {
                address: lqdy_mocked_fin.address.clone(),
                denom: "eth-usdc".to_string(),
            }],
            Some("ruji".to_string()), //None,//
            None,
        )
        .unwrap();

    // queries
    // let resp = test_env
    //     .swap
    //     .query_config(&mut test_env.app)
    //     .unwrap();
    //     assert_eq!(resp.fee_bps,10u16
    // );

    let user = test_env.app.api().addr_make("user");

    let lqdy_balance_user = test_env.app.query_balance("user", "lqdy", true);

    assert_eq!(
        lqdy_balance_user,
        Uint128::from(10_000_000_000u128 + 17_000_000u128 - 17_000u128)
    );
    let lqdy_balance_fee_collector = test_env.app.query_balance("fee_collector", "lqdy", true);
    assert_eq!(lqdy_balance_fee_collector, Uint128::from(17_000u128));

    let usdc_balance = test_env.app.query_balance("user", "eth-usdc", true);
    assert_eq!(
        usdc_balance,
        Uint128::from(10_000_000_000u128 - 1_700_000_000u128)
    );
}

#[test]
fn simulate_simple_swap() {
    // Initialize user balances (same as base_test)
    let balances = vec![
        (
            "user",
            vec![
                coin(10_000_000_000, "nami"),
                coin(10_000_000_000, "auto"),
                coin(10_000_000_000, "lqdy"),
                coin(10_000_000_000, "eth-usdc"),
            ],
        ),
        (
            "owner",
            vec![
                coin(100_000_000_000, "nami"),
                coin(100_000_000_000, "auto"),
                coin(100_000_000_000, "lqdy"),
                coin(500_000_000_000, "eth-usdc"),
            ],
        ),
        ("fee_collector", vec![]),
    ];
    let mut test_env = env::setup(
        balances,
        vec!["nami".to_string(), "auto".to_string(), "lqdy".to_string()],
    );

    // populate swap mocks so that fair price is 1 for everyone (same as base_test)
    test_env.populate_orderbooks("owner");

    let (_lqdy_token, lqdy_mocked_fin) = &test_env.fin_pairs[2];

    // Test simulation with the same parameters as base_test
    let simulation_result = test_env
        .swap
        .query_simulate(
            &mut test_env.app,
            coin(1_700_000_000u128, "eth-usdc"),
            vec![Stage {
                address: lqdy_mocked_fin.address.clone(),
                denom: "eth-usdc".to_string(),
            }],
        )
        .unwrap();

    // Verify simulation result
    // Expected: 16,983,000 lqdy tokens with 17,000 fee
    assert_eq!(simulation_result.returned, Uint128::from(16_983_000u128));
    assert_eq!(simulation_result.fee, Uint128::from(17_000u128));

    // Test simulation with different amount
    let simulation_result_2 = test_env
        .swap
        .query_simulate(
            &mut test_env.app,
            coin(850_000_000u128, "eth-usdc"), // Half the amount
            vec![Stage {
                address: lqdy_mocked_fin.address.clone(),
                denom: "eth-usdc".to_string(),
            }],
        )
        .unwrap();

    // Should return half the tokens minus platform fee
    // Expected: 8,491,500 with 8,500 fee
    assert_eq!(simulation_result_2.returned, Uint128::from(8_491_500u128));
    assert_eq!(simulation_result_2.fee, Uint128::from(8_500u128));
}

#[test]
fn multi_hop() {
    // Initialize user balances
    let balances = vec![
        (
            "user",
            vec![
                coin(10_000_000_000, "nami"),
                coin(10_000_000_000, "auto"),
                coin(10_000_000_000, "lqdy"),
                coin(10_000_000_000, "eth-usdc"),
            ],
        ),
        (
            "owner",
            vec![
                coin(100_000_000_000, "nami"),
                coin(100_000_000_000, "auto"),
                coin(100_000_000_000, "lqdy"),
                coin(500_000_000_000, "eth-usdc"),
            ],
        ),
        ("fee_collector", vec![]),
    ];
    let mut test_env = env::setup(
        balances,
        vec!["nami".to_string(), "auto".to_string(), "lqdy".to_string()],
    );
    let (_nami_token, nami_mocked_fin) = &test_env.fin_pairs[0];
    //let (_auto_token,auto_mocked_fin )=&test_env.fin_pairs[1];
    let (_lqdy_token, lqdy_mocked_fin) = &test_env.fin_pairs[2];

    // populate swap mocks so that fair price is 1 for everyone
    let owner = test_env.app.api().addr_make("owner");
    //let user = test_env.app.api().addr_make("user");
    //let fee_collector = test_env.app.api().addr_make("fee_collector");

    test_env.fin_pairs.iter().for_each(|(denom, swap_mock)| {
        swap_mock
            .populate_orderbook(
                &mut test_env.app,
                &owner.clone(),
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

    //Successful swap
    test_env
        .swap
        .execute_swap(
            &mut test_env.app,
            "user",
            coins(17_000_000u128, "lqdy"),
            coin(1u128, "eth-usdc"),
            vec![Stage {
                address: lqdy_mocked_fin.address.clone(),
                denom: "lqdy".to_string(),
            }],
            None,
            None,
        )
        .unwrap();

    let mut lqdy_balance_user = test_env.app.query_balance("user", "lqdy", true);
    assert_eq!(
        lqdy_balance_user,
        Uint128::from(10_000_000_000u128 - 17_000_000u128)
    );
    let mut usdc_balance_user = test_env.app.query_balance("user", "eth-usdc", true);
    assert_eq!(
        usdc_balance_user,
        Uint128::from(10_000_000_000u128 + 1_674_424_821u128)
    );
    let usdc_balance_fee_collector = test_env
        .app
        .query_balance("fee_collector", "eth-usdc", true);
    assert_eq!(usdc_balance_fee_collector, Uint128::from(1_676_101u128));

    test_env
        .swap
        .execute_swap(
            &mut test_env.app,
            "user",
            coins(16_830_000u128, "eth-usdc"),
            coin(1u128, "nami"),
            vec![Stage {
                address: nami_mocked_fin.address.clone(),
                denom: "eth-usdc".to_string(),
            }],
            None,
            None,
        )
        .unwrap();

    usdc_balance_user = test_env.app.query_balance("user", "eth-usdc", true);
    assert_eq!(
        usdc_balance_user,
        Uint128::from(10_000_000_000u128 + 1_674_424_821u128 - 16_830_000u128)
    );
    let mut nami_balance_user = test_env.app.query_balance("user", "nami", true);
    assert_eq!(
        nami_balance_user,
        Uint128::from(10_000_000_000u128 + 168_131u128)
    );
    let mut nami_balance_fee_collector = test_env.app.query_balance("fee_collector", "nami", true);
    assert_eq!(nami_balance_fee_collector, Uint128::from(169u128));

    //Successful swap
    test_env
        .swap
        .execute_swap(
            &mut test_env.app,
            "user",
            coins(170_000u128, "lqdy"),
            coin(1u128, "nami"),
            vec![
                Stage {
                    address: nami_mocked_fin.address.clone(),
                    denom: "eth-usdc".to_string(),
                },
                Stage {
                    address: lqdy_mocked_fin.address.clone(),
                    denom: "lqdy".to_string(),
                },
            ],
            None,
            None,
        )
        .unwrap();

    lqdy_balance_user = test_env.app.query_balance("user", "lqdy", true);
    assert_eq!(
        lqdy_balance_user,
        Uint128::from(10_000_000_000u128 - 17_000_000u128 - 170_000u128)
    );
    nami_balance_user = test_env.app.query_balance("user", "nami", true);
    assert_eq!(
        nami_balance_user,
        Uint128::from(10_000_000_000u128 + 168_131u128 + 166_433u128)
    );
    nami_balance_fee_collector = test_env.app.query_balance("fee_collector", "nami", true);
    assert_eq!(nami_balance_fee_collector, Uint128::from(168u128 + 168u128));
}

#[test]
fn simulate_multihop_swap() {
    // Initialize user balances
    let balances = vec![
        (
            "user",
            vec![
                coin(10_000_000_000, "nami"),
                coin(10_000_000_000, "auto"),
                coin(10_000_000_000, "lqdy"),
                coin(10_000_000_000, "eth-usdc"),
            ],
        ),
        (
            "owner",
            vec![
                coin(100_000_000_000, "nami"),
                coin(100_000_000_000, "auto"),
                coin(100_000_000_000, "lqdy"),
                coin(500_000_000_000, "eth-usdc"),
            ],
        ),
        ("fee_collector", vec![]),
    ];
    let mut test_env = env::setup(
        balances,
        vec!["nami".to_string(), "auto".to_string(), "lqdy".to_string()],
    );

    // populate swap mocks so that fair price is 1 for everyone
    test_env.populate_orderbooks("owner");

    let (_nami_token, nami_mocked_fin) = &test_env.fin_pairs[0];
    //let (_auto_token,auto_mocked_fin )=&test_env.fin_pairs[1];
    let (_lqdy_token, lqdy_mocked_fin) = &test_env.fin_pairs[2];

    let simulation_result_step_1 = test_env
        .swap
        .query_simulate(
            &mut test_env.app,
            coin(170_000u128, "lqdy"),
            vec![Stage {
                address: lqdy_mocked_fin.address.clone(),
                denom: "lqdy".to_string(),
            }],
        )
        .unwrap();

    // Verify simulation result
    // Expected: 16,813,170 with fee 16,830
    assert_eq!(
        simulation_result_step_1.returned,
        Uint128::from(16_813_170u128)
    );
    assert_eq!(simulation_result_step_1.fee, Uint128::from(16_830u128));

    let simulation_result_step_2 = test_env
        .swap
        .query_simulate(
            &mut test_env.app,
            coin(16_813_170u128, "eth-usdc"),
            vec![Stage {
                address: nami_mocked_fin.address.clone(),
                denom: "eth-usdc".to_string(),
            }],
        )
        .unwrap();

    // Verify simulation result
    // Expected: 167,962 with fee 169
    assert_eq!(
        simulation_result_step_2.returned,
        Uint128::from(167_962u128)
    );
    assert_eq!(simulation_result_step_2.fee, Uint128::from(169u128));

    let simulation_result = test_env
        .swap
        .query_simulate(
            &mut test_env.app,
            coin(170_000u128, "lqdy"),
            vec![
                Stage {
                    address: nami_mocked_fin.address.clone(),
                    denom: "eth-usdc".to_string(),
                },
                Stage {
                    address: lqdy_mocked_fin.address.clone(),
                    denom: "lqdy".to_string(),
                },
            ],
        )
        .unwrap();

    // Verify simulation result
    // Expected: 168,131 with fee 169
    assert_eq!(simulation_result.returned, Uint128::from(168_131u128));
    assert_eq!(simulation_result.fee, Uint128::from(169u128));
}

#[test]
fn min_return() {
    // Initialize user balances
    let balances = vec![
        (
            "user",
            vec![
                coin(10_000_000_000, "nami"),
                coin(10_000_000_000, "auto"),
                coin(10_000_000_000, "lqdy"),
                coin(10_000_000_000, "eth-usdc"),
            ],
        ),
        (
            "owner",
            vec![
                coin(100_000_000_000, "nami"),
                coin(100_000_000_000, "auto"),
                coin(100_000_000_000, "lqdy"),
                coin(500_000_000_000, "eth-usdc"),
            ],
        ),
        ("fee_collector", vec![]),
    ];
    let mut test_env = env::setup(
        balances,
        vec!["nami".to_string(), "auto".to_string(), "lqdy".to_string()],
    );

    // populate swap mocks so that fair price is 1 for everyone
    let owner = test_env.app.api().addr_make("owner");

    //let fee_collector = test_env.app.api().addr_make("fee_collector");

    test_env.fin_pairs.iter().for_each(|(denom, swap_mock)| {
        // if denom=="lqdy"{
        //     lqdy_contract=swap_mock.address.clone();
        // }
        swap_mock
            .populate_orderbook(
                &mut test_env.app,
                &owner.clone(),
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

    let (_lqdy_token, lqdy_mocked_fin) = &test_env.fin_pairs[2];

    //Successful swap
    let resp = test_env.swap.execute_swap(
        &mut test_env.app,
        "user",
        coins(1_700_000_000u128, "eth-usdc"),
        coin(16_983_001u128, "lqdy"),
        vec![Stage {
            address: lqdy_mocked_fin.address.clone(),
            denom: "eth-usdc".to_string(),
        }],
        Some("ruji".to_string()), //None,//
        None,
    );

    // queries
    // let resp = test_env
    //     .swap
    //     .query_config(&mut test_env.app)
    //     .unwrap();
    //     assert_eq!(resp.fee_bps,10u16
    // );
    assert!(resp.is_err());
    assert!(resp
        .unwrap_err()
        .root_cause()
        .to_string()
        .contains("Invalid return amount 16983000 expected 16983001"));
}

#[test]
fn update_config() {
    let balances = vec![
        (
            "user",
            vec![
                coin(10_000_000_000, "nami"),
                coin(10_000_000_000, "auto"),
                coin(10_000_000_000, "lqdy"),
                coin(10_000_000_000, "eth-usdc"),
            ],
        ),
        (
            "owner",
            vec![
                coin(100_000_000_000, "nami"),
                coin(100_000_000_000, "auto"),
                coin(100_000_000_000, "lqdy"),
                coin(500_000_000_000, "eth-usdc"),
            ],
        ),
        ("fee_collector", vec![]),
    ];
    let mut test_env = env::setup(
        balances,
        vec!["nami".to_string(), "auto".to_string(), "lqdy".to_string()],
    );
    //let (_nami_token,nami_mocked_fin )=&test_env.fin_pairs[0];
    //let (_auto_token,auto_mocked_fin )=&test_env.fin_pairs[1];
    let (_lqdy_token, lqdy_mocked_fin) = &test_env.fin_pairs[2];

    let owner = test_env.app.api().addr_make("owner");
    //let user = test_env.app.api().addr_make("user");
    let fee_collector = test_env.app.api().addr_make("fee_collector");

    let mut usdc_balance_owner = test_env.app.query_balance("owner", "eth-usdc", true);
    assert_eq!(usdc_balance_owner, Uint128::from(500_000_000_000u128));

    let mut swap_config = test_env.swap.query_config(&mut test_env.app).unwrap();

    assert_eq!(swap_config.fee_bps, 10u16);
    assert_eq!(
        swap_config.fee_collector.to_string(),
        fee_collector.to_string()
    );

    test_env
        .swap
        .sudo_execute_config_fee_collector_update(&mut test_env.app, owner.to_string().clone())
        .unwrap();
    swap_config = test_env.swap.query_config(&mut test_env.app).unwrap();

    assert_eq!(swap_config.fee_collector.to_string(), owner.to_string());

    test_env
        .swap
        .sudo_execute_config_fee_bps_update(&mut test_env.app, 20u16)
        .unwrap();
    swap_config = test_env.swap.query_config(&mut test_env.app).unwrap();
    assert_eq!(swap_config.fee_bps, 20u16);

    test_env.fin_pairs.iter().for_each(|(denom, swap_mock)| {
        swap_mock
            .populate_orderbook(
                &mut test_env.app,
                &owner.clone(),
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

    usdc_balance_owner = test_env.app.query_balance("owner", "eth-usdc", true);
    assert_eq!(usdc_balance_owner, Uint128::from(491_000_000_000u128));

    //Successful swap
    test_env
        .swap
        .execute_swap(
            &mut test_env.app,
            "user",
            coins(17_000_000u128, "lqdy"),
            coin(16830000u128, "eth-usdc"),
            vec![Stage {
                address: lqdy_mocked_fin.address.clone(),
                denom: "lqdy".to_string(),
            }],
            None,
            None,
        )
        .unwrap();

    let lqdy_balance_user = test_env.app.query_balance("user", "lqdy", true);
    assert_eq!(
        lqdy_balance_user,
        Uint128::from(10_000_000_000u128 - 17_000_000u128)
    );
    let usdc_balance_user = test_env.app.query_balance("user", "eth-usdc", true);
    assert_eq!(
        usdc_balance_user,
        Uint128::from(10_000_000_000u128 + 1_672_748_720u128)
    );
    let usdc_balance_owner = test_env.app.query_balance("owner", "eth-usdc", true);
    assert_eq!(
        usdc_balance_owner,
        Uint128::from(491_000_000_000u128 + 3_352_202u128)
    );
}

#[test]
fn affiliate_admin() {
    let mut test_env = env::setup(vec![], vec![]);

    let ruji = test_env.app.api().addr_make("ruji");
    let affiliates_list_resp = test_env
        .swap
        .query_affiliates(&mut test_env.app, None, None)
        .unwrap();
    assert_eq!(affiliates_list_resp.affiliates.len(), 0);

    test_env
        .swap
        .sudo_execute_add_affiliate(
            &mut test_env.app,
            Affiliate {
                affiliate_code: "ruji".to_string(),
                referral_fee_share_bps: 10u16,
                affiliate_fee_bps: 0u16,
                referral_payment_address: ruji.to_string().clone(),
                affiliate_payment_address: ruji.to_string().clone(),
            },
        )
        .unwrap();

    let affiliates_list_resp = test_env
        .swap
        .query_affiliates(&mut test_env.app, None, None)
        .unwrap();
    let (affiliate_key, affiliate) = &affiliates_list_resp.affiliates[0];

    assert_eq!(affiliates_list_resp.affiliates.len(), 1);
    let ruji_affiliate = test_env
        .swap
        .query_affiliate(&mut test_env.app, "ruji".to_string())
        .unwrap();
    assert_eq!(ruji_affiliate.affiliate.affiliate_code, "ruji");
    assert_eq!(ruji_affiliate.affiliate.referral_fee_share_bps, 10u16);
    assert_eq!(ruji_affiliate.affiliate.affiliate_fee_bps, 0u16);
    assert_eq!(
        ruji_affiliate.affiliate.referral_payment_address,
        ruji.to_string()
    );
    assert_eq!(
        ruji_affiliate.affiliate.affiliate_payment_address,
        ruji.to_string()
    );

    //Update an existing
    test_env
        .swap
        .sudo_execute_add_affiliate(
            &mut test_env.app,
            Affiliate {
                affiliate_code: "ruji".to_string(),
                referral_fee_share_bps: 15u16,
                affiliate_fee_bps: 10u16,
                referral_payment_address: ruji.to_string().clone(),
                affiliate_payment_address: ruji.to_string().clone(),
            },
        )
        .unwrap();
    let ruji_affiliate = test_env
        .swap
        .query_affiliate(&mut test_env.app, "ruji".to_string())
        .unwrap();

    assert_eq!(ruji_affiliate.affiliate.referral_fee_share_bps, 15u16);
    assert_eq!(ruji_affiliate.affiliate.affiliate_fee_bps, 10u16);

    test_env
        .swap
        .sudo_execute_remove_affiliate(&mut test_env.app, "ruji".to_string())
        .unwrap();

    let affiliates_list_resp = test_env
        .swap
        .query_affiliates(&mut test_env.app, None, None)
        .unwrap();
    assert_eq!(affiliates_list_resp.affiliates.len(), 0);
}

#[test]
fn affiliate_admin_affiliate_fee_incorrect() {
    let mut test_env = env::setup(vec![], vec![]);

    let ruji = test_env.app.api().addr_make("ruji");
    let affiliates_list_resp = test_env
        .swap
        .query_affiliates(&mut test_env.app, None, None)
        .unwrap();
    assert_eq!(affiliates_list_resp.affiliates.len(), 0);

    let res = test_env.swap.sudo_execute_add_affiliate(
        &mut test_env.app,
        Affiliate {
            affiliate_code: "ruji".to_string(),
            referral_fee_share_bps: 10u16,
            affiliate_fee_bps: 600u16,
            referral_payment_address: ruji.to_string().clone(),
            affiliate_payment_address: ruji.to_string().clone(),
        },
    );

    assert!(res.is_err());
    assert!(res
        .unwrap_err()
        .root_cause()
        .to_string()
        .contains("Invalid affiliate fee > 500"));
}

#[test]
fn affiliate_admin_referral_fee_incorrect() {
    let mut test_env = env::setup(vec![], vec![]);

    let ruji = test_env.app.api().addr_make("ruji");
    let affiliates_list_resp = test_env
        .swap
        .query_affiliates(&mut test_env.app, None, None)
        .unwrap();
    assert_eq!(affiliates_list_resp.affiliates.len(), 0);

    let res = test_env.swap.sudo_execute_add_affiliate(
        &mut test_env.app,
        Affiliate {
            affiliate_code: "ruji".to_string(),
            referral_fee_share_bps: 10001u16,
            affiliate_fee_bps: 10u16,
            referral_payment_address: ruji.to_string().clone(),
            affiliate_payment_address: ruji.to_string().clone(),
        },
    );

    assert!(res.is_err());
    assert!(res
        .unwrap_err()
        .root_cause()
        .to_string()
        .contains("Invalid referral fee > 10000"));
}

#[test]
fn affiliate_admin_referral_fee_max() {
    let balances = vec![
        (
            "user",
            vec![
                coin(10_000_000_000, "nami"),
                coin(10_000_000_000, "auto"),
                coin(10_000_000_000, "lqdy"),
                coin(10_000_000_000, "eth-usdc"),
            ],
        ),
        (
            "owner",
            vec![
                coin(100_000_000_000, "nami"),
                coin(100_000_000_000, "auto"),
                coin(100_000_000_000, "lqdy"),
                coin(500_000_000_000, "eth-usdc"),
            ],
        ),
        ("fee_collector", vec![]),
    ];
    let mut test_env = env::setup(
        balances,
        vec!["nami".to_string(), "auto".to_string(), "lqdy".to_string()],
    );

    // populate swap mocks so that fair price is 1 for everyone
    let owner = test_env.app.api().addr_make("owner");

    //let fee_collector = test_env.app.api().addr_make("fee_collector");

    test_env.fin_pairs.iter().for_each(|(denom, swap_mock)| {
        // if denom=="lqdy"{
        //     lqdy_contract=swap_mock.address.clone();
        // }
        swap_mock
            .populate_orderbook(
                &mut test_env.app,
                &owner.clone(),
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

    let (_lqdy_token, lqdy_mocked_fin) = &test_env.fin_pairs[2];

    //Add affiliate with 100% share of fee
    let ruji = test_env.app.api().addr_make("ruji");
    let affiliates_list_resp = test_env
        .swap
        .query_affiliates(&mut test_env.app, None, None)
        .unwrap();
    assert_eq!(affiliates_list_resp.affiliates.len(), 0);

    let res = test_env.swap.sudo_execute_add_affiliate(
        &mut test_env.app,
        Affiliate {
            affiliate_code: "ruji".to_string(),
            referral_fee_share_bps: 10000u16,
            affiliate_fee_bps: 0u16,
            referral_payment_address: ruji.to_string().clone(),
            affiliate_payment_address: ruji.to_string().clone(),
        },
    );

    let ruji_affiliate = test_env
        .swap
        .query_affiliate(&mut test_env.app, "ruji".to_string())
        .unwrap();

    assert_eq!(ruji_affiliate.affiliate.referral_fee_share_bps, 10000u16);
    assert_eq!(ruji_affiliate.affiliate.affiliate_fee_bps, 0u16);

    //Successful swap
    test_env
        .swap
        .execute_swap(
            &mut test_env.app,
            "user",
            coins(1_700_000_000u128, "eth-usdc"),
            coin(1u128, "lqdy"),
            vec![Stage {
                address: lqdy_mocked_fin.address.clone(),
                denom: "eth-usdc".to_string(),
            }],
            Some("ruji".to_string()), //None,//
            None,
        )
        .unwrap();

    let user = test_env.app.api().addr_make("user");

    let lqdy_balance_user = test_env.app.query_balance("user", "lqdy", true);

    assert_eq!(
        lqdy_balance_user,
        Uint128::from(10_000_000_000u128 + 17_000_000u128 - 17_000u128)
    );
    let lqdy_balance_fee_collector = test_env.app.query_balance("fee_collector", "lqdy", true);
    assert_eq!(lqdy_balance_fee_collector, Uint128::from(0u128));

    let lqdy_balance_affiliate_fee_collector = test_env.app.query_balance("ruji", "lqdy", true);
    assert_eq!(
        lqdy_balance_affiliate_fee_collector,
        Uint128::from(17_000u128)
    );

    let usdc_balance = test_env.app.query_balance("user", "eth-usdc", true);
    assert_eq!(
        usdc_balance,
        Uint128::from(10_000_000_000u128 - 1_700_000_000u128)
    );
}

#[test]
fn affiliate_admin_only_affiliate_fee() {
    let balances = vec![
        (
            "user",
            vec![
                coin(10_000_000_000, "nami"),
                coin(10_000_000_000, "auto"),
                coin(10_000_000_000, "lqdy"),
                coin(10_000_000_000, "eth-usdc"),
            ],
        ),
        (
            "owner",
            vec![
                coin(100_000_000_000, "nami"),
                coin(100_000_000_000, "auto"),
                coin(100_000_000_000, "lqdy"),
                coin(500_000_000_000, "eth-usdc"),
            ],
        ),
        ("fee_collector", vec![]),
    ];
    let mut test_env = env::setup(
        balances,
        vec!["nami".to_string(), "auto".to_string(), "lqdy".to_string()],
    );

    // populate swap mocks so that fair price is 1 for everyone
    let owner = test_env.app.api().addr_make("owner");

    //let fee_collector = test_env.app.api().addr_make("fee_collector");

    test_env.fin_pairs.iter().for_each(|(denom, swap_mock)| {
        // if denom=="lqdy"{
        //     lqdy_contract=swap_mock.address.clone();
        // }
        swap_mock
            .populate_orderbook(
                &mut test_env.app,
                &owner.clone(),
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

    let (_lqdy_token, lqdy_mocked_fin) = &test_env.fin_pairs[2];

    //Add affiliate with 100% share of fee
    let ruji = test_env.app.api().addr_make("ruji");
    let affiliates_list_resp = test_env
        .swap
        .query_affiliates(&mut test_env.app, None, None)
        .unwrap();
    assert_eq!(affiliates_list_resp.affiliates.len(), 0);

    let res = test_env.swap.sudo_execute_add_affiliate(
        &mut test_env.app,
        Affiliate {
            affiliate_code: "ruji".to_string(),
            referral_fee_share_bps: 0u16,
            affiliate_fee_bps: 10u16,
            referral_payment_address: ruji.to_string().clone(),
            affiliate_payment_address: ruji.to_string().clone(),
        },
    );

    let ruji_affiliate = test_env
        .swap
        .query_affiliate(&mut test_env.app, "ruji".to_string())
        .unwrap();

    assert_eq!(ruji_affiliate.affiliate.referral_fee_share_bps, 0u16);
    assert_eq!(ruji_affiliate.affiliate.affiliate_fee_bps, 10u16);

    //Successful swap
    test_env
        .swap
        .execute_swap(
            &mut test_env.app,
            "user",
            coins(1_700_000_000u128, "eth-usdc"),
            coin(1u128, "lqdy"),
            vec![Stage {
                address: lqdy_mocked_fin.address.clone(),
                denom: "eth-usdc".to_string(),
            }],
            Some("ruji".to_string()), //None,//
            None,
        )
        .unwrap();

    let user = test_env.app.api().addr_make("user");

    let lqdy_balance_user = test_env.app.query_balance("user", "lqdy", true);

    assert_eq!(
        lqdy_balance_user,
        Uint128::from(10_000_000_000u128 + 17_000_000u128 - 17_000u128 - 17_000u128)
    );
    let lqdy_balance_fee_collector = test_env.app.query_balance("fee_collector", "lqdy", true);
    assert_eq!(lqdy_balance_fee_collector, Uint128::from(17_000u128));

    let lqdy_balance_affiliate_fee_collector = test_env.app.query_balance("ruji", "lqdy", true);
    assert_eq!(
        lqdy_balance_affiliate_fee_collector,
        Uint128::from(17_000u128)
    );

    let usdc_balance = test_env.app.query_balance("user", "eth-usdc", true);
    assert_eq!(
        usdc_balance,
        Uint128::from(10_000_000_000u128 - 1_700_000_000u128)
    );
}

#[test]
fn swap_affiliate_only_referral() {
    let balances = vec![
        (
            "user",
            vec![
                coin(10_000_000_000, "nami"),
                coin(10_000_000_000, "auto"),
                coin(10_000_000_000, "lqdy"),
                coin(20_000_000_000, "eth-usdc"),
            ],
        ),
        (
            "owner",
            vec![
                coin(100_000_000_000, "nami"),
                coin(100_000_000_000, "auto"),
                coin(100_000_000_000, "lqdy"),
                coin(500_000_000_000, "eth-usdc"),
            ],
        ),
        ("fee_collector", vec![]),
        ("ruji", vec![]),
    ];
    let mut test_env = env::setup(
        balances,
        vec!["nami".to_string(), "auto".to_string(), "lqdy".to_string()],
    );
    //let (_nami_token, nami_mocked_fin) = &test_env.fin_pairs[0];
    //let (_auto_token,auto_mocked_fin )=&test_env.fin_pairs[1];
    let (_lqdy_token, lqdy_mocked_fin) = &test_env.fin_pairs[2];

    // populate swap mocks so that fair price is 1 for everyone
    let owner = test_env.app.api().addr_make("owner");
    let user = test_env.app.api().addr_make("user");
    let ruji = test_env.app.api().addr_make("ruji");
    //let fee_collector = test_env.app.api().addr_make("fee_collector");

    test_env.fin_pairs.iter().for_each(|(denom, swap_mock)| {
        swap_mock
            .populate_orderbook(
                &mut test_env.app,
                &owner.clone(),
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

    test_env
        .swap
        .sudo_execute_add_affiliate(
            &mut test_env.app,
            Affiliate {
                affiliate_code: "ruji".to_string(),
                referral_fee_share_bps: 1000u16,
                affiliate_fee_bps: 0u16,
                referral_payment_address: ruji.to_string().clone(),
                affiliate_payment_address: ruji.to_string().clone(),
            },
        )
        .unwrap();

    //Successful swap
    let res = test_env
        .swap
        .execute_swap(
            &mut test_env.app,
            "user",
            coins(17_000_000_000u128, "eth-usdc"),
            coin(1u128, "lqdy"),
            vec![Stage {
                address: lqdy_mocked_fin.address.clone(),
                denom: "eth-usdc".to_string(),
            }],
            Some("ruji".to_string()), //None,//
            None,
        )
        .unwrap();

    res.assert_event(
        &Event::new("wasm-liquidy-swap/execute").add_attributes(vec![
            ("affiliate", "ruji".to_string()),
            ("denom", "lqdy".to_string()),
            ("volume", "170000000".to_string()),
            ("platform_fee", "153000".to_string()),
            ("referral_fee", "17000".to_string()),
            ("affiliate_fee", "0".to_string()),
        ]),
    );

    let lqdy_balance_user = test_env.app.query_balance("user", "lqdy", true);
    let lqdy_balance_fee_collector = test_env.app.query_balance("fee_collector", "lqdy", true);
    let lqdy_balance_affiliate_ruji = test_env.app.query_balance("ruji", "lqdy", true);

    assert_eq!(
        lqdy_balance_user,
        Uint128::from(10_000_000_000u128 + 170_000_000u128 - 170_000u128)
    );
    assert_eq!(
        lqdy_balance_fee_collector,
        Uint128::from(170_000u128 - 17_000u128)
    );
    assert_eq!(lqdy_balance_affiliate_ruji, Uint128::from(17_000u128));

    let usdc_balance = test_env.app.query_balance("user", "eth-usdc", true);
    assert_eq!(
        usdc_balance,
        Uint128::from(20_000_000_000u128 - 17_000_000_000u128)
    );
}

#[test]
fn swap_affiliate_with_affiliate_fee() {
    let balances = vec![
        (
            "user",
            vec![
                coin(10_000_000_000, "nami"),
                coin(10_000_000_000, "auto"),
                coin(10_000_000_000, "lqdy"),
                coin(20_000_000_000, "eth-usdc"),
            ],
        ),
        (
            "owner",
            vec![
                coin(100_000_000_000, "nami"),
                coin(100_000_000_000, "auto"),
                coin(100_000_000_000, "lqdy"),
                coin(500_000_000_000, "eth-usdc"),
            ],
        ),
        ("fee_collector", vec![]),
        ("ruji", vec![]),
    ];
    let mut test_env = env::setup(
        balances,
        vec!["nami".to_string(), "auto".to_string(), "lqdy".to_string()],
    );
    //let (_nami_token, nami_mocked_fin) = &test_env.fin_pairs[0];
    //let (_auto_token,auto_mocked_fin )=&test_env.fin_pairs[1];
    let (_lqdy_token, lqdy_mocked_fin) = &test_env.fin_pairs[2];

    // populate swap mocks so that fair price is 1 for everyone
    let owner = test_env.app.api().addr_make("owner");
    let user = test_env.app.api().addr_make("user");
    let ruji = test_env.app.api().addr_make("ruji");
    //let fee_collector = test_env.app.api().addr_make("fee_collector");

    test_env.fin_pairs.iter().for_each(|(denom, swap_mock)| {
        swap_mock
            .populate_orderbook(
                &mut test_env.app,
                &owner.clone(),
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

    test_env
        .swap
        .sudo_execute_add_affiliate(
            &mut test_env.app,
            Affiliate {
                affiliate_code: "ruji".to_string(),
                referral_fee_share_bps: 1000u16,
                affiliate_fee_bps: 10u16,
                referral_payment_address: ruji.to_string().clone(),
                affiliate_payment_address: ruji.to_string().clone(),
            },
        )
        .unwrap();

    //Successful swap
    test_env
        .swap
        .execute_swap(
            &mut test_env.app,
            "user",
            coins(17_000_000_000u128, "eth-usdc"),
            coin(1u128, "lqdy"),
            vec![Stage {
                address: lqdy_mocked_fin.address.clone(),
                denom: "eth-usdc".to_string(),
            }],
            Some("ruji".to_string()), //None,//
            None,
        )
        .unwrap();

    let lqdy_balance_user = test_env.app.query_balance("user", "lqdy", true);
    let lqdy_balance_fee_collector = test_env.app.query_balance("fee_collector", "lqdy", true);
    let lqdy_balance_affiliate_ruji = test_env.app.query_balance("ruji", "lqdy", true);

    assert_eq!(
        lqdy_balance_user,
        Uint128::from(10_000_000_000u128 + 170_000_000u128 - 170_000u128 - 170_000u128)
    );
    assert_eq!(
        lqdy_balance_fee_collector,
        Uint128::from(170_000u128 - 17_000u128)
    );
    assert_eq!(
        lqdy_balance_affiliate_ruji,
        Uint128::from(17_000u128 + 170_000u128)
    );

    let usdc_balance = test_env.app.query_balance("user", "eth-usdc", true);
    assert_eq!(
        usdc_balance,
        Uint128::from(20_000_000_000u128 - 17_000_000_000u128)
    );
}

#[test]
fn swap_affiliate_with_affiliate_fee_two_addresses() {
    let balances = vec![
        (
            "user",
            vec![
                coin(10_000_000_000, "nami"),
                coin(10_000_000_000, "auto"),
                coin(10_000_000_000, "lqdy"),
                coin(20_000_000_000, "eth-usdc"),
            ],
        ),
        (
            "owner",
            vec![
                coin(100_000_000_000, "nami"),
                coin(100_000_000_000, "auto"),
                coin(100_000_000_000, "lqdy"),
                coin(500_000_000_000, "eth-usdc"),
            ],
        ),
        ("fee_collector", vec![]),
        ("ruji", vec![]),
    ];
    let mut test_env = env::setup(
        balances,
        vec!["nami".to_string(), "auto".to_string(), "lqdy".to_string()],
    );
    //let (_nami_token, nami_mocked_fin) = &test_env.fin_pairs[0];
    //let (_auto_token,auto_mocked_fin )=&test_env.fin_pairs[1];
    let (_lqdy_token, lqdy_mocked_fin) = &test_env.fin_pairs[2];

    // populate swap mocks so that fair price is 1 for everyone
    let owner = test_env.app.api().addr_make("owner");
    let user = test_env.app.api().addr_make("user");
    let ruji_referral = test_env.app.api().addr_make("ruji_referral");
    let ruji_affiliate = test_env.app.api().addr_make("ruji_affiliate");
    //let fee_collector = test_env.app.api().addr_make("fee_collector");

    test_env.fin_pairs.iter().for_each(|(denom, swap_mock)| {
        swap_mock
            .populate_orderbook(
                &mut test_env.app,
                &owner.clone(),
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

    test_env
        .swap
        .sudo_execute_add_affiliate(
            &mut test_env.app,
            Affiliate {
                affiliate_code: "ruji".to_string(),
                referral_fee_share_bps: 1000u16,
                affiliate_fee_bps: 10u16,
                referral_payment_address: ruji_referral.to_string().clone(),
                affiliate_payment_address: ruji_affiliate.to_string().clone(),
            },
        )
        .unwrap();

    //Successful swap
    test_env
        .swap
        .execute_swap(
            &mut test_env.app,
            "user",
            coins(17_000_000_000u128, "eth-usdc"),
            coin(1u128, "lqdy"),
            vec![Stage {
                address: lqdy_mocked_fin.address.clone(),
                denom: "eth-usdc".to_string(),
            }],
            Some("ruji".to_string()), //None,//
            None,
        )
        .unwrap();

    let lqdy_balance_user = test_env.app.query_balance("user", "lqdy", true);
    let lqdy_balance_fee_collector = test_env.app.query_balance("fee_collector", "lqdy", true);
    let lqdy_balance_ruji_referral = test_env.app.query_balance("ruji_referral", "lqdy", true);
    let lqdy_balance_ruji_affiiate = test_env.app.query_balance("ruji_affiliate", "lqdy", true);

    assert_eq!(
        lqdy_balance_user,
        Uint128::from(10_000_000_000u128 + 170_000_000u128 - 170_000u128 - 170_000u128)
    );
    assert_eq!(
        lqdy_balance_fee_collector,
        Uint128::from(170_000u128 - 17_000u128)
    );
    assert_eq!(lqdy_balance_ruji_referral, Uint128::from(17_000u128));
    assert_eq!(lqdy_balance_ruji_affiiate, Uint128::from(170_000u128));

    let usdc_balance = test_env.app.query_balance("user", "eth-usdc", true);
    assert_eq!(
        usdc_balance,
        Uint128::from(20_000_000_000u128 - 17_000_000_000u128)
    );
}

#[test]
fn affiliate_list() {
    let mut test_env = env::setup(vec![], vec![]);

    let ruji = test_env.app.api().addr_make("ruji");
    let affiliates_list_resp = test_env
        .swap
        .query_affiliates(&mut test_env.app, None, None)
        .unwrap();
    assert_eq!(affiliates_list_resp.affiliates.len(), 0);

    test_env
        .swap
        .sudo_execute_add_affiliate(
            &mut test_env.app,
            Affiliate {
                affiliate_code: "ruji".to_string(),
                referral_fee_share_bps: 10u16,
                affiliate_fee_bps: 0u16,
                referral_payment_address: ruji.to_string().clone(),
                affiliate_payment_address: ruji.to_string().clone(),
            },
        )
        .unwrap();

    test_env
        .swap
        .sudo_execute_add_affiliate(
            &mut test_env.app,
            Affiliate {
                affiliate_code: "ruji2".to_string(),
                referral_fee_share_bps: 10u16,
                affiliate_fee_bps: 0u16,
                referral_payment_address: ruji.to_string().clone(),
                affiliate_payment_address: ruji.to_string().clone(),
            },
        )
        .unwrap();
    test_env
        .swap
        .sudo_execute_add_affiliate(
            &mut test_env.app,
            Affiliate {
                affiliate_code: "ruji3".to_string(),
                referral_fee_share_bps: 10u16,
                affiliate_fee_bps: 0u16,
                referral_payment_address: ruji.to_string().clone(),
                affiliate_payment_address: ruji.to_string().clone(),
            },
        )
        .unwrap();

    let affiliates_list_resp = test_env
        .swap
        .query_affiliates(&mut test_env.app, Some(1u8), None)
        .unwrap();
    let (affiliate_key, affiliate) = &affiliates_list_resp.affiliates[0];
    assert_eq!(affiliate_key, "ruji");

    assert_eq!(affiliates_list_resp.affiliates.len(), 1);

    let affiliates_list_resp = test_env
        .swap
        .query_affiliates(&mut test_env.app, Some(1u8), Some("ruji".to_string()))
        .unwrap();
    let (affiliate_key, affiliate) = &affiliates_list_resp.affiliates[0];

    assert_eq!(affiliates_list_resp.affiliates.len(), 1);
    assert_eq!(affiliate_key, "ruji2");
}
