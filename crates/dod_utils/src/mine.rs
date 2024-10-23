use crate::bitwork::{compare_bitwork_range, Bitwork};
use bitcoin::hashes::{sha256, sha256d, Hash};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

fn hex_to_vec(hex_string: &str) -> Option<Vec<u8>> {
    let mut bytes = Vec::new();

    for i in 0..(hex_string.len() / 2) {
        let res = u8::from_str_radix(&hex_string[i * 2..i * 2 + 2], 16);
        match res {
            Ok(v) => bytes.push(v),
            Err(_) => return None,
        }
    }

    Some(bytes)
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize, Hash)]
pub struct BitworkResult {
    pub prefix: Vec<u8>,
    pub len: usize,
    pub min: u8,
    pub max: u8,
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize, Hash)]
pub struct BitworkResult2 {
    pub prefix: Vec<u8>,
    pub len: usize,
    pub k: u8,
}

pub fn easy_bitwork_2(
    hex_string: &str,
    string_width: u64,
    bitworkx: Option<String>,
) -> Option<BitworkResult2> {
    if hex_string.len() % 2 != 0 {
        None
    } else {
        Some(BitworkResult2 {
            prefix: hex_to_vec(hex_string).unwrap(),
            len: string_width as usize,
            k: bitworkx.map_or_else(|| 0, |r| u8::from_str_radix(r.as_str(), 16).unwrap()),
        })
    }
}

pub fn easy_bitwork(bitwork: &str, bitworkx: Option<String>) -> BitworkResult {
    if bitwork.len() % 2 == 0 {
        let (min, max) = bitworkx
            .map(|bitworkx| {
                (
                    u8::from_str_radix(&(bitworkx.to_owned() + "0"), 16).unwrap(),
                    u8::from_str_radix("ff", 16).unwrap(),
                )
            })
            .unwrap_or((0, 0));

        BitworkResult {
            prefix: hex_to_vec(bitwork).unwrap(),
            len: bitwork.len() / 2,
            min,
            max,
        }
    } else {
        let len = bitwork.len() - 1;
        let prefix = &bitwork[0..len];
        let last = &bitwork[len..];

        let min = u8::from_str_radix(&(last.to_string() + "0"), 16).unwrap();
        let max = u8::from_str_radix(&(last.to_string() + "f"), 16).unwrap();

        let min = bitworkx
            .map(|bitworkx| {
                u8::from_str_radix(&(last.to_string() + bitworkx.as_str()), 16).unwrap()
            })
            .unwrap_or(min);

        BitworkResult {
            prefix: hex_to_vec(prefix).unwrap(),
            len: len / 2,
            min,
            max,
        }
    }
}

pub fn easy_bitwork_from_obj(bitwork: Bitwork, override_prefix: Option<String>) -> BitworkResult {
    let bitworkx = Some(bitwork.post_hex.clone());
    let pre = override_prefix.unwrap_or(format!("{}", bitwork.pre));
    easy_bitwork(&format!("{}", pre), bitworkx)
}

pub fn mine_bitwork(
    tx: Vec<u8>,
    offset: usize,
    hashes: u64,
    prefix: String,
    ext: Option<String>,
) -> Result<u64, String> {
    let bitwork = easy_bitwork(&prefix, ext);
    mine_bitwork_raw(tx, offset, hashes, bitwork)
}

pub fn mine_bitwork_with_deadline(
    tx: Vec<u8>,
    offset: usize,
    hashes: u64,
    prefix: String,
    string_width: u64,
    ext: Option<String>,
    deadline: u128,
) -> Result<u64, String> {
    let bitwork = easy_bitwork_2(&prefix, string_width, ext);
    if bitwork.is_none() {
        Err("invalid bitwork".into())
    } else {
        mine_bitwork_raw_dead_line(tx, offset, hashes, bitwork.unwrap(), Some(deadline))
    }
}

pub fn mine_bitwork_raw_dead_line(
    tx: Vec<u8>,
    offset: usize,
    hashes: u64,
    bitwork: BitworkResult2,
    deadline: Option<u128>,
) -> Result<u64, String> {
    let mut tx = tx.clone();
    let s = offset;
    let e = offset + 8;
    let mut hashes: u64 = hashes;
    let mut new_tx_hash: [u8; 32];
    let mut sha2_hash: sha256::Hash;
    let mut sha2d_hash: sha256d::Hash;
    let mut hashes_bytes: [u8; 8];
    loop {
        if let Some(deadline) = deadline {
            if SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
                > deadline
            {
                return Err(hashes.to_string());
            }
        }

        // update tx
        hashes_bytes = hashes.to_le_bytes();
        tx[s..e].copy_from_slice(&hashes_bytes);

        // double sha256 and reverse
        sha2_hash = Hash::hash(&tx);
        sha2d_hash = sha2_hash.hash_again();
        new_tx_hash = sha2d_hash.to_byte_array();

        if compare_bitwork_range(&new_tx_hash, &bitwork.prefix, bitwork.len, bitwork.k) {
            return Ok(hashes);
        }

        // increment hashes
        hashes += 1;
        if hashes == 0xfffffffffffffffe {
            return Err(
                "The hashes has exceeded the allowed value of RBF (0xfffffffffffffffe)".into(),
            );
        }
    }
}

pub fn mine_bitwork_raw(
    tx: Vec<u8>,
    offset: usize,
    hashes: u64,
    bitwork: BitworkResult,
) -> Result<u64, String> {
    let mut tx = tx.clone();
    let s = offset;
    let e = offset + 8;
    let mut hashes: u64 = hashes;
    let mut new_tx_hash: [u8; 32];
    let mut sha2_hash: sha256::Hash;
    let mut sha2d_hash: sha256d::Hash;
    let mut hashes_bytes: [u8; 8];
    loop {
        // update tx
        hashes_bytes = hashes.to_le_bytes();
        tx[s..e].copy_from_slice(&hashes_bytes);

        // double sha256 and reverse
        sha2_hash = Hash::hash(&tx);
        sha2d_hash = sha2_hash.hash_again();
        new_tx_hash = sha2d_hash.to_byte_array();
        new_tx_hash.reverse();

        // check if txid has valid bitwork
        if new_tx_hash[0..bitwork.len] == bitwork.prefix {
            if bitwork.max == 0 {
                return Ok(hashes);
            }
            let val = new_tx_hash[bitwork.len];
            if val >= bitwork.min && val <= bitwork.max {
                return Ok(hashes);
            }
        }

        // increment hashes
        hashes += 1;
        if hashes == 0xfffffffffffffffe {
            return Err(
                "The hashes has exceeded the allowed value of RBF (0xfffffffffffffffe)".into(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::bitwork::compare_bitwork_range;
    use crate::mine::{easy_bitwork, easy_bitwork_2, mine_bitwork_with_deadline};
    use std::time::SystemTime;

    #[test]
    fn easy_bitwork2() {
        let hash = "123456";
        let string_width = 6;
        let hash_ext = "8";
        let bitwork = easy_bitwork_2(hash, string_width, Some(hash_ext.to_string())).unwrap();

        let cmp_result = compare_bitwork_range(
            &[0x8a, 0x56, 0x34, 0x12],
            &bitwork.prefix,
            bitwork.len,
            bitwork.k,
        );

        println!("bitwork {:?}", bitwork);
        println!("cmp_result {:?}", cmp_result);
    }

    #[test]
    fn it_works() {
        println!("7777.a {:?}", easy_bitwork("7777", Some("a".to_string())));
        println!("7777 {:?}", easy_bitwork("7777", None));
        println!("aabbcc {:?}", easy_bitwork("aabbcc", None));
        println!(
            "aabbcc.2 {:?}",
            easy_bitwork("aabbcc", Some("2".to_string()))
        );
        println!("aabbccd {:?}", easy_bitwork("aabbccd", None));
        println!(
            "aabbccd.7 {:?}",
            easy_bitwork("aabbccd", Some("7".to_string()))
        );
        println!(
            "aabbccd.12 {:?}",
            easy_bitwork("aabbccd", Some("c".to_string()))
        );
        println!(
            "123456789.0 {:?}",
            easy_bitwork("123456789", Some("0".to_string()))
        );
        let _tx = vec![
            1, 0, 0, 0, 1, 200, 49, 254, 51, 64, 175, 51, 119, 93, 14, 71, 47, 220, 151, 213, 181,
            238, 158, 208, 78, 244, 84, 19, 231, 19, 244, 175, 42, 134, 36, 193, 1, 1, 0, 0, 0, 0,
            1, 0, 0, 0, 3, 70, 5, 0, 0, 0, 0, 0, 0, 34, 81, 32, 22, 39, 197, 165, 169, 0, 65, 189,
            93, 216, 241, 3, 156, 134, 168, 84, 150, 21, 54, 216, 104, 208, 135, 159, 151, 175,
            251, 214, 20, 101, 48, 66, 248, 17, 0, 0, 0, 0, 0, 0, 34, 81, 32, 123, 146, 234, 37,
            163, 109, 167, 77, 38, 53, 106, 55, 87, 217, 166, 139, 7, 137, 173, 53, 30, 155, 237,
            48, 196, 120, 42, 210, 49, 64, 233, 163, 127, 111, 29, 0, 0, 0, 0, 0, 34, 81, 32, 135,
            152, 173, 174, 177, 28, 49, 177, 197, 129, 193, 59, 163, 212, 115, 215, 233, 15, 147,
            204, 18, 241, 224, 85, 33, 142, 45, 191, 202, 198, 103, 68, 0, 0, 0, 0,
        ];
        // let i = mine_bitwork(tx, 42, 1, "1234".to_string(), None).unwrap();
        // println!("{}", i);
        //
        // assert_eq!(
        //     easy_bitwork_from_obj(
        //         Bitwork {
        //             pre: 1234,
        //             post_hex: "1".to_string(),
        //         },
        //         None
        //     ),
        //     easy_bitwork("1234", Some("1".to_string()))
        // );

        let tx2 = hex::decode("01000000011d345364868f17be20373460fe022fc54243cf749ea888e97dcc24873691168b0000000000fdffffff03b0040000000000002251201a6b8cce40e18cc56ce97592b6d579bd0f0bc716383b1beda3a53da2a25fd11b00000000000000000a6a089d4b1212d0c917e64e5201000000000022512061f023b192540b40b459e9aa62aedceb874e6ea599723d21aa7274e5ddc3be8900000000").unwrap();

        let i2 = mine_bitwork_with_deadline(
            tx2,
            101,
            1,
            "1234".to_string(),
            4,
            Some("a".to_string()),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
                + 10000000000000,
        )
        .unwrap();
        println!("{:?}", i2.to_le_bytes() as [u8; 8]);
    }
}
