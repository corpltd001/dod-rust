use crate::types::{BlockData, LoginDetails, MinerInfo, MiningResultType, SignMessageType};
use bip322_simple::simple_signature_with_wif_taproot;

use bitcoin::key::TapTweak;
use bitcoin::secp256k1::{Secp256k1, XOnlyPublicKey};
use bitcoin::{Address, Network};
use candid::{Decode, Encode, Principal};
use dod_cpu::tx::{compose_submit_result, CreateDodTxExt};
use dod_utils::types::{MinerSubmitPayload, MinerSubmitResponse};
use ic_agent::agent::EnvelopeContent;
use ic_agent::identity::{BasicIdentity, DelegatedIdentity, Delegation, SignedDelegation};
use ic_agent::{Agent, Identity, Signature};
use log::info;
use ring::signature::Ed25519KeyPair;
use std::sync::Arc;

#[derive(Clone)]
pub struct ClonableIdentity {
    inner: Arc<dyn Identity + 'static>,
}

impl ClonableIdentity {
    pub fn new(identity: impl Identity + 'static + Send + Sync) -> Self {
        Self {
            inner: Arc::new(identity),
        }
    }
    pub fn get_inner(&self) -> Arc<dyn Identity + 'static> {
        self.inner.clone()
    }
}

impl Identity for ClonableIdentity {
    fn sender(&self) -> Result<Principal, String> {
        self.inner.sender()
    }

    fn public_key(&self) -> Option<Vec<u8>> {
        self.inner.public_key()
    }

    fn sign(&self, content: &EnvelopeContent) -> Result<Signature, String> {
        self.inner.sign(content)
    }

    fn sign_delegation(&self, content: &Delegation) -> Result<Signature, String> {
        self.inner.sign_delegation(content)
    }

    fn sign_arbitrary(&self, content: &[u8]) -> Result<Signature, String> {
        self.inner.sign_arbitrary(content)
    }

    fn delegation_chain(&self) -> Vec<SignedDelegation> {
        self.inner.delegation_chain()
    }
}

pub struct FetcherService {
    pub delegation_identity: Option<ClonableIdentity>,
    pub siwb_canister: Principal,
    pub dod_canister: Principal,
    pub ic_network: Option<String>,
    pub is_miner: bool,
}

impl Default for FetcherService {
    fn default() -> Self {
        Self {
            delegation_identity: None,
            siwb_canister: Principal::from_text("be2us-64aaa-aaaaa-qaabq-cai").unwrap(),
            dod_canister: Principal::from_text("bkyz2-fmaaa-aaaaa-qaaaq-cai").unwrap(),
            ic_network: Some("local".to_string()),
            is_miner: false,
        }
    }
}

impl FetcherService {
    pub fn new(
        delegation_identity: Option<ClonableIdentity>,
        siwb_canister: Principal,
        dod_canister: Principal,
        ic_network: Option<String>,
    ) -> Self {
        Self {
            delegation_identity,
            siwb_canister,
            dod_canister,
            ic_network,
            is_miner: false,
        }
    }

    pub fn get_delegation_identity(&self) -> Option<ClonableIdentity> {
        self.delegation_identity.clone()
    }

    pub fn set_delegation_identity(&mut self, delegation_identity: ClonableIdentity) {
        self.delegation_identity = Some(delegation_identity);
    }

    pub fn set_is_miner(&mut self, is_miner: bool) {
        self.is_miner = is_miner;
    }

    pub fn set_siwb_canister(&mut self, siwb_canister: Principal) {
        self.siwb_canister = siwb_canister;
    }

    pub fn set_dod_canister(&mut self, dod_canister: Principal) {
        self.dod_canister = dod_canister;
    }

    pub fn set_ic_network(&mut self, ic_network: Option<String>) {
        self.ic_network = ic_network;
    }

    pub fn get_dod_canister(&self) -> Principal {
        self.dod_canister.clone()
    }

    pub fn get_siwb_canister(&self) -> Principal {
        self.siwb_canister.clone()
    }

    pub fn get_ic_network(&self) -> Option<String> {
        self.ic_network.clone()
    }

    pub fn is_miner(&self) -> bool {
        self.is_miner
    }

    pub async fn connect(
        &mut self,
        wif: String,
        btc_address: String,
        btc_pubkey: String,
    ) -> Result<(), String> {
        let session = create_basic_identity()?;
        let session_key = session
            .public_key()
            .map(|x| x)
            .expect("Could not get public key");
        let _session = ClonableIdentity::new(session);

        let agent = with_agent(_session.clone(), self.get_ic_network()).await;
        let canister = self.get_siwb_canister();

        let siwb_prepare_login_call_res = agent
            .update(&canister, "siwb_prepare_login")
            .with_arg(Encode!(&btc_address).map_err(|e| format!("Error encoding: {:?}", e))?)
            .await
            .map_err(|e| format!("Error siwb_prepare_login: {:?}", e))?;

        let decoded_message =
            Decode!(siwb_prepare_login_call_res.as_slice(), Result<String, String>)
                .unwrap_or_else(|e| Err(e.to_string()))?;

        let signature = simple_signature_with_wif_taproot(decoded_message.as_str(), wif.as_str());

        let siwb_login_call_res = agent
            .update(&canister, "siwb_login")
            .with_arg(
                Encode!(
                    &signature,
                    &btc_address,
                    &btc_pubkey,
                    &session_key,
                    &SignMessageType::Bip322Simple
                )
                .map_err(|e| format!("Error encoding: {:?}", e))?,
            )
            .await
            .map_err(|e| format!("Error siwb_login: {:?}", e))?;

        let login_details = Decode!(siwb_login_call_res.as_slice(), Result<LoginDetails, String>)
            .unwrap_or_else(|e| Err(e.to_string()))?;

        let siwb_get_delegation_res = agent
            .query(&canister, "siwb_get_delegation")
            .with_arg(
                Encode!(&btc_address, &session_key, &login_details.expiration)
                    .map_err(|e| format!("Error encoding: {:?}", e))?,
            )
            .await
            .map_err(|e| format!("Error siwb_get_delegation: {:?}", e))?;

        let delegation_result =
            Decode!(siwb_get_delegation_res.as_slice(), Result<crate::types::SignedDelegation, String>)
                .unwrap_or_else(|e| Err(e.to_string()))?;

        let delegated_pubkey = delegation_result.delegation.pubkey.clone();
        let expiration = delegation_result.delegation.expiration;
        let targets = delegation_result.delegation.targets;

        let signature = delegation_result.signature;

        let user_key = login_details.user_canister_pubkey;

        let delegation = Delegation {
            expiration,
            pubkey: delegated_pubkey.into_vec(),
            targets,
        };

        let delegated_identity = DelegatedIdentity::new(
            user_key.into_vec(),
            Box::new(_session.clone().get_inner()),
            vec![SignedDelegation {
                delegation,
                signature: signature.into_vec(),
            }],
        );

        self.set_delegation_identity(ClonableIdentity::new(delegated_identity));

        Ok(())
    }

    pub async fn get_last_block(&self) -> Result<Option<(u64, BlockData)>, String> {
        let agent = with_agent(
            self.delegation_identity.clone().unwrap(),
            self.get_ic_network(),
        )
        .await;
        let get_last_block = agent
            .query(&self.get_dod_canister(), "get_last_block")
            .with_arg(Encode!().unwrap())
            .await
            .map_err(|e| format!("Error get_last_block: {:?}", e))?;

        let rrr = Decode!(get_last_block.as_slice(), Option<(u64, BlockData)>)
            .map_err(|e| format!("Error decoding: {:?}", e))?;

        Ok(rrr)
    }

    pub async fn register_miner(
        &mut self,
        btc_address: String,
        public_key: String,
    ) -> Result<bool, String> {
        let agent = with_agent(
            self.delegation_identity.clone().unwrap(),
            self.get_ic_network(),
        )
        .await;
        let register = agent
            .update(&self.get_dod_canister(), "register")
            .with_arg(Encode!(&btc_address, &public_key).unwrap())
            .await
            .map_err(|e| format!("Error who_am_i: {:?}", e))?;

        let rrr = Decode!(register.as_slice(), Result<MinerInfo,String>)
            .map_err(|e| format!("Error decoding: {:?}", e))?;

        let mut is_miner = false;

        if rrr.is_ok() || (rrr.is_err() && rrr.clone().unwrap_err().contains("already existed")) {
            is_miner = true;
        }

        self.set_is_miner(is_miner);
        Ok(is_miner)
    }

    pub async fn submit_result(
        &self,
        remote_hash: Vec<u8>,
        raw_pubkey: Vec<u8>,
        address: String,
        wif: String,
        mining_result: MiningResultType,
        cycles_price: u128,
    ) -> Result<MinerSubmitResponse, String> {
        let private_key = bitcoin::key::PrivateKey::from_wif(wif.as_str()).unwrap();

        let cycles_price = cycles_price;
        let (time, nonce, num_bytes) = match mining_result {
            MiningResultType::Cpu(r) => (r.time, r.nonce, r.num_bytes.to_le_bytes().to_vec()),
        };

        println!("remote_hash {:?}", hex::encode(remote_hash.clone()));
        let composed = compose_submit_result(
            CreateDodTxExt {
                remote_hash: remote_hash.clone(),
                raw_pubkey,
                time,
                nonce,
                num_bytes,
                address: address.clone(),
            },
            &private_key,
        );

        let payload = MinerSubmitPayload {
            btc_address: address.clone(),
            signed_commit_psbt: composed.signed_commit_psbt,
            signed_reveal_psbt: composed.signed_reveal_psbt,
            cycles_price,
        };

        let agent = with_agent(
            self.delegation_identity.clone().unwrap(),
            self.get_ic_network(),
        )
        .await;
        let submitted = agent
            .update(&self.get_dod_canister(), "miner_submit_hash")
            .with_arg(Encode!(&payload).unwrap())
            .await
            .map_err(|e| format!("Error miner_submit_hash: {:?}", e))?;

        let submitted_result = Decode!(submitted.as_slice(), Result<MinerSubmitResponse, String>)
            .map_err(|e| format!("Error decoding: {:?}", e))?;

        info!("Result Submitted: {:?}", submitted_result);
        submitted_result
    }
}

pub fn create_basic_identity() -> Result<impl Identity + 'static, String> {
    let rng = ring::rand::SystemRandom::new();
    let key_pair = Ed25519KeyPair::generate_pkcs8(&rng).expect("Could not generate a key pair.");

    Ok(BasicIdentity::from_key_pair(
        Ed25519KeyPair::from_pkcs8(key_pair.as_ref()).expect("Could not read the key pair."),
    ))
}

pub async fn with_agent(identity: ClonableIdentity, ic_network: Option<String>) -> Agent {
    let agent = create_agent(identity, ic_network.clone())
        .await
        .expect("Could not create an agent.");

    if ic_network.clone().is_some() && ic_network.clone().unwrap() == "local" {
        agent
            .fetch_root_key()
            .await
            .expect("could not fetch root key");
    }
    agent
}

pub async fn create_agent(
    identity: ClonableIdentity,
    ic_network: Option<String>,
) -> Result<Agent, String> {
    if ic_network.is_none() || ic_network.unwrap() == "local" {
        let port_env = std::env::var("IC_REF_PORT").unwrap_or_else(|_| "8080".into());
        let port = port_env
            .parse::<u32>()
            .expect("Could not parse the IC_REF_PORT environment variable as an integer.");

        Agent::builder()
            .with_url(format!("http://127.0.0.1:{}", port))
            .with_identity(identity)
            .build()
            .map_err(|e| format!("{:?}", e))
    } else {
        Agent::builder()
            .with_url("https://icp-api.io")
            .with_identity(identity)
            .build()
            .map_err(|e| format!("{:?}", e))
    }
}

pub fn get_p2tr_from_wif(wif: &str, network: &str) -> (String, String) {
    let private_key = bitcoin::key::PrivateKey::from_wif(wif).unwrap();
    let secp = Secp256k1::new();

    // Step 3: 从私钥生成XOnly公钥 (Schnorr公钥)
    let key_pair = bitcoin::secp256k1::Keypair::from_secret_key(&secp, &private_key.inner);
    let (x_only_pubkey, _parity) = XOnlyPublicKey::from_keypair(&key_pair);

    // Step 4: 应用tweak调整
    let (tweaked_pubkey, _) = x_only_pubkey.tap_tweak(&secp, None);
    let _network = if network == "mainnet" || network == "bitcoin" || network == "ic" {
        Network::Bitcoin
    } else {
        Network::Testnet
    };

    // Step 5: 生成比特币P2TR地址
    let tweaked_address = Address::p2tr_tweaked(tweaked_pubkey, _network);
    (
        tweaked_address.to_string(),
        key_pair.public_key().to_string(),
    )
}
