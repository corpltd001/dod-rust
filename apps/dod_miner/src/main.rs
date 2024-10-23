use candid::Principal;
use clap::Parser;
use dod_miner::fetcher::get_p2tr_from_wif;

use dod_miner::miner::multi_run_v3;
use dod_miner::state::{LATEST_BLOCK, MINER, RUNNING, THREADS};
use dod_miner::types::{MiningResultExt, MiningResultType};
use dod_utils::bitwork::Bitwork;
use dotenv::dotenv;
use flume::Sender;
use log::{error, info};
use std::time::Duration;
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};

#[derive(Parser)] // requires `derive` feature
enum DodCli {
    Miner(MinerArgs),
}

#[derive(clap::Args)]
struct MinerArgs {
    #[arg(long = "threads")]
    threads: Option<u32>,
    #[arg(long = "wif")]
    wif: String,
    #[arg(long = "cycles_price")]
    cycles_price: String, // #[arg(long = "siwb_canister")]
                          // siwb_canister: Option<String>,
                          // #[arg(long = "dod_canister")]
                          // dod_canister: Option<String>,
}

#[tokio::main]
async fn main() {
    let DodCli::Miner(minter_args) = DodCli::parse();
    dotenv().ok();
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();

    let _siwb_canister = String::from("mwm4a-eiaaa-aaaah-aebnq-cai");
    let _dod_canister = String::from("tmhkz-dyaaa-aaaah-aedeq-cai");
    let _ic_network = String::from("ic");
    let _account_wif = minter_args.wif;

    let _deadline_diff = String::from("5000000000");
    let _cycles_price = (minter_args.cycles_price.parse::<f64>().unwrap()
        * u128::pow(10, 12) as f64)
        .round() as u128;

    let _threads = minter_args
        .threads
        .map_or_else(|| None, |f| Some(f.clone()));

    if _threads.is_some() {
        tokio::spawn(async move {
            let mut tm = THREADS.lock().await;
            tm.set_max_threads(_threads.unwrap());
        });
    }

    let (tx, rx) = flume::unbounded::<MiningResultExt>();

    let _registered = register(
        _dod_canister.as_str(),
        _siwb_canister.as_str(),
        _ic_network.as_str(),
        _account_wif.as_str(),
    )
    .await;

    let (btc_address, btc_pubkey) = _registered.unwrap();
    _schedule_fetch(
        (btc_address.clone(), btc_pubkey.clone()),
        _threads,
        tx.clone(),
        Some(u64::from_str_radix(_deadline_diff.as_str(), 10).unwrap_or(5_000_000_000)),
    )
    .await
    .unwrap();

    //
    // _schedule_fetch(tx.clone(), None).await.unwrap();
    //
    // // println!("123123");
    //
    // // first run
    // // runner(tx.clone(), None, None);
    //
    let result = rx.iter();
    for r in result {
        submit(
            (btc_address.clone(), btc_pubkey.clone()),
            _account_wif.clone(),
            r,
            _cycles_price.clone(),
        );
    }
}

async fn _schedule_fetch(
    miner: (String, String),
    threads: Option<u32>,
    tx: Sender<MiningResultExt>,
    deadline_diff: Option<u64>,
) -> Result<String, JobSchedulerError> {
    let sched = JobScheduler::new().await?;
    let check_job = Job::new_async("1/5 * * * * *", move |_uuid, _l| {
        let _tx = tx.clone();
        let _miner = miner.clone();
        let _threads = threads.clone();
        Box::pin(async move {
            match fetch_blocks(deadline_diff).await {
                Ok((hash, bitwork, dead_line)) => {
                    let _ = multi_run_v3(
                        bitwork,
                        hash.clone(),
                        hex::decode(_miner.1).unwrap(),
                        _threads,
                        dead_line,
                    )
                    .await
                    .map_or_else(
                        |e| {
                            error!("Mined Error: {}", e);
                        },
                        |r| {
                            _tx.send(MiningResultExt {
                                result: MiningResultType::Cpu(r),
                                remote_hash: hash.clone(),
                                dead_line,
                            })
                            .unwrap();
                        },
                    );
                }
                Err(e) => {
                    info!("{:?}", e);
                }
            }
        })
    })?;
    let id = sched.add(check_job).await?;
    sched.start().await?;

    // Wait while the jobs run
    tokio::time::sleep(Duration::from_secs(1)).await;
    Ok(id.to_string())
}

async fn register(
    dod: &str,
    siwb: &str,
    ic_network: &str,
    wif: &str,
) -> Result<(String, String), String> {
    let mut miner = MINER.lock().await;

    miner.set_dod_canister(Principal::from_text(dod).unwrap());
    miner.set_siwb_canister(Principal::from_text(siwb).unwrap());
    miner.set_ic_network(Some(ic_network.to_string()));

    let (btc_address, btc_pubkey) = get_p2tr_from_wif(wif, ic_network);

    match miner
        .connect(wif.to_string(), btc_address.clone(), btc_pubkey.clone())
        .await
    {
        Ok(_) => {
            info!("Connected to the miner successfully");
            match miner
                .register_miner(btc_address.clone(), btc_pubkey.clone())
                .await
            {
                Ok(_) => {
                    info!("Miner registered successfully");
                    Ok((btc_address.clone(), btc_pubkey.clone()))
                }
                Err(e) => {
                    error!("Error registering the miner: {}", e);
                    Err(e)
                }
            }
        }
        Err(e) => {
            error!("Error connecting to the miner: {}", e);
            Err(e)
        }
    }
}

async fn fetch_blocks(deadline_diff: Option<u64>) -> Result<(Vec<u8>, Bitwork, u128), String> {
    info!("should fetch blocks?");
    let miner = MINER.lock().await;
    let mut latest_block = LATEST_BLOCK.lock().await;
    let running = RUNNING.lock().await;

    if running.clone() == true {
        return Err("Already running".to_string());
    }

    if miner.is_miner() {
        match miner.get_last_block().await {
            Ok(b) => {
                if b.is_none() {
                    Err("No blocks found".to_string())
                } else {
                    let (num, block) = b.unwrap();
                    if latest_block.is_none()
                        || (latest_block.is_some() && num > latest_block.unwrap())
                    {
                        *latest_block = Some(num);

                        let dead_line = (block.next_block_time
                            - deadline_diff.unwrap_or(5_000_000_000))
                            as u128;
                        let hash = block.hash.clone();
                        let bitwork = block.difficulty.clone();
                        Ok((hash, bitwork, dead_line))
                    } else {
                        Err("No new blocks found".to_string())
                    }
                }
            }
            Err(e) => Err(e),
        }
    } else {
        Err("Not a miner".to_string())
    }
}

fn submit(
    miner_tuple: (String, String),
    wif: String,
    mining_result: MiningResultExt,
    cycles_price: u128,
) {
    let _remote_hash = mining_result.remote_hash.clone();
    let _miner_tuple = miner_tuple.clone();
    let _wif = wif.clone();
    let _mining_result = mining_result.result.clone();
    let _cycles_price = cycles_price;
    let _dead_line = mining_result.dead_line.clone();

    let _ = tokio::spawn(async move {
        let miner = MINER.lock().await;
        let raw_pub = hex::decode(_miner_tuple.1).unwrap();

        let _ = miner
            .submit_result(
                _remote_hash,
                raw_pub,
                _miner_tuple.0,
                _wif,
                _mining_result,
                _cycles_price,
            )
            .await;
    });
}
