#![allow(unused)]
#![allow(clippy::all)]

use super::schema::*;
use diesel::insert_into;
use diesel::pg::Pg;
use diesel::serialize::Output;
use diesel::sql_types::{Jsonb, Uuid};
use diesel::types::{FromSql, ToSql};
use diesel::Queryable;
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use std::io::Write;
#[derive(Debug, Deserialize, Serialize, Queryable, Insertable)]
pub struct Worker {
    pub id: String,
    pub url: String,
    pub zone: String,
    pub specification: WorkerSpec,
    pub description: String,
}

#[derive(FromSqlRow, AsExpression, serde::Serialize, serde::Deserialize, Debug, Default)]
#[sql_type = "Jsonb"]
pub struct WorkerSpec {
    pub cpus: Option<u16>,
    pub ram: Option<u32>,
    pub bandwidth: Option<u32>,
}

impl FromSql<Jsonb, Pg> for Worker {
    fn from_sql(bytes: Option<&[u8]>) -> diesel::deserialize::Result<Self> {
        let value = <serde_json::Value as FromSql<Jsonb, Pg>>::from_sql(bytes)?;
        Ok(serde_json::from_value(value)?)
    }
}

impl ToSql<Jsonb, Pg> for Worker {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> diesel::serialize::Result {
        let value = serde_json::to_value(self)?;
        <serde_json::Value as ToSql<Jsonb, Pg>>::to_sql(&value, out)
    }
}
