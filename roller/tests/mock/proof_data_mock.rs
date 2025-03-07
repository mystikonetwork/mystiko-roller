use mystiko_crypto::zkp::G16Proof;
use mystiko_fs::read_file_bytes;
use mystiko_protocol::rollup::RollupProof;

pub async fn mock_proof_data() -> RollupProof<G16Proof> {
    let proof = read_file_bytes("./tests/test_files/zkp/proof.json").await.unwrap();
    let proof: serde_json::Value = serde_json::from_reader(proof.as_slice()).unwrap();
    let proof = G16Proof::from_json_string(&proof.to_string()).unwrap();
    RollupProof {
        zk_proof: proof,
        new_root: Default::default(),
        leaves_hash: Default::default(),
    }
}
