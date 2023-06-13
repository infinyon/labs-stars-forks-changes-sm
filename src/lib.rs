use std::sync::atomic::{AtomicU32, Ordering};

use once_cell::sync::OnceCell;

use fluvio_smartmodule::{
    dataplane::smartmodule::SmartModuleExtraParams,
    eyre, smartmodule, Record, RecordData, Result,
};
use serde::{Deserialize, Serialize};

// use u32 to represent the metric
type Metric = u32;
type AtomicMetric = AtomicU32;

/// Incoming record from Github
#[derive(Default, Deserialize)]
struct GithubRecord {
    stars: Metric,
    forks: Metric,
}

/// Outgoing record
#[derive(Default, Serialize)]
struct GithubOutgoing {
    result: String
}

static STARS_FORKS: OnceCell<StarsForks> = OnceCell::new();

/// Accumulator for stars and forks
/// Use Atomic to update internal state
#[derive(Default, Debug, Deserialize)]
struct StarsForks {
    stars: AtomicMetric,
    forks: AtomicMetric,
}

impl StarsForks {
    fn new(github_record: GithubRecord) -> StarsForks {
        StarsForks {
            stars: AtomicMetric::new(github_record.stars),
            forks: AtomicMetric::new(github_record.forks)
        }
    }

    fn get_stars(&self) -> Metric {
        self.stars.load(Ordering::SeqCst)
    }

    fn set_stars(&self, new: Metric) {
        self.stars.store(new, Ordering::SeqCst);
    }

    fn get_forks(&self) -> Metric {
        self.forks.load(Ordering::SeqCst)
    }

    fn set_forks(&self, new: Metric) {
        self.forks.store(new, Ordering::SeqCst);
    }

    // generate emoji string based on the new stars and forks
    fn update_and_generate_moji_string(&self, new: &GithubRecord) -> Option<GithubOutgoing> {
        let current_stars = self.get_stars();
        let current_forks = self.get_forks();

        if new.stars != current_stars && new.forks != current_forks {
            // if both stars and forks are changed, generate new emoji on prev stats
            let emoji = GithubOutgoing {
                result: format!(":flags: {} \n:star2: {}", new.forks, new.stars )
            };
            self.set_forks(new.forks);
            self.set_stars(new.stars);
            Some(emoji)
        } else if new.forks != current_forks {
            // if only forks are changed, generate new emoji on prev stats
            let emoji = GithubOutgoing {
                result: format!(":flags: {}", new.forks)
            };
            self.set_forks(new.forks);
            Some(emoji)
        } else if new.stars != current_stars {
            let emoji = GithubOutgoing {
                result: format!(":star2: {}", new.stars)
            };
            self.set_stars(new.stars);
            Some(emoji)
        } else {
            // no changes
            None
        }
    }
}

#[smartmodule(init)]
fn init(_params: SmartModuleExtraParams) -> Result<()> {
    STARS_FORKS
        .set(StarsForks::default())
        .map_err(|err| eyre!("init error: {:#?}", err))
}

#[smartmodule(look_back)]
pub fn look_back(record: &Record) -> Result<()> {
    let last_value = serde_json::from_slice(record.value.as_ref())?;

    STARS_FORKS
        .set(StarsForks::new(last_value))
        .map_err(|err| eyre!("init error: {:#?}", err))
}

#[smartmodule(filter_map)]
pub fn filter_map(record: &Record) -> Result<Option<(Option<RecordData>, RecordData)>> {
    let new_data: GithubRecord = serde_json::from_slice(record.value.as_ref())?;

    let accumulator = STARS_FORKS.get().unwrap();

    if let Some(emoji) = accumulator.update_and_generate_moji_string(&new_data) {
        let output = serde_json::to_vec(&emoji)?;
        Ok(Some((record.key.clone(), output.into())))
    } else {
        Ok(None)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_updated_and_generate_emoji_string() {
        let accum = StarsForks::new(GithubRecord { stars: 1723, forks: 134});
        
        // first record sets-up accumulator - no changes        
        let mut record = GithubRecord { stars: 1723, forks: 134};
        assert!(accum.update_and_generate_moji_string(&record).is_none());

        // same values - no changes        
        record = GithubRecord { stars: 1723, forks: 134};
        assert!(accum.update_and_generate_moji_string(&record).is_none());

        // forks changed
        record = GithubRecord { stars: 1723, forks: 135};
        assert_eq!(
            accum.update_and_generate_moji_string(&record).unwrap().result,
            format!(":flags: 135")
        );

        // stars changed
        record = GithubRecord { stars: 1724, forks: 135};
        assert_eq!(
            accum.update_and_generate_moji_string(&record).unwrap().result,
            format!(":star2: 1724")
        );

        // both changed
        record = GithubRecord { stars: 1723, forks: 134};
        assert_eq!(
            accum.update_and_generate_moji_string(&record).unwrap().result,
            format!(":flags: 134 \n:star2: 1723")
        );

        // same values - no changes        
        record = GithubRecord { stars: 1723, forks: 134};
        assert!(accum.update_and_generate_moji_string(&record).is_none());

    }
}