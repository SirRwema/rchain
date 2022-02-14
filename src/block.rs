use super::*;


#[derive(Serialize, Deserilize, Debug, Clone)]
pub struct Block {
    pub id: u64,
    pub timestamp: u64,
    pub hash: String,
    pub previous_hash: String,
    pub data: String,
    pub nonce: u64,
}

impl Block{

    pub fn new(id: u64, previous_hash: String, data: String) -> Self {
        let time = now.timestamp();
        let (nonce, hash) = mine_block(id, now.timestamp(), &previous_hash, &data);
        Self {
            id,
            hash,
            timestamp: time,
            previous_hash,
            data,
            nonce,
        }
    }

    fn mine_block(id: u64, timestamp: i64, previous_hash: &str, data: &str) -> (u64, String){
        info!("mining block...");
        let mut nonce = 0;

        loop {
            if nonce % 10000 == 0 {
                info!("nonce: {}", nonce);
            }
            let hash = calculate_hash(id, timestamp, previous_hash, data, nonce);
            let binary_hash = hash_to_binary_representation(&hash);
            if binary_hash.starts_with(DIFFICULTY_PREFIX){
                info!(
                    "mined! nonce: {}, hash: {}",
                    nonce,
                    hex::encode(&hash),
                    binary_hash
                );
                return (nonce, hex::encode(hash));
            }

            nonce += 1;
        }
    }

}