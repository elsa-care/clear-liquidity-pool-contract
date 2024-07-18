use soroban_sdk::{Address, Env, Map, Vec};

use crate::errors::LPError;
use crate::storage::read_lender;

type ContributionsMap = Map<Address, i64>;
type LenderAmountMap = Map<Address, i128>;
type ContributionResult = Result<(ContributionsMap, LenderAmountMap), LPError>;

pub(crate) const ONE_XLM_IN_STROOPS: i64 = 10_000_000;

fn calculate_percentage(amount: &i128, total_balance: &i128) -> i64 {
    ((*amount * 100 * ONE_XLM_IN_STROOPS as i128) / *total_balance) as i64
}

pub fn calculate_new_lender_amount(
    loan_amount: &i128,
    lender_balance: &i128,
    percentage: i64,
) -> i128 {
    lender_balance - (loan_amount * percentage as i128 / (100 * ONE_XLM_IN_STROOPS as i128))
}

pub fn calculate_repayment_amount(amount: i128, percentage: i64) -> i128 {
    (amount * percentage as i128) / (100 * ONE_XLM_IN_STROOPS as i128)
}

pub fn process_lender_contribution(
    env: &Env,
    contributions: Vec<Address>,
    loan_amount: &i128,
    total_balance: &i128,
) -> ContributionResult {
    let mut lender_contributions = Map::new(env);
    let mut new_lender_amounts = Map::new(env);

    for address in contributions.iter() {
        let lender = read_lender(env, &address)?;

        if lender.active {
            let percentage = calculate_percentage(&lender.balance, total_balance);
            let new_lender_amount =
                calculate_new_lender_amount(loan_amount, &lender.balance, percentage);
            lender_contributions.set(address.clone(), percentage);
            new_lender_amounts.set(address.clone(), new_lender_amount);
        }
    }
    Ok((lender_contributions, new_lender_amounts))
}
