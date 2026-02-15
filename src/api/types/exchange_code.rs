use serde::{Deserialize, Serialize};

/// Exchange code response from the web-based SID auth flow.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExchangeCode {
    pub code: Option<String>,
}
