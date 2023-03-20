use beacon_api_client::{
    BalanceSummary, BlockId, FinalityCheckpoints, StateId, ValidatorStatus, ValidatorSummary,
};

pub struct StateIdExt {
    pub inner: StateId,
}

impl Clone for StateIdExt {
    fn clone(&self) -> Self {
        match self.inner {
            StateId::Genesis => StateIdExt {
                inner: StateId::Genesis,
            },
            StateId::Finalized => StateIdExt {
                inner: StateId::Finalized,
            },
            StateId::Justified => StateIdExt {
                inner: StateId::Justified,
            },
            StateId::Head => StateIdExt {
                inner: StateId::Head,
            },
            StateId::Slot(x) => StateIdExt {
                inner: StateId::Slot(x),
            },
            StateId::Root(x) => StateIdExt {
                inner: StateId::Root(x),
            },
        }
    }
}

pub struct BlockIdExt {
    pub inner: BlockId,
}

impl Clone for BlockIdExt {
    fn clone(&self) -> Self {
        match self.inner {
            BlockId::Genesis => BlockIdExt {
                inner: BlockId::Genesis,
            },
            BlockId::Finalized => BlockIdExt {
                inner: BlockId::Finalized,
            },
            BlockId::Head => BlockIdExt {
                inner: BlockId::Head,
            },
            BlockId::Slot(x) => BlockIdExt {
                inner: BlockId::Slot(x),
            },
            BlockId::Root(x) => BlockIdExt {
                inner: BlockId::Root(x),
            },
        }
    }
}

pub struct FinalityCheckpointsExt {
    pub inner: FinalityCheckpoints,
}

impl std::fmt::Debug for FinalityCheckpointsExt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FinalityCheckpointsExt")
            .field("previous_justified", &self.inner.previous_justified)
            .field("current_justified", &self.inner.current_justified)
            .field("finalized", &self.inner.finalized)
            .finish()
    }
}

impl Eq for FinalityCheckpointsExt {}

impl PartialEq for FinalityCheckpointsExt {
    fn eq(&self, other: &Self) -> bool {
        if self.inner.previous_justified != other.inner.previous_justified {
            return false;
        }
        if self.inner.current_justified != other.inner.current_justified {
            return false;
        }
        self.inner.finalized == other.inner.finalized
    }
}

#[derive(Debug)]
pub struct ValidatorSummaryExt {
    pub inner: ValidatorSummary,
}

impl Eq for ValidatorSummaryExt {}

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

#[derive(Debug)]
pub struct ValidatorStatusExt {
    pub inner: ValidatorStatus,
}

impl Eq for ValidatorStatusExt {}

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

impl std::fmt::Debug for BalanceSummaryExt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BalanceSummaryExt")
            .field("index", &self.inner.index)
            .field("balance", &self.inner.balance)
            .finish()
    }
}

impl Eq for BalanceSummaryExt {}

impl PartialEq for BalanceSummaryExt {
    fn eq(&self, other: &Self) -> bool {
        if self.inner.index != other.inner.index {
            return false;
        }
        self.inner.balance == other.inner.balance
    }
}
