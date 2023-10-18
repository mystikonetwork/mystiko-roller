use mystiko_crypto::zkp::proof::{G1Point, G2Point, Proof, ZKProof};
use mystiko_protocol::rollup::RollupProof;

pub fn mock_proof_data() -> RollupProof {
    RollupProof {
        zk_proof: ZKProof {
            proof: Proof {
                a: G1Point {
                    x: "0x29fbefd6cb599c09888dd65220052e90f90e748e48cbd56162b916e865853772".to_string(),
                    y: "0x1c6e2baeeb85f42516e5ebdbcb83a2fce361c20bfd62e68dfcfe0e923330b413".to_string(),
                },
                b: G2Point {
                    x: [
                        "0x2c56f5e4cb7a464f718514dd868f5f98a5d620250669b941b85a499342f63540".to_string(),
                        "0x14cb5c1abd619455933aeed2553a7c099c579291e509f393ac2de135482c4bb0".to_string(),
                    ],
                    y: [
                        "0x1fe3a4f88757932d6322c4e818463c2bb0ee6abd3122f06d94611dacde460ffa".to_string(),
                        "0x1e4841ca07a44d756c0791c6e18e260953755d0a9a2b78c9a0f253e76d2c1689".to_string(),
                    ],
                },
                c: G1Point {
                    x: "0x11da97a1df5fba3cf1e3dcb9502ec4565b256c42a162fffa945a0e9900ba2976".to_string(),
                    y: "0x0a7382541ea8171ff5e7c40ba4b7ee9671aa007282859bd18a259a6796813018".to_string(),
                },
            },
            inputs: vec![
                "0x22c21d5862b87489724035886342e313087df95a70b0bd042ef03c638a0bf46c".to_string(),
                "0x1e75e854260840f462b3de02fd4a96195fe5bce0fe4dbd435c36d48f13f006d6".to_string(),
                "0x05d3245056163cdb31f2b57bfce5a7a65cf86d4feb0fba04c0ad9c1dafcd3c20".to_string(),
                "0x0000000000000000000000000000000000000000000000000000000000000001".to_string(),
            ],
        },
        new_root: Default::default(),
        leaves_hash: Default::default(),
    }
}
