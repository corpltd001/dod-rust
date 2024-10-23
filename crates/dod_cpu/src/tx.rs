use crate::protocol::{DodAssets, DodMining, DodOps, DodStruct, MAGIC_VALUE, PROTOCOL_ID};
use bitcoin::absolute::LockTime;
use bitcoin::bip32::KeySource;
use bitcoin::consensus::serialize;
use bitcoin::key::{Secp256k1, TapTweak};
use bitcoin::psbt::PsbtSighashType;
use bitcoin::script::PushBytes;
use bitcoin::sighash::SighashCache;
use bitcoin::taproot::{LeafVersion, TaprootBuilder};
use bitcoin::transaction::Version;
use bitcoin::Network::{Bitcoin, Testnet};
use bitcoin::{
    opcodes, psbt, script, secp256k1, sighash, taproot, Address, AddressType, Amount, Network,
    OutPoint, Psbt, PublicKey, ScriptBuf, Sequence, TapLeafHash, TapSighash, TapSighashType,
    Transaction, TxIn, TxOut, Txid, Witness, XOnlyPublicKey,
};
use serde::Serialize;
use std::collections::BTreeMap;
use std::convert::TryInto;
use std::str::FromStr;

pub struct CreateDodTxDefault {
    pub remote_hash: Vec<u8>,
    pub raw_pubkey: Vec<u8>,
    pub time: u32,
    pub nonce: u32,
}

pub struct CreateDodTxExt {
    pub remote_hash: Vec<u8>,
    pub raw_pubkey: Vec<u8>,
    pub time: u32,
    pub nonce: u32,
    pub num_bytes: Vec<u8>,
    pub address: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct SubmitSignedPayload {
    pub btc_address: String,
    pub signed_commit_psbt: String,
    pub signed_reveal_psbt: String,
}

pub fn create_dod_tx(req: CreateDodTxDefault, random16: bool) -> (Vec<u8>, u32) {
    let mut remote = req.remote_hash.clone();
    remote.reverse();
    let prev_out = OutPoint {
        txid: Txid::from_str(hex::encode(remote).as_str()).unwrap(),
        vout: 0,
    };
    // let time = 1700000000u32;
    // let nonce = 9999999u32;
    let dod_struct = DodStruct {
        n: None,
        t: DodAssets::DMT,
        dmt: Some(DodMining {
            nonce: req.nonce,
            time: req.time,
        }),
    };

    let xonly = XOnlyPublicKey::from(PublicKey::from_slice(req.raw_pubkey.as_slice()).unwrap());

    let cbored = serde_cbor::to_vec(&dod_struct).unwrap();

    let script = script::Builder::new()
        .push_x_only_key(&xonly)
        .push_opcode(opcodes::all::OP_CHECKSIG)
        .push_opcode(opcodes::OP_FALSE)
        .push_opcode(opcodes::all::OP_IF)
        .push_slice(PROTOCOL_ID)
        .push_slice(DodOps::Mine.to_slice())
        .push_slice::<&PushBytes>(cbored.as_slice().try_into().unwrap())
        .push_opcode(opcodes::all::OP_ENDIF)
        .into_script();

    let default_r16 = if random16 {
        hex::decode("9d4b1212d0c917e668e55bbeb5eda717").unwrap()
    } else {
        hex::decode("9d4b1212d0c917e6").unwrap()
    };

    let default_op_return = TxOut {
        script_pubkey: script::Builder::new()
            .push_opcode(opcodes::all::OP_RETURN)
            .push_slice::<&PushBytes>(default_r16.as_slice().try_into().unwrap())
            .into_script(),
        value: Amount::from_sat(0),
    };

    let _raw_tx = Transaction {
        version: Version(1),
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
            previous_output: prev_out,
            script_sig: Default::default(),
            sequence: Sequence(0xFFFFFFFF),
            witness: Witness::default(),
        }],
        output: vec![
            TxOut {
                value: Amount::from_sat(1200),
                script_pubkey: script.to_p2tr(&Secp256k1::new(), xonly),
            },
            default_op_return,
        ],
    };

    let mut psbt = Psbt::from_unsigned_tx(_raw_tx).unwrap();

    psbt.inputs[0].witness_utxo = Some(TxOut {
        value: Amount::from_sat(MAGIC_VALUE),
        script_pubkey: script.to_p2tr(&Secp256k1::new(), xonly),
    });

    let s = serialize(&psbt.unsigned_tx);
    let binding = hex::encode(s.clone());
    let arr = if random16 {
        binding
            .split("9d4b1212d0c917e668e55bbeb5eda717")
            .collect::<Vec<&str>>()
    } else {
        binding.split("9d4b1212d0c917e6").collect::<Vec<&str>>()
    };
    let start = (arr[0].len() / 2) as u32;

    (s.clone(), start)
}

pub fn compose_submit_result(
    req: CreateDodTxExt,
    key: &bitcoin::key::PrivateKey,
) -> SubmitSignedPayload {
    let AddressInfo { script_buf, .. } = get_script_from_address(req.address.clone()).unwrap();

    let mut remote = req.remote_hash.clone();
    remote.reverse();

    let prev_out = OutPoint {
        txid: Txid::from_str(hex::encode(remote).as_str()).unwrap(),
        vout: 0,
    };
    // let time = 1700000000u32;
    // let nonce = 9999999u32;
    let dod_struct = DodStruct {
        n: None,
        t: DodAssets::DMT,
        dmt: Some(DodMining {
            nonce: req.nonce,
            time: req.time,
        }),
    };

    let xonly = XOnlyPublicKey::from(PublicKey::from_slice(req.raw_pubkey.as_slice()).unwrap());

    let cbored = serde_cbor::to_vec(&dod_struct).unwrap();

    let script = script::Builder::new()
        .push_x_only_key(&xonly)
        .push_opcode(opcodes::all::OP_CHECKSIG)
        .push_opcode(opcodes::OP_FALSE)
        .push_opcode(opcodes::all::OP_IF)
        .push_slice(PROTOCOL_ID)
        .push_slice(DodOps::Mine.to_slice())
        .push_slice::<&PushBytes>(cbored.as_slice().try_into().unwrap())
        .push_opcode(opcodes::all::OP_ENDIF)
        .into_script();

    let real_bytes = req.num_bytes.clone();

    let commit_op_return = TxOut {
        script_pubkey: script::Builder::new()
            .push_opcode(opcodes::all::OP_RETURN)
            .push_slice::<&PushBytes>(real_bytes.as_slice().try_into().unwrap())
            .into_script(),
        value: Amount::from_sat(0),
    };

    let reveal_input = TxOut {
        value: Amount::from_sat(1200),
        script_pubkey: script.to_p2tr(&Secp256k1::new(), xonly),
    };

    let _commit_tx_outs = vec![reveal_input.clone(), commit_op_return];

    let _commit_tx = Transaction {
        version: Version(1),
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
            previous_output: prev_out,
            script_sig: Default::default(),
            sequence: Sequence(0xFFFFFFFF),
            witness: Witness::default(),
        }],
        output: _commit_tx_outs.clone(),
    };

    let mut commit_psbt = Psbt::from_unsigned_tx(_commit_tx.clone()).unwrap();

    commit_psbt.inputs[0].witness_utxo = Some(TxOut {
        value: Amount::from_sat(MAGIC_VALUE),
        script_pubkey: script.to_p2tr(&Secp256k1::new(), xonly),
    });

    let _commit_tx_id = _commit_tx.clone().compute_txid();

    println!("_commit_tx_id: {:?}", _commit_tx_id.to_string());

    let signed_commit_psbt = sign_commit_psbt(
        commit_psbt,
        xonly,
        vec![TxOut {
            value: Amount::from_sat(MAGIC_VALUE),
            script_pubkey: script_buf.clone(),
        }],
        key,
    );

    let _reveal_tx_outs = vec![TxOut {
        value: Amount::from_sat(546),      // default value of 546 satoshi
        script_pubkey: script_buf.clone(), // send back to user
    }];

    let _reveal_tx = Transaction {
        version: Version(1),
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint {
                txid: _commit_tx_id,
                vout: 0,
            },
            script_sig: Default::default(),
            sequence: Sequence(0xFFFFFFFF),
            witness: Witness::default(),
        }],
        output: _reveal_tx_outs.clone(),
    };

    let mut reveal_psbt = Psbt::from_unsigned_tx(_reveal_tx).unwrap();
    reveal_psbt.inputs[0].witness_utxo = Some(reveal_input.clone());

    // let reveal_tx_id = _reveal_tx.compute_txid();

    let signed_reveal_psbt =
        sign_reveal_psbt(reveal_psbt, script, xonly, vec![reveal_input.clone()], key);

    SubmitSignedPayload {
        btc_address: req.address.clone(),
        signed_commit_psbt: signed_commit_psbt.to_string(),
        signed_reveal_psbt: signed_reveal_psbt.to_string(),
    }
}

fn sign_commit_psbt(
    _psbt: Psbt,
    xonly: XOnlyPublicKey,
    input_txouts: Vec<TxOut>,
    key: &bitcoin::key::PrivateKey,
) -> Psbt {
    let secp = Secp256k1::new();
    let mut psbt = _psbt.clone();
    let unsigned_tx = psbt.unsigned_tx.clone();
    psbt.inputs
        .iter_mut()
        .enumerate()
        .try_for_each::<_, Result<(), Box<dyn std::error::Error>>>(|(vout, input)| {
            let mut origins = BTreeMap::new();
            origins.insert(xonly, (vec![], KeySource::default()));
            input.tap_internal_key = Some(xonly);
            input.tap_key_origins = origins;

            let sighash_type = input
                .sighash_type
                .and_then(|psbt_sighash_type| psbt_sighash_type.taproot_hash_ty().ok())
                .unwrap_or(TapSighashType::All);
            let hash = SighashCache::new(&unsigned_tx)
                .taproot_key_spend_signature_hash(
                    vout,
                    &sighash::Prevouts::All(input_txouts.as_slice()),
                    sighash_type,
                )
                .unwrap();

            let (_, (_, _derivation_path)) = input
                .tap_key_origins
                .get(
                    &input
                        .tap_internal_key
                        .ok_or("internal key missing in PSBT")?,
                )
                .ok_or("missing Taproot key origin")?;

            let secret_key = key.inner;
            sign_psbt_taproot(secret_key, xonly, None, input, hash, sighash_type, &secp);
            Ok(())
        })
        .unwrap();

    psbt.inputs.iter_mut().for_each(|input| {
        let mut script_witness: Witness = Witness::new();
        script_witness.push(input.tap_key_sig.unwrap().to_vec());
        input.final_script_witness = Some(script_witness);
        // Clear all the data fields as per the spec.
        input.partial_sigs = BTreeMap::new();
        input.sighash_type = None;
        input.redeem_script = None;
        input.witness_script = None;
        input.bip32_derivation = BTreeMap::new();
        input.tap_key_sig = None;
        input.tap_internal_key = None;
        input.tap_key_origins = BTreeMap::new();
    });
    psbt
}

fn sign_reveal_psbt(
    _reveal_psbt: Psbt,
    script: ScriptBuf,
    xonly: XOnlyPublicKey,
    input_txouts_reveal: Vec<TxOut>,
    key: &bitcoin::key::PrivateKey,
) -> Psbt {
    let secp = Secp256k1::new();
    let leaf_hash = script.clone().tapscript_leaf_hash();
    let taproot_spend_info = TaprootBuilder::new()
        .add_leaf(0, script.clone())
        .unwrap()
        .finalize(&secp, xonly)
        .expect("should be finalizable");
    let mut reveal_psbt = _reveal_psbt.clone();
    let unsigned_tx = reveal_psbt.unsigned_tx.clone();
    reveal_psbt
        .inputs
        .iter_mut()
        .enumerate()
        .try_for_each::<_, Result<(), Box<dyn std::error::Error>>>(|(vout, input)| {
            let mut origins = BTreeMap::new();
            origins.insert(xonly, (vec![leaf_hash], KeySource::default()));
            input.tap_internal_key = Some(xonly);
            input.tap_key_origins = origins;
            input.tap_merkle_root = taproot_spend_info.merkle_root();
            let ty = PsbtSighashType::from_str("SIGHASH_ALL")?;
            let mut tap_scripts = BTreeMap::new();
            tap_scripts.insert(
                taproot_spend_info
                    .control_block(&(script.clone(), LeafVersion::TapScript))
                    .unwrap(),
                (script.clone(), LeafVersion::TapScript),
            );
            input.tap_scripts = tap_scripts;
            input.sighash_type = Some(ty);
            for (x_only_pubkey, (leaf_hashes, (_, _))) in &input.tap_key_origins.clone() {
                let secret_key = key.inner;
                for lh in leaf_hashes {
                    let sighash_type = TapSighashType::All;
                    let hash = SighashCache::new(&unsigned_tx)
                        .taproot_script_spend_signature_hash(
                            vout,
                            &sighash::Prevouts::All(&input_txouts_reveal.as_slice()),
                            *lh,
                            sighash_type,
                        )
                        .unwrap();
                    sign_psbt_taproot(
                        secret_key,
                        *x_only_pubkey,
                        Some(*lh),
                        input,
                        hash,
                        sighash_type,
                        &secp,
                    );
                }
            }
            Ok(())
        })
        .unwrap();
    reveal_psbt.inputs.iter_mut().for_each(|input| {
        let mut script_witness: Witness = Witness::new();
        for (_, signature) in input.tap_script_sigs.iter() {
            script_witness.push(signature.to_vec());
        }
        for (control_block, (scriptb, _)) in input.tap_scripts.iter() {
            script_witness.push(scriptb.to_bytes());
            script_witness.push(control_block.serialize());
        }
        input.final_script_witness = Some(script_witness);

        // Clear all the data fields as per the spec.
        input.partial_sigs = BTreeMap::new();
        input.sighash_type = None;
        input.redeem_script = None;
        input.witness_script = None;
        input.bip32_derivation = BTreeMap::new();
        input.tap_script_sigs = BTreeMap::new();
        input.tap_scripts = BTreeMap::new();
        input.tap_key_sig = None;
        input.tap_merkle_root = None;
        input.tap_internal_key = None;
        input.tap_key_origins = BTreeMap::new();
    });
    reveal_psbt
}

fn sign_psbt_taproot(
    secret_key: secp256k1::SecretKey,
    pubkey: XOnlyPublicKey,
    leaf_hash: Option<TapLeafHash>,
    psbt_input: &mut psbt::Input,
    hash: TapSighash,
    sighash_type: TapSighashType,
    secp: &Secp256k1<secp256k1::All>,
) {
    let keypair = bitcoin::key::Keypair::from_seckey_slice(secp, secret_key.as_ref()).unwrap();
    let keypair = match leaf_hash {
        None => keypair
            .tap_tweak(secp, psbt_input.tap_merkle_root)
            .to_inner(),
        Some(_) => keypair, // no tweak for script spend
    };

    let msg = secp256k1::Message::from(hash);
    let signature = secp.sign_schnorr_no_aux_rand(&msg, &keypair);

    let final_signature = taproot::Signature {
        signature,
        sighash_type,
    };

    if let Some(lh) = leaf_hash {
        psbt_input
            .tap_script_sigs
            .insert((pubkey, lh), final_signature);
    } else {
        psbt_input.tap_key_sig = Some(final_signature);
    }
}

pub struct AddressInfo {
    pub address: String,
    pub script_buf: ScriptBuf,
    pub network: Network,
    pub address_type: AddressType,
}

pub fn get_script_from_address(address: String) -> Result<AddressInfo, String> {
    let mut network = Bitcoin;
    let mut address_type = AddressType::P2tr;

    if address.starts_with("bc1q") {
        address_type = AddressType::P2wpkh;
        network = Bitcoin;
    } else if address.starts_with("bc1p") {
        address_type = AddressType::P2tr;
        network = Bitcoin;
    } else if address.starts_with('1') {
        address_type = AddressType::P2pkh;
        network = Bitcoin;
    } else if address.starts_with('3') {
        address_type = AddressType::P2sh;
        network = Bitcoin;
    } else if address.starts_with("tb1q") {
        address_type = AddressType::P2wpkh;
        network = Testnet;
    } else if address.starts_with('m') || address.starts_with('n') {
        address_type = AddressType::P2pkh;
        network = Testnet;
    } else if address.starts_with('2') {
        address_type = AddressType::P2sh;
        network = Testnet;
    } else if address.starts_with("tb1p") {
        address_type = AddressType::P2tr;
        network = Testnet;
    }
    let addr = Address::from_str(address.as_str())
        .map_err(|e| format!("Cannot gen address {:?}", e).to_string())?;

    let addr_checked = addr
        .clone()
        .require_network(network)
        .map_err(|e| format!("Cannot require network {:?}", e).to_string())?;

    Ok(AddressInfo {
        address: addr_checked.to_string(),
        script_buf: addr_checked.script_pubkey(),
        network,
        address_type,
    })
}

#[cfg(test)]
mod test {
    use crate::hash::do_sha256;
    use crate::tx::{create_dod_tx, CreateDodTxDefault};

    #[test]
    pub fn test_encode() {
        let remote_hash =
            hex::decode("59d0e915ea1d5d2e1feb78cb29f0548c0fb7f7c37d72aa6e237f6fb57e0eac5d")
                .unwrap();
        let raw_pubkey =
            hex::decode("02afee55a2cdcb6c47a593d629b04e13399354d348a3d84ad19310e2b6396e7237")
                .unwrap();

        let res = create_dod_tx(
            CreateDodTxDefault {
                nonce: 1,
                time: 1,
                remote_hash: remote_hash.clone(),
                raw_pubkey: raw_pubkey.clone(),
            },
            false,
        );
        println!("{:?}", res.1);

        let res2 = create_dod_tx(
            CreateDodTxDefault {
                nonce: 1,
                time: 1,
                remote_hash: remote_hash.clone(),
                raw_pubkey: raw_pubkey.clone(),
            },
            true,
        );
        println!(
            "{:?}, byteslength {:?}",
            hex::encode(res2.clone().0),
            res2.clone().0.len()
        );

        // vec_to_u84(&tx_bytes);

        let f = hex::encode(do_sha256("30226098This is a test".as_bytes().to_vec()));
        println!("sha256d{:?}", f);
    }
}
