mod executor;
mod wsclient_builder;

use anyhow::anyhow;
pub use executor::*;
use std::fmt::Debug;

pub(crate) fn toanyhowerror<E>(e: E) -> anyhow::Error
where
    E: Debug,
{
    anyhow!("{:?}", &e)
}
