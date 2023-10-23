use cosmwasm_std::{Timestamp, Uint128, StdResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Claim {
    pub amount: Uint128,
    pub release: Timestamp
}

impl Claim {
    
    pub fn is_released(&self, now: Timestamp) -> bool {
        now.cmp(&self.release).is_ge()
    }

}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct Claims(Vec<Claim>);

impl Claims {
    
    pub fn new() -> Self {
        
        Claims(Vec::new())
    
    }

    pub fn total(&self) -> StdResult<Uint128> {
        
        self.clone()
            .0
            .into_iter()
            .try_fold(Uint128::from(0u64), |sum: Uint128, c: Claim| -> StdResult<Uint128> {
                Ok(sum.checked_add(c.amount)?)
            })
    
    }

    pub fn add(&mut self, other: Claim) {

        self.0.push(other)

    }

    

}

impl IntoIterator for Claims {
    type Item = Claim;
    type IntoIter = std::vec::IntoIter<Claim>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Extend<Claim> for Claims {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = Claim>,
    {
        self.0.extend(iter);
    }
}