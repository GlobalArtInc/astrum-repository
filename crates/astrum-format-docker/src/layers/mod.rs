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

use std::time::Duration;

use astrum_content::inventory::PackageRef;
use astrum_content::{ContentError, Ctx};

use crate::manifest::Layer;
use crate::upstream::Upstreams;

use self::facts::LayerFacts;
use self::limits::Limits;

pub const EXTRACTOR: i32 = 1;
const BUDGET: Duration = Duration::from_secs(570);
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
) -> Result<Contribution, ContentError> {
    if layers.len() > MAX_LAYERS {
        return Ok(Contribution::incomplete(format!(
            "the image has more than {MAX_LAYERS} layers"
        )));
    }
    let mut facts: Vec<LayerFacts> = Vec::with_capacity(layers.len());
    for layer in layers {
        facts.push(access::facts_of(ctx, upstreams, image, layer, limits).await?);
    }
    let merged = merge::merge(&facts, limits.max_packages);
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
) -> Result<Contribution, ContentError> {
    let work = collect(ctx, upstreams, image, layers, Limits::DEFAULT);
    match tokio::time::timeout(BUDGET, work).await {
        Ok(result) => result,
        Err(_) => Ok(Contribution::incomplete(format!(
            "reading the layers took longer than {} seconds",
            BUDGET.as_secs()
        ))),
    }
}
