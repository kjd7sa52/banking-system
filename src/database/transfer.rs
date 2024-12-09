use crate::transport::record::Amount;

#[derive(Debug, derive_new::new)]
pub struct Transfer {
    pub amount: Amount,
    pub disputed: bool,
}
