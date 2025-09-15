use cosmwasm_std::Decimal;
use std::ops::Mul;
pub fn mul_bps(amount: u128, bps: u16) -> Decimal {
    Decimal::from_ratio(amount, 1u128).mul(Decimal::from_ratio(bps, 10000u16))
}

#[test]
fn bps_mul() {
    let base_fee = mul_bps(100_000_000u128, 10u16).to_uint_ceil();
    assert_eq!(base_fee.u128(), 100_000u128);

    let base_fee = mul_bps(100_000_000u128, 15u16).to_uint_ceil();
    assert_eq!(base_fee.u128(), 150_000u128);

    let base_fee = mul_bps(100_000_000u128, 1u16).to_uint_ceil();
    assert_eq!(base_fee.u128(), 10_000u128);

    let base_fee = mul_bps(123_456_789u128, 10u16).to_uint_ceil();
    assert_eq!(base_fee.u128(), 123_457u128);

    let base_fee = mul_bps(123_456_789u128, 10u16).to_uint_floor();
    assert_eq!(base_fee.u128(), 123_456u128);
}
