use crate::{app::App, key2volume, Record};

struct File {
    name: String,
    filetype: String,
    mtime: String,
}
struct RebuildRequest {
    vol: String,
    url: String,
}
fn rebuild(a: &mut App, vol: String, name: String) -> bool {
    let key = base64::decode(&name).unwrap();
    let kvolumes = key2volume(&key, &(&a.volumes), a.replicas, a.subvolumes);
    if !a.lockKey(&key) {
        return false;
    }
    let rec = {
        match a.getRecord(&key) {
            None => Record::default(),
            Some(mut rec) => {
                rec.rvolumes.push(vol);
                rec
            }
        }
    };
    // sort by order in kvolumes (sorry it's n^2 but n is small)
    let pvalues = kvolumes
        .iter()
        .zip(rec.rvolumes.iter())
        .filter(|(v, v2)| v == v2)
        .map(|(_, v)| v.to_owned())
        .collect::<Vec<_>>();
    if !a.putRecord(
        &key,
        Record {
            rvolumes: pvalues.clone(),
            deleted: crate::Deleted::NO,
            hash: "".to_owned(),
        },
    ) {
        return false;
    }
    println!("{:?},{:?}", &key, &pvalues);
    a.unlockKey(&key);
    true
}
