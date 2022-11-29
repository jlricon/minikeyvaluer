use http::StatusCode;
use reqwest::Client;

use std::cmp::Ordering;
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone, Copy)]
pub enum Deleted {
    NO,
    SOFT,
    HARD,
}
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct Record {
    pub rvolumes: Vec<String>,
    pub deleted: Deleted,
    pub hash: String,
}
impl Record {
    fn default() -> Record {
        return Record {
            rvolumes: vec![],
            deleted: Deleted::NO,
            hash: "".to_string(),
        };
    }
}
impl Into<Record> for &[u8] {
    fn into(self) -> Record {
        let mut rec = Record::default();
        let mut ss: String = self.iter().map(|b| *b as char).collect();
        if ss.starts_with("DELETED") {
            rec.deleted = Deleted::SOFT;
            ss = ss[7..].to_string();
        };
        if ss.starts_with("HASH") {
            rec.hash = ss[4..36].to_string();
            ss = ss[36..].to_string();
        };
        rec.rvolumes = ss
            .split(',')
            .map(|c| c.to_string())
            .collect::<Vec<String>>();
        return rec;
    }
}
impl Into<Vec<u8>> for Record {
    fn into(self) -> Vec<u8> {
        let mut cc = "".to_string();
        match self.deleted {
            Deleted::HARD => panic!("Can't put HARD delete in the db!"),
            Deleted::SOFT => cc += "DELETED",
            Deleted::NO => {}
        };
        if self.hash.len() == 32 {
            cc += &("HASH".to_string() + &self.hash);
        };
        let vols = self.rvolumes.join(",");
        let ret = cc + &vols;
        return ret.into_bytes();
    }
}

// Hash functions
pub fn key2path(key: &[u8]) -> String {
    let mkey = md5::compute(key);
    let b64key = base64::encode(&key);
    return format!("/{:02x}/{:02x}/{}", mkey[0], mkey[1], b64key);
}
#[derive(Debug)]
struct SortVol {
    score: Vec<u8>,
    volume: String,
}
impl PartialEq for SortVol {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}

impl PartialOrd for SortVol {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(other.score.cmp(&self.score))
    }
}
impl Eq for SortVol {}
impl Ord for SortVol {
    fn cmp(&self, other: &Self) -> Ordering {
        other.score.cmp(&self.score)
    }
}

type ByScore = Vec<SortVol>;

fn key2volume(key: &[u8], volumes: &[&str], count: usize, svcount: usize) -> Vec<String> {
    let mut svs: ByScore = volumes
        .into_iter()
        .map(|vol| {
            let hash = md5::compute([key, vol.as_bytes()].concat());
            SortVol {
                score: hash.to_vec(),
                volume: vol.to_string(),
            }
        })
        .collect();

    svs.sort();
    dbg!(&svs);
    return svs
        .iter()
        .map(|sv| {
            if svcount == 1 {
                return sv.volume.clone();
            } else {
                let svhash = ((sv.score[12] as usize) << 24)
                    + ((sv.score[13] as usize) << 16)
                    + ((sv.score[14] as usize) << 8)
                    + sv.score[15] as usize;
                dbg!(svhash);

                return format!("{}/sv{:02X}", sv.volume, svhash % svcount);
            }
        })
        .take(count)
        .collect::<Vec<String>>();
}
fn needs_rebalance(volumes: Vec<String>, kvolumes: Vec<String>) -> bool {
    if volumes.len() != kvolumes.len() {
        return true;
    };
    volumes.iter().zip(kvolumes.iter()).any(|(a, b)| a != b)
}
pub fn remote_delete(remote: &str) -> Result<(), Box<dyn std::error::Error>> {
    let resp = reqwest::blocking::Client::new().delete(remote).send()?;
    if resp.status() != StatusCode::NO_CONTENT || resp.status() != StatusCode::NOT_FOUND {
        return Err(format!(
            "Error deleting. Wrong status code {}",
            resp.status().as_str()
        )
        .into());
    } else {
        return Ok(());
    }
}
// Tests
#[cfg(test)]
mod tests {
    fn from_to_record_example(rec: Record, val: String) {
        let rec2 = rec.clone();
        let recs: Vec<u8> = rec.into();

        // Make recs into a string
        let recs_string: String = recs.iter().map(|b| *b as char).collect();
        assert_eq!(recs_string, val);
        let reca: Record = (&*recs).into();
        assert_eq!(reca, rec2);
    }

    use super::*;
    #[test]
    fn test_key2path() {
        let examples = vec![
            ("hello", "/5d/41/aGVsbG8="),
            ("helloworld", "/fc/5e/aGVsbG93b3JsZA=="),
        ];
        for (key, path) in examples {
            assert_eq!(key2path(key.as_bytes()), path);
        }
    }
    #[test]
    fn test_key2volume() {
        let volumes = vec!["larry", "moe", "curly"];
        let examples = vec![
            ("hello", "larry"),
            ("helloworld", "curly"),
            ("world", "moe"),
            ("blah", "curly"),
        ];
        for (key, v) in examples {
            let ret = key2volume(key.as_bytes(), &volumes, 1, 3);
            assert_eq!(ret[0].split("/").nth(0).unwrap(), v);
        }
    }
    #[test]
    fn test_fromtorecord() {
        from_to_record_example(
            Record {
                rvolumes: vec!["hello", "world"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                deleted: Deleted::SOFT,
                hash: "".to_owned(),
            },
            "DELETEDhello,world".to_owned(),
        );
        from_to_record_example(
            Record {
                rvolumes: vec!["hello", "world"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                deleted: Deleted::NO,
                hash: "".to_owned(),
            },
            "hello,world".to_owned(),
        );
        from_to_record_example(
            Record {
                rvolumes: vec!["hello"].iter().map(|s| s.to_string()).collect(),
                deleted: Deleted::NO,
                hash: "".to_owned(),
            },
            "hello".to_owned(),
        );
        from_to_record_example(
            Record {
                rvolumes: vec!["hello", "world"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                deleted: Deleted::NO,
                hash: "".to_owned(),
            },
            "hello,world".to_owned(),
        );
        from_to_record_example(
            Record {
                rvolumes: vec!["hello".to_owned()],
                deleted: Deleted::NO,
                hash: "5d41402abc4b2a76b9719d911017c592".to_owned(),
            },
            "HASH5d41402abc4b2a76b9719d911017c592hello".to_owned(),
        );
        from_to_record_example(
            Record {
                rvolumes: vec!["hello".to_owned()],
                deleted: Deleted::SOFT,
                hash: "5d41402abc4b2a76b9719d911017c592".to_owned(),
            },
            "DELETEDHASH5d41402abc4b2a76b9719d911017c592hello".to_owned(),
        );
    }
}
