use crate::state::Status;
use crate::util::mul_bps;
use crate::{config::Config, error::ContractError, events::execute_event};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coins, ensure, to_json_binary, wasm_execute, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut,
    Empty, Env, MessageInfo, QuerierWrapper, Response, StdResult, Uint128,
};
use cw2::set_contract_version;
use liquidy_rs::swap::{
    AffiliateResponse, AffiliatesResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, Stage,
    SudoMsg,
};
use rujira_rs::fin::{self, SimulationResponse, SwapRequest};

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const STATUS: Status = Status::new();

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config::new(deps.api, msg.clone())?;
    config.save(deps.storage)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let config = Config::load(deps.storage)?;

    match msg {
        ExecuteMsg::Swap {
            min_return,
            stages,
            recipient,
            affiliate_code,
            callback,
        } => {
            let mut local_stages = stages;

            match local_stages.pop() {
                None => {
                    // We're done, return balances to sender minus fees
                    let balance = deps
                        .querier
                        .query_balance(env.contract.address.clone(), min_return.denom.clone())?;

                    let base_platform_fee =
                        mul_bps(balance.amount.into(), config.fee_bps).to_uint_ceil();

                    let affiliate = affiliate_code
                        .as_ref()
                        .map(|code| STATUS.get(deps.storage, code))
                        .transpose()?
                        .flatten();

                    let (user_return, platform_fee, referral_fee, affiliate_fee) = match affiliate {
                        Some(ref aff) => {
                            let referral_fee =
                                mul_bps(base_platform_fee.into(), aff.referral_fee_share_bps)
                                    .to_uint_floor();

                            let platform_fee = base_platform_fee.checked_sub(referral_fee)?;

                            let affiliate_fee =
                                mul_bps(balance.amount.into(), aff.affiliate_fee_bps)
                                    .to_uint_floor();
                            let user_return = balance
                                .amount
                                .checked_sub(base_platform_fee)?
                                .checked_sub(affiliate_fee)?;

                            (user_return, platform_fee, referral_fee, affiliate_fee)
                        }
                        None => {
                            let user_return = balance.amount.checked_sub(base_platform_fee)?;
                            (user_return, base_platform_fee, 0u128.into(), 0u128.into())
                        }
                    };

                    let mut resp = Response::new().add_event(execute_event(
                        min_return.denom.clone(),
                        &balance.amount,
                        &platform_fee,
                        affiliate_code,
                        &referral_fee,
                        &affiliate_fee,
                    ));

                    //Check user balance is more than min_return
                    ensure!(
                        user_return >= min_return.amount,
                        ContractError::InvalidReturnAmount {
                            returned: user_return.into(),
                            min_return: min_return.amount.into()
                        }
                    );

                    let recipient =
                        recipient.ok_or(ContractError::Invalid("recipient not set".to_string()))?;
                    match callback {
                        Some(callback) => {
                            resp = resp.add_message(callback.to_message(
                                &recipient,
                                Empty {},
                                coins(user_return.u128(), min_return.denom.clone()),
                            )?)
                        }
                        None => {
                            resp = resp.add_message(CosmosMsg::Bank(BankMsg::Send {
                                to_address: recipient.to_string(),
                                amount: coins(user_return.u128(), min_return.denom.clone()),
                            }));
                        }
                    };

                    if platform_fee > 0u128.into() {
                        resp = resp.add_message(CosmosMsg::Bank(BankMsg::Send {
                            to_address: config.fee_collector.to_string(),
                            amount: coins(platform_fee.u128(), min_return.denom.clone()),
                        }));
                    }
                    if let Some(affiliate) = affiliate {
                        if referral_fee > 0u128.into() {
                            resp = resp.add_message(CosmosMsg::Bank(BankMsg::Send {
                                to_address: affiliate.referral_payment_address.to_string(),
                                amount: coins(referral_fee.u128(), min_return.denom.clone()),
                            }));
                        }
                        if affiliate_fee > 0u128.into() {
                            resp = resp.add_message(BankMsg::Send {
                                to_address: affiliate.affiliate_payment_address.to_string(),
                                amount: coins(affiliate_fee.u128(), min_return.denom.clone()),
                            });
                        }
                    }
                    Ok(resp)
                }
                Some(s) => {
                    let msg = execute_swap(deps.querier, &env, s)?;

                    Ok(Response::default()
                        .add_message(msg)
                        .add_message(wasm_execute(
                            env.contract.address,
                            &ExecuteMsg::Swap {
                                stages: local_stages,
                                min_return: min_return,
                                affiliate_code: affiliate_code.clone(),
                                recipient: Some(recipient.unwrap_or(info.sender.clone())),
                                callback: callback,
                            },
                            vec![],
                        )?))
                }
            }
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, _env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    let mut config = Config::load(deps.storage)?;
    match msg {
        SudoMsg::UpdateFeeCollectorConfig { fee_collector } => {
            config.fee_collector = deps.api.addr_validate(&fee_collector)?;
            config.save(deps.storage)?;
            Ok(Response::default())
        }
        SudoMsg::UpdateFeeBbsConfig { fee_bps } => {
            config.fee_bps = fee_bps;
            config.save(deps.storage)?;
            Ok(Response::default())
        }
        SudoMsg::UpdateMaxAffiliateFeeBbsConfig {
            max_affiliate_fee_bps,
        } => {
            config.max_affiliate_fee_bps = max_affiliate_fee_bps;
            config.save(deps.storage)?;
            Ok(Response::default())
        }
        SudoMsg::AddAffiliate { affiliate } => {
            let _referral_addr = deps
                .api
                .addr_validate(&affiliate.referral_payment_address)?;
            let _affiliate_addr = deps
                .api
                .addr_validate(&affiliate.affiliate_payment_address)?;
            STATUS.add_affiliate(deps.storage, affiliate, config.max_affiliate_fee_bps)?;
            Ok(Response::default())
        }
        SudoMsg::RemoveAffiliate { affiliate_code } => {
            STATUS.remove_affiliate(deps.storage, &affiliate_code)?;
            Ok(Response::default())
        }
    }
    //Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    let config = Config::load(deps.storage)?;
    match msg {
        QueryMsg::Config {} => Ok(to_json_binary(&config)?),
        QueryMsg::Affiliates { limit, start_after } => {
            let limit = limit.unwrap_or(100);

            let affiliates = STATUS.get_affiliates(deps.storage, limit, start_after)?;
            Ok(to_json_binary(&AffiliatesResponse { affiliates })?)
        }
        QueryMsg::Affiliate { affiliate_code } => {
            let affiliate = STATUS.get(deps.storage, &affiliate_code);
            match affiliate {
                Ok(None) => Err(ContractError::Invalid(affiliate_code.to_string())),
                Ok(Some(a)) => Ok(to_json_binary(&AffiliateResponse { affiliate: a })?),
                Err(_) => Err(ContractError::Invalid(affiliate_code.to_string())),
            }
        }
        QueryMsg::Simulate { coin, stages } => {
            let local_stages = stages;
            let result = simulate_recursive(deps.querier, &env, local_stages, coin, &config)?;
            Ok(to_json_binary(&result)?)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}

fn execute_swap(
    querier: QuerierWrapper,
    env: &Env,
    stage: Stage,
) -> Result<CosmosMsg, ContractError> {
    let balance = querier.query_balance(env.contract.address.clone(), stage.denom)?;
    ensure!(
        balance.amount.gt(&Uint128::zero()),
        ContractError::NoBalance {}
    );
    let msg = wasm_execute(
        stage.address,
        &fin::ExecuteMsg::Swap(SwapRequest {
            min_return: None,
            to: None,
            callback: None,
        }),
        coins(balance.amount.u128(), balance.denom.clone()),
    )?;
    Ok(msg.into())
}

fn simulate_recursive(
    querier: QuerierWrapper,
    env: &Env,
    mut stages: Vec<Stage>,
    coin: Coin,
    config: &Config,
) -> StdResult<SimulationResponse> {
    match stages.pop() {
        None => {
            // No more stages, deduct base platform fee and return the coin amount
            let base_platform_fee = mul_bps(coin.amount.into(), config.fee_bps).to_uint_ceil();
            let user_return = coin.amount.checked_sub(base_platform_fee)?;

            Ok(SimulationResponse {
                returned: user_return,
                fee: base_platform_fee,
            })
        }
        Some(stage) => {
            // Simulate current stage
            let result = simulate_swap(querier, env, stage, coin)?;

            // If no more stages, deduct base platform fee and return result, otherwise continue recursively
            if stages.is_empty() {
                let base_platform_fee =
                    mul_bps(result.returned.into(), config.fee_bps).to_uint_ceil();
                let user_return = result.returned.checked_sub(base_platform_fee)?;

                Ok(SimulationResponse {
                    returned: user_return,
                    fee: base_platform_fee,
                })
            } else {
                // Create next coin for remaining stages
                // The next stage's input denom is determined by the next stage in the vector
                // Since we're processing in reverse order, we need to look at the next stage's denom
                let next_stage_denom = stages.last().unwrap().denom.clone();
                let next_coin = Coin {
                    denom: next_stage_denom,
                    amount: result.returned,
                };
                simulate_recursive(querier, env, stages, next_coin, config)
            }
        }
    }
}

fn simulate_swap(
    querier: QuerierWrapper,
    _env: &Env,
    stage: Stage,
    coin: Coin,
) -> StdResult<SimulationResponse> {
    // Query the swap contract to simulate the swap

    let simulation_response: SimulationResponse =
        querier.query_wasm_smart(stage.address, &rujira_rs::fin::QueryMsg::Simulate(coin))?;

    Ok(simulation_response)
}

#[cfg(test)]
mod tests {}
