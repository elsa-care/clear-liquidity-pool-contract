use soroban_sdk::{Address, Map};

pub(crate) const ONE_XLM_IN_STROOPS: i64 = 10_000_000;

fn calculate_percentage(amount: &i128, total_balance: &i128) -> i64 {
    ((*amount * 100 * ONE_XLM_IN_STROOPS as i128) / *total_balance) as i64
}

fn recalculate_percentage(lender_percentage: i64, old_balance: i128, total_balance: &i128) -> i64 {
    (lender_percentage * old_balance as i64 / *total_balance as i64) as i64
}

pub fn process_lender_contribution(
    length: u32,
    lender: &Address,
    mut lender_contribution: Map<Address, i64>,
    lender_balance: &i128,
    total_balance: &i128,
    old_balance: &i128,
) -> Map<Address, i64> {
    if lender_contribution.len() == length {
        let percentage = calculate_percentage(&lender_balance, &total_balance);
        lender_contribution.set(lender.clone(), percentage)
    } else {
        for (address, percentage) in lender_contribution.iter() {
            let new_percentage = recalculate_percentage(percentage, *old_balance, &total_balance);
            lender_contribution.set(address.clone(), new_percentage);
        }
        let percentage = calculate_percentage(&lender_balance, &total_balance);
        lender_contribution.set(lender.clone(), percentage)
    }
    lender_contribution
}
