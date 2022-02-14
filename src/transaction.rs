use super::*;
use ed25519_dalek::PublicKey;

#[derive(Debug)]
pub struct Transaction {
    pub sender: Option<PublicKey>,
    pub receiver: Option<PublicKey>,
    pub amount: f32,
}

impl Transaction {

    pub fn bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        if let Some(sender) = self.sender {
            bytes.extend(sender.as_bytes());
        }
        if let Some(receiver) = self.receiver{
            bytes.extend(receiver.as_bytes());
        }
        bytes.extend(&self.amount.to_bits().to_ne_bytes());

        bytes
    }


}
