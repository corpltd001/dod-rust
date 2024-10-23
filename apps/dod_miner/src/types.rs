use bitcoin::key::Keypair;
use candid::{CandidType, Principal};
use core::fmt;
use flume::Sender;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse<T> {
    pub success: bool,
    pub data: T,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAccountReq {
    pub from_phrase: Option<String>,
    pub derived_path: Option<String>,
    pub network: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBatchAccountReq {
    pub from_phrase: Option<String>,
    pub derived_path: Option<String>,
    pub network: Option<String>,
    pub batch_size: Option<u32>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetAccountReq {
    pub network: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAccountRes {
    pub address: String,
    pub mnemonic: String,
    pub derived_path: String,
    pub network: String,
    pub kp: Option<Keypair>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyMessagePayload {
    pub address: String,
    pub message: String,
    pub signature: String,
    pub public_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterWorkerDTO {
    pub token: String,
    #[serde(rename = "userAddress")]
    pub user_address: String,
    #[serde(rename = "publicKey")]
    pub public_key: String,
    pub core: u32,
    pub memory: u32,
    pub ts: u32,
    pub ip: Option<String>,
    pub host_name: Option<String>,
    #[serde(rename = "jobVersions")]
    pub job_versions: Vec<String>,
    #[serde(rename = "workJobTypes")]
    pub worker_job_types: Vec<String>,
    #[serde(rename = "workerVersion")]
    pub worker_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchJobDTO {
    #[serde(rename = "userAddress")]
    pub user_address: String,
    pub amount: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchJobResponse {
    pub success: bool,
    pub data: Vec<JobDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchJobResponseV3 {
    pub success: bool,
    pub data: Vec<JobDetailV3>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobDetail {
    #[serde(rename = "workerJobToken")]
    pub worker_job_token: String,
    #[serde(rename = "jobData")]
    pub job_data: String,
    #[serde(rename = "jobTsHex")]
    pub job_ts_hex: Option<String>,
    #[serde(rename = "jobNonceHex")]
    pub job_nonce_hex: Option<String>,
    #[serde(rename = "jobBatchSize")]
    pub job_batch_size: u64,
    #[serde(rename = "jobVersion")]
    pub job_version: String,
    #[serde(rename = "jobBitworkType")]
    pub job_bitwork_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobDetailV3 {
    #[serde(rename = "workerJobToken")]
    pub worker_job_token: String,
    #[serde(rename = "jobData")]
    pub job_data: String,
    #[serde(rename = "jobTsHex")]
    pub job_ts_hex: String,
    #[serde(rename = "jobNonceHex")]
    pub job_nonce_hex: String,
    #[serde(rename = "jobBatchSize")]
    pub job_batch_size: u64,
    #[serde(rename = "jobVersion")]
    pub job_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckJobs {
    pub pending: u32,
    pub running: u32,
    pub finished: u32,
    pub submitted: u32,
}

pub type WorkerJobToken = String;
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum ThreadStatus {
    Idle,
    Busy(WorkerJobToken),
}

impl fmt::Display for ThreadStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ThreadStatus::Idle => write!(f, "Idle"),
            ThreadStatus::Busy(r) => write!(f, "Busy: {}", r.clone()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadResultExt {
    pub generated_nonce: u64,
    pub expired: bool,
    pub index: u32,
    pub worker_job_token: String,
    pub ts: String,
    pub nonce: String,
    pub start_time: u32,
    pub submitted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletData {
    pub wif: String,
    pub address: String,
    pub principal: Principal,
}

#[derive(Debug, Clone)]
pub struct TaskRunner<T> {
    pub is_loop: bool,
    pub sender: Option<Sender<T>>,
}

#[derive(CandidType, Clone, Serialize, Deserialize)]
pub enum SignMessageType {
    ECDSA,
    Bip322Simple,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct LoginDetails {
    /// The session expiration time in nanoseconds since the UNIX epoch. This is the time at which
    /// the delegation will no longer be valid.
    pub expiration: u64,

    /// The user canister public key. This key is used to derive the user principal.
    pub user_canister_pubkey: ByteBuf,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct SignedDelegation {
    pub delegation: Delegation,
    pub signature: ByteBuf,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Delegation {
    pub pubkey: ByteBuf,
    pub expiration: u64,
    pub targets: Option<Vec<Principal>>,
}

use dod_utils::bitwork::Bitwork;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt::Display;

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct BtcAddress(pub String);

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub enum MinerStatus {
    Activate,
    Deactivate,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct MinerInfo {
    pub owner: Principal,
    pub status: MinerStatus,
    pub ecdsa_pubkey: Vec<u8>,
    pub btc_address: String,
    pub reward_cycles: Option<u128>, // cycles
    pub claimed_dod: u64,            // dod coin
    pub total_dod: u64,              // dod coin
}

pub type Height = u64;
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct BlockData {
    pub height: Height,
    pub rewards: u64,
    pub winner: Option<MinerInfo>,
    pub difficulty: Bitwork,
    pub hash: Vec<u8>,
    pub block_time: u64,
    pub next_block_time: u64,
    pub history: bool,
    pub cycle_burned: u128,
    pub dod_burned: u64,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum DifficultyStatus {
    Increase,
    Decrease,
    Keep,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct BlockSigs {
    pub commit_tx: Vec<u8>,
    pub reveal_tx: Vec<u8>,
}

#[derive(CandidType, Serialize, Deserialize, Clone)]
pub struct BootStrapParams {
    pub dod_token_canister: Option<Principal>,
    pub dod_block_sub_account: Vec<u8>,
    pub block_timer: u64,
    pub difficulty_epoch: u64,
    pub default_rewards: u64,
    pub start_difficulty: Option<Bitwork>,
    pub halving_settings: Option<HalvingSettings>,
}

#[derive(CandidType, Serialize, Deserialize, Clone)]
pub struct MinerSubmitPayload {
    pub btc_address: String,
    pub signed_commit_psbt: String,
    pub signed_reveal_psbt: String,
    pub cycles_price: u128,
}

#[derive(CandidType, Serialize, Deserialize, Clone)]
pub struct MinerSubmitResponse {
    pub block_height: u64,
    pub cycles_price: u128,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct MinterCandidates {
    pub candidates: BTreeMap<String, MinerCandidate>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct MinerCandidate {
    pub btc_address: String,
    pub submit_time: u64,
    pub cycles_price: u128,
    pub signed_commit_psbt: String,
    pub signed_reveal_psbt: String,
}

impl Ord for MinerCandidate {
    fn cmp(&self, other: &Self) -> Ordering {
        // 先比较 block，较小的排在前面
        match self.cycles_price.cmp(&other.cycles_price) {
            Ordering::Equal => {
                // block 相同时再比较 index
                self.submit_time.cmp(&other.submit_time)
            }
            other => other,
        }
    }
}

impl PartialOrd for MinerCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct MinerCandidateKey {
    pub btc_address: String,
    pub block: u64,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct BlockOrders {
    pub block_height: u64,
    pub orders: BTreeMap<UserOrdersKey, u128>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum UserType {
    Miner,
    User,
    Treasury,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct UserOrdersKey {
    pub p: Principal,
    pub u: UserType,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct UserOrders {
    pub principal: Principal,
    pub orders: BTreeMap<u64, u128>,
    pub user_type: UserType,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct UserBlockOrder {
    pub block: u64,
    pub amount: u128,
    pub share: f64,
    pub reward: u64,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct UserBlockOrderRes {
    pub total: u64,
    pub from: u64,
    pub to: u64,
    pub data: Vec<UserBlockOrder>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct HalvingSettings {
    pub interval: u64,
    pub ratio: f64,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct DodCanisters {
    pub ledger: Principal,
    pub index: Principal,
    pub archive: Principal,
}
pub type BlockNumber = u64;
pub type BlockRange = (BlockNumber, BlockNumber);

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct NewBlockOrderValue {
    pub r: BlockRange,
    pub v: u128,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct UserBlockOrderData {
    pub height: u64,
    pub amount: u128, // cycles_amount
    pub share: f64,   // cycles_share
    pub reward: u64,  // dod reward
    pub user: Principal,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct BlockDataFull {
    pub block: BlockData,
    pub user_data: Vec<UserBlockOrderData>,
    pub miners: Vec<MinerCandidateExt>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct MinerCandidateExt {
    pub miner_principal: Principal,
    pub btc_address: String,
    pub submit_time: u64,
    pub cycles_price: u128,
    pub signed_commit_psbt: String,
    pub signed_reveal_psbt: String,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum OrderStatus {
    Pending,
    Filled,
    Cancelled,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct OrderDetail {
    pub value: u128,
    pub status: OrderStatus,
}
#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct ThreadResult {
    pub res: Option<MiningResult>,
    pub generated_nonce: u64,
    pub expired: bool,
    pub index: u32,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct MiningResult {
    pub num_bytes: u64,
    pub time: u32,
    pub nonce: u32,
}

impl Display for MiningResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        serde_json::to_string(self).unwrap().fmt(f)
    }
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum MiningResultType {
    Cpu(MiningResult),
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct MiningResultExt {
    pub result: MiningResultType,
    pub remote_hash: Vec<u8>,
    pub dead_line: u128,
}
