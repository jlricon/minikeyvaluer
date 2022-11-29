use http::StatusCode;
use leveldb::{kv::KV, options::WriteOptions};
use minikeyvaluer::{key2path, remote_delete, Deleted, Record};

use crate::app::App;

struct ListResponse {
    next: String,
    keys: Vec<String>,
}
impl App {
    fn delete(&mut self, key: &[u8], unlink: bool) -> StatusCode {
        let rec = self.getRecord(key).unwrap();
        let maybe_status = match rec.deleted {
            // TODO: Check if the unlink applies to the SOFT only or everything
            Deleted::HARD => Some(StatusCode::NOT_FOUND),
            Deleted::SOFT if unlink => Some(StatusCode::NOT_FOUND),
            Deleted::NO if !unlink && self.protect => Some(StatusCode::FORBIDDEN),
            _ => None,
        };
        if let Some(status) = maybe_status {
            return status;
        }
        let new_record = Record {
            deleted: Deleted::SOFT,
            ..rec
        };
        let rvolumes = new_record.rvolumes.clone();
        if !self.putRecord(key, new_record) {
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
        if !unlink {
            let delete_error = rvolumes
                .iter()
                .map(|v| {
                    let remote = format!("http://{}{}", v, key2path(key));

                    remote_delete(&remote).map_or_else(|v| true, |v| false)
                })
                .any(|f| f);
            if delete_error {
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
            self.db
                .delete(WriteOptions::new(), App::bytes_to_i32(key))
                .unwrap();
        }
        return StatusCode::NO_CONTENT;
    }
}
