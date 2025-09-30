use cosmwasm_std::{Event, Uint128};

pub fn execute_event(
    denom: String,
    volume: &Uint128,
    platform_fee: &Uint128,
    affiliate: Option<String>,
    referral_fee: &Uint128,
    affiliate_fee: &Uint128,
) -> Event {
    let mut event = Event::new(format!("{}/execute", env!("CARGO_PKG_NAME")))
        .add_attribute("denom", denom)
        .add_attribute("volume", volume.to_string())
        .add_attribute("platform_fee", platform_fee.to_string());
    if let Some(key) = affiliate {
        event = event
            .add_attribute("affiliate", key)
            .add_attribute("referral_fee", referral_fee.to_string())
            .add_attribute("affiliate_fee", affiliate_fee.to_string())
    } else {
        event = event
            .add_attribute("affiliate", "")
            .add_attribute("referral_fee", "0")
            .add_attribute("affiliate_fee", "0")
    }
    event
}
