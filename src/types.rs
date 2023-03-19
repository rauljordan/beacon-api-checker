use beacon_api_client::{BalanceSummary, ValidatorStatus, ValidatorSummary};
use ethereum_consensus::capella::mainnet::SignedBeaconBlock;

pub struct SignedBeaconBlockExt {
    pub inner: SignedBeaconBlock,
}

impl PartialEq for SignedBeaconBlockExt {
    fn eq(&self, other: &Self) -> bool {
        return true;
    }
}

pub struct ValidatorSummaryExt {
    pub inner: ValidatorSummary,
}

impl PartialEq for ValidatorSummaryExt {
    fn eq(&self, other: &Self) -> bool {
        if self.inner.index != other.inner.index {
            return false;
        }
        if self.inner.balance != other.inner.balance {
            return false;
        }
        let s1 = ValidatorStatusExt {
            inner: self.inner.status,
        };
        let s2 = ValidatorStatusExt {
            inner: other.inner.status,
        };
        if s1 != s2 {
            return false;
        }
        self.inner.validator == other.inner.validator
    }
}

pub struct ValidatorStatusExt {
    pub inner: ValidatorStatus,
}

impl PartialEq for ValidatorStatusExt {
    fn eq(&self, other: &Self) -> bool {
        let s1 = self.inner.to_string();
        let s2 = other.inner.to_string();
        s1 == s2
    }
}

pub struct BalanceSummaryExt {
    pub inner: BalanceSummary,
}

impl PartialEq for BalanceSummaryExt {
    fn eq(&self, other: &Self) -> bool {
        if self.inner.index != other.inner.index {
            return false;
        }
        self.inner.balance == other.inner.balance
    }
}
