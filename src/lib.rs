use std::sync::atomic::{AtomicU32, Ordering};

use once_cell::sync::OnceCell;

use fluvio_smartmodule::{
    dataplane::smartmodule::SmartModuleExtraParams, eyre, smartmodule, Record, RecordData, Result,
};
use serde::{Deserialize, Serialize};

// use u32 to represent the metric
type METRIC = u32;

/// Incoming record from Github
#[derive(Default, Deserialize)]
struct GithubRecord {
    stars: u32,
    forks: u32,
}

/// Outgoing record
#[derive(Default, Serialize)]
struct GithubOutgoing {
    #[serde(skip_serializing_if = "Option::is_none")]
    stars: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    forks: Option<u32>,
}

/// Accumulator for stars and forks
/// Use AtomicU32 to update internal state
#[derive(Default, Debug, Deserialize)]
struct StarsForks {
    stars: AtomicU32,
    forks: AtomicU32,
}

impl StarsForks {
    fn get_stars(&self) -> METRIC {
        self.stars.load(Ordering::SeqCst)
    }

    fn set_stars(&self, new: METRIC) {
        self.stars.store(new, Ordering::SeqCst);
    }

    fn get_forks(&self) -> METRIC {
        self.forks.load(Ordering::SeqCst)
    }

    fn set_forks(&self, new: METRIC) {
        self.forks.store(new, Ordering::SeqCst);
    }

    // generate emoji string based on the new stars and forks
    fn update_and_generate_moji_string(&self, new: &GithubRecord) -> Option<GithubOutgoing> {
        let current_stars = self.get_stars();
        let current_forks = self.get_forks();

        if new.stars == 0 && new.forks == 0 {
            // if no stars and forks, return None
            None
        } else if new.stars != current_stars && new.forks != current_forks {
            // if both stars and forks are changed, generate new emoji on prev stats
            let emoji = GithubOutgoing {
                stars: Some(current_stars),
                forks: Some(current_forks),
            };
            self.set_forks(new.forks);
            self.set_stars(new.stars);
            Some(emoji)
        } else if new.forks != current_forks {
            // if only forks are changed, generate new emoji on prev stats
            let emoji = GithubOutgoing {
                stars: None,
                forks: Some(current_forks),
            };
            self.set_forks(new.forks);
            Some(emoji)
        } else if new.stars != current_stars {
            let emoji = GithubOutgoing {
                stars: Some(current_stars),
                forks: None,
            };
            self.set_stars(new.stars);
            Some(emoji)
        } else {
            // no changes
            None
        }
    }
}

static STARS_FORKS: OnceCell<StarsForks> = OnceCell::new();

#[smartmodule(init)]
fn init(_params: SmartModuleExtraParams) -> Result<()> {
    STARS_FORKS
        .set(StarsForks::default())
        .map_err(|err| eyre!("regex init: {:#?}", err))
}

#[smartmodule(filter_map)]
pub fn filter_map(record: &Record) -> Result<Option<(Option<RecordData>, RecordData)>> {
    let new_data: GithubRecord = serde_json::from_slice(record.value.as_ref())?;

    let accumulator = STARS_FORKS.get().unwrap();

    if let Some(emoji) = accumulator.update_and_generate_moji_string(&new_data) {
        let output = serde_json::to_vec(&emoji)?;
        // need to update
        Ok(Some((record.key.clone(), output.into())))
    } else {
        Ok(None)
    }
}
