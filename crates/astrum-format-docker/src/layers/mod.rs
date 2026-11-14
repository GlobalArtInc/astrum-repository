mod access;
mod apk;
mod cache;
mod distro;
mod dpkg;
mod entries;
mod facts;
mod limits;
mod merge;
mod paths;
mod records;
mod stream;

#[cfg(test)]
mod fixtures;

use std::time::{Duration, Instant};

use astrum_content::inventory::PackageRef;
use astrum_content::{ContentError, Ctx};

use crate::manifest::Layer;
use crate::upstream::Upstreams;

use self::limits::Limits;
use self::merge::Merge;

pub const EXTRACTOR: i32 = 2;
const SLACK: Duration = Duration::from_secs(30);
const MAX_LAYERS: usize = 128;

pub struct Contribution {
    pub packages: Vec<PackageRef>,
    pub complete: bool,
    pub reasons: Vec<String>,
}

impl Contribution {
    fn incomplete(reason: String) -> Self {
        Contribution {
            packages: Vec::new(),
            complete: false,
            reasons: vec![reason],
        }
    }
}

async fn collect(
    ctx: &Ctx,
    upstreams: &Upstreams,
    image: &str,
    layers: &[Layer],
    limits: Limits,
    deadline: Instant,
) -> Result<Contribution, ContentError> {
    if layers.len() > MAX_LAYERS {
        return Ok(Contribution::incomplete(format!(
            "the image has more than {MAX_LAYERS} layers"
        )));
    }
    let mut merge = Merge::new(&limits);
    for layer in layers {
        let facts = access::facts_of(ctx, upstreams, image, layer, limits, deadline).await?;
        merge.apply(facts);
    }
    let merged = merge.finish();
    Ok(Contribution {
        packages: merged.packages,
        complete: merged.complete,
        reasons: merged.reasons,
    })
}

pub async fn read(
    ctx: &Ctx,
    upstreams: &Upstreams,
    image: &str,
    layers: &[Layer],
    scan_deadline: Instant,
) -> Result<Contribution, ContentError> {
    let remaining = scan_deadline
        .saturating_duration_since(Instant::now())
        .saturating_sub(SLACK);
    if remaining.is_zero() {
        return Ok(Contribution::incomplete(
            "there was no time left to read the layers".to_owned(),
        ));
    }
    let deadline = Instant::now() + remaining;
    let work = collect(ctx, upstreams, image, layers, Limits::DEFAULT, deadline);
    match tokio::time::timeout(remaining, work).await {
        Ok(result) => result,
        Err(_) => Ok(Contribution::incomplete(format!(
            "reading the layers took longer than {} seconds",
            remaining.as_secs()
        ))),
    }
}
