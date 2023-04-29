use fluvio_smartmodule::{smartmodule, eyre, Record, RecordData, Result};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use once_cell::sync::OnceCell;

#[derive(Default, Debug)]
struct StarsForks {
    stars: u32,
    forks: u32,
}

#[derive(Default, Deserialize)]
pub struct GithubRecord {
    stars: u32,
    forks: u32,
}

#[derive(Default, Serialize)]
pub struct GithubString {
    result: String
}

fn stars_forks() -> &'static Mutex<StarsForks> {
    static STARS_FORKS: OnceCell<Mutex<StarsForks>> = OnceCell::new();
    STARS_FORKS.get_or_init(|| {
        Mutex::new(StarsForks::default())
    })
}

impl GithubRecord {
    pub fn to_emoji_string(&self, stars: u32, forks: u32) -> Option<GithubString> {
        if stars == 0 && forks == 0 {
            None
        } else if stars != self.stars && forks !=self.forks {
            Some(GithubString::new(format!(":gitfork: {} \n:star2: {}", self.forks, self.stars )))
        } else if forks != self.forks {
            Some(GithubString::new(format!(":gitfork: {}", self.forks)))
        } else if stars != self.stars {
            Some(GithubString::new(format!(":star2: {}", self.stars)))
        } else { // no changes
            None
        }
    }
}

impl GithubString {
    pub fn new(result: String) -> Self {
        Self { result }
    }
}

#[smartmodule(filter_map)]
pub fn filter_map(record: &Record) -> Result<Option<(Option<RecordData>, RecordData)>> {
    let record_json: GithubRecord = serde_json::from_slice(record.value.as_ref())?;

    let star_forks_mutex = stars_forks();
    let mut star_forks = star_forks_mutex.lock().map_err(|e| eyre!("lock poisioned {e}"))?;
    let string_result = record_json.to_emoji_string(star_forks.stars, star_forks.forks);

    star_forks.stars = record_json.stars;
    star_forks.forks = record_json.forks;
    
    if let Some (result) = string_result {
        let output = serde_json::to_vec(&result)?;
        Ok(Some((record.key.clone(), output.into()))) 
    } else {
        Ok(None)
    }
}
