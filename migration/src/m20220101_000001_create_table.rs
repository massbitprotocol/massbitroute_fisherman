use entity::jobs;
use entity::providers;
use entity::*;
use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{ConnectionTrait, Statement};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220101_000001_create_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        /*
                // JobAssignments table
                manager
                    .create_table(
                        sea_query::Table::create()
                            .table(job_assignments::Entity)
                            .if_not_exists()
                            .col(
                                ColumnDef::new(job_assignments::Column::Id)
                                    .integer()
                                    .not_null()
                                    .auto_increment()
                                    .primary_key(),
                            )
                            // .col(ColumnDef::new(jobs::Column::JobId).string().not_null())
                            .col(
                                ColumnDef::new(job_assignments::Column::JobId)
                                    .string()
                                    .not_null(),
                            )
                            .col(
                                ColumnDef::new(job_assignments::Column::WorkerId)
                                    .string()
                                    .not_null(),
                            )
                            .col(
                                ColumnDef::new(job_assignments::Column::PlanId)
                                    .string()
                                    .not_null(),
                            )
                            .col(
                                ColumnDef::new(job_assignments::Column::Status)
                                    .string()
                                    .not_null()
                                    .default(""),
                            )
                            .col(
                                ColumnDef::new(job_assignments::Column::AssignTime)
                                    .big_integer()
                                    .not_null(),
                            )
                            .col(
                                ColumnDef::new(job_assignments::Column::JobName)
                                    .string()
                                    .not_null()
                                    .default(""),
                            )
                            .col(
                                ColumnDef::new(job_assignments::Column::JobType)
                                    .string()
                                    .not_null()
                                    .default(""),
                            )
                            .to_owned(),
                    )
                    .await?;

                // JobResultBenchmarks table
                manager
                    .create_table(
                        sea_query::Table::create()
                            .table(job_result_benchmarks::Entity)
                            .if_not_exists()
                            .col(
                                ColumnDef::new(job_result_benchmarks::Column::Id)
                                    .integer()
                                    .not_null()
                                    .auto_increment()
                                    .primary_key(),
                            )
                            .col(
                                ColumnDef::new(job_result_benchmarks::Column::JobId)
                                    .string()
                                    .not_null(),
                            )
                            .col(
                                ColumnDef::new(job_result_benchmarks::Column::WorkerId)
                                    .string()
                                    .not_null(),
                            )
                            .col(
                                ColumnDef::new(job_result_benchmarks::Column::PrviderId)
                                    .string()
                                    .not_null(),
                            )
                            .col(
                                ColumnDef::new(job_result_benchmarks::Column::ProviderType)
                                    .string()
                                    .not_null(),
                            )
                            .col(
                                ColumnDef::new(job_result_benchmarks::Column::ExecutionTimestamp)
                                    .big_integer()
                                    .not_null()
                                    .default(0),
                            )
                            .col(
                                ColumnDef::new(job_result_benchmarks::Column::RecordedTimestamp)
                                    .integer()
                                    .not_null()
                                    .default(0),
                            )
                            .col(
                                ColumnDef::new(job_result_benchmarks::Column::RequestRate)
                                    .float()
                                    .not_null()
                                    .default(0),
                            )
                            .col(
                                ColumnDef::new(job_result_benchmarks::Column::TransferRate)
                                    .float()
                                    .not_null()
                                    .default(0),
                            )
                            .col(
                                ColumnDef::new(job_result_benchmarks::Column::AverageLatency)
                                    .float()
                                    .not_null()
                                    .default(0),
                            )
                            .col(
                                ColumnDef::new(job_result_benchmarks::Column::Histogram90)
                                    .float()
                                    .not_null()
                                    .default(0),
                            )
                            .col(
                                ColumnDef::new(job_result_benchmarks::Column::Histogram95)
                                    .float()
                                    .not_null()
                                    .default(0),
                            )
                            .col(
                                ColumnDef::new(job_result_benchmarks::Column::Histogram99)
                                    .float()
                                    .not_null()
                                    .default(0),
                            )
                            .col(
                                ColumnDef::new(job_result_benchmarks::Column::ErrorCode)
                                    .integer()
                                    .not_null()
                                    .default(0),
                            )
                            .col(
                                ColumnDef::new(job_result_benchmarks::Column::message)
                                    .string()
                                    .not_null(),
                            )
                            .col(
                                ColumnDef::new(job_result_benchmarks::Column::ResponseTime)
                                    .integer()
                                    .not_null(),
                            )
                            .col(
                                ColumnDef::new(job_result_benchmarks::Column::PlanId)
                                    .string()
                                    .not_null(),
                            )
                            .to_owned(),
                    )
                    .await?;

                // JobResultBenchmarks table
                manager
                    .create_table(
                        sea_query::Table::create()
                            .table(job_result_http_requests::Entity)
                            .if_not_exists()
                            .col(
                                ColumnDef::new(job_result_benchmarks::Column::Id)
                                    .integer()
                                    .not_null()
                                    .auto_increment()
                                    .primary_key(),
                            )
                            .col(
                                ColumnDef::new(job_result_benchmarks::Column::JobId)
                                    .string()
                                    .not_null(),
                            )
                            .col(
                                ColumnDef::new(job_result_benchmarks::Column::JobName)
                                    .string()
                                    .not_null(),
                            )
                            .col(
                                ColumnDef::new(job_result_benchmarks::Column::WorkerId)
                                    .string()
                                    .not_null(),
                            )
                            .col(
                                ColumnDef::new(job_result_benchmarks::Column::PrviderId)
                                    .string()
                                    .not_null(),
                            )
                            .col(
                                ColumnDef::new(job_result_benchmarks::Column::ProviderType)
                                    .string()
                                    .not_null(),
                            )
                            .col(
                                ColumnDef::new(job_result_benchmarks::Column::ExecutionTimestamp)
                                    .big_integer()
                                    .not_null()
                                    .default(0),
                            )
                            .col(
                                ColumnDef::new(job_result_benchmarks::Column::ChainId)
                                    .string()
                                    .not_null()
                            )
                            .col(
                                ColumnDef::new(job_result_benchmarks::Column::PlanId)
                                    .string()
                                    .not_null()
                            )
                            .col(
                                ColumnDef::new(job_result_benchmarks::Column::HttpCode)
                                    .integer()
                                    .not_null()
                                    .default(0),
                            )
                            .col(
                                ColumnDef::new(job_result_benchmarks::Column::ErrorCode)
                                    .integer()
                                    .not_null()
                                    .default(0),
                            )
                            .col(
                                ColumnDef::new(job_result_benchmarks::Column::message)
                                    .string()
                                    .not_null(),
                            )
                            .col(
                                ColumnDef::new(job_result_benchmarks::Column::ResponseTime)
                                    .integer()
                                    .not_null(),
                            )
                            .col(
                                ColumnDef::new(job_result_benchmarks::Column::Values)
                                    .json_binary()
                                    .not_null(),
                                    .default("{}")
                            )
                            .to_owned(),
                    )
                    .await?;

                // Job table
                manager
                    .create_table(
                        sea_query::Table::create()
                            .table(jobs::Entity)
                            .if_not_exists()
                            .col(
                                ColumnDef::new(jobs::Column::Id)
                                    .integer()
                                    .not_null()
                                    .auto_increment()
                                    .primary_key(),
                            )
                            .col(ColumnDef::new(jobs::Column::JobId).string().not_null())
                            .col(
                                ColumnDef::new(jobs::Column::ComponentId)
                                    .string()
                                    .not_null(),
                            )
                            .col(
                                ColumnDef::new(jobs::Column::Header)
                                    .json_binary()
                                    .default("{}"),
                            )
                            .col(
                                ColumnDef::new(jobs::Column::JobDetail)
                                    .json_binary()
                                    .default("{}"),
                            )
                            .col(
                                ColumnDef::new(jobs::Column::Priority)
                                    .big_integer()
                                    .not_null(),
                            )
                            .col(
                                ColumnDef::new(jobs::Column::ExpectedRuntime)
                                    .integer()
                                    .not_null()
                                    .default(0),
                            )
                            .col(
                                ColumnDef::new(jobs::Column::Parallelable)
                                    .boolean()
                                    .not_null(),
                            )
                            .col(
                                ColumnDef::new(jobs::Column::Timeout)
                                    .big_integer()
                                    .not_null(),
                            )
                            .col(
                                ColumnDef::new(jobs::Column::ComponentUrl)
                                    .string()
                                    .not_null(),
                            )
                            .col(
                                ColumnDef::new(jobs::Column::RepeatNumber)
                                    .integer()
                                    .not_null(),
                            )
                            .col(
                                ColumnDef::new(jobs::Column::Interval)
                                    .big_integer()
                                    .not_null(),
                            )
                            .col(ColumnDef::new(jobs::Column::JobName).string().not_null())
                            .col(ColumnDef::new(jobs::Column::PlanId).string().not_null())
                            .col(
                                ColumnDef::new(jobs::Column::ComponentType)
                                    .string()
                                    .not_null(),
                            )
                            .col(ColumnDef::new(jobs::Column::Phase).string().not_null())
                            .to_owned(),
                    )
                    .await?;
                manager
                    .create_table(
                        sea_query::Table::create()
                            .table(providers::Entity)
                            .if_not_exists()
                            .col(
                                ColumnDef::new(providers::Column::Id)
                                    .integer()
                                    .not_null()
                                    .auto_increment()
                                    .primary_key(),
                            )
                            .col(ColumnDef::new(providers::Column::Url).string().not_null())
                            .col(ColumnDef::new(providers::Column::Zone).string().not_null())
                            .col(
                                ColumnDef::new(providers::Column::Specification)
                                    .json_binary()
                                    .default("{}"),
                            )
                            .col(ColumnDef::new(providers::Column::Description).text())
                            .to_owned(),
                    )
                    .await?;
        */

        let sqls = r#"
create table if not exists workers
(
    id            serial
        primary key,
    url           varchar           not null,
    zone          varchar           not null,
    specification jsonb   default '{}'::jsonb,
    description   text,
    worker_id     varchar           not null,
    worker_ip     varchar           not null,
    active        integer default 1 not null
);


create table if not exists providers
(
    id            serial
        primary key,
    url           varchar not null,
    zone          varchar not null,
    specification jsonb default '{}'::jsonb,
    description   text
);


create table if not exists jobs
(
    id               serial
        primary key,
    job_id           varchar                                   not null,
    job_type         varchar default ''::character varying     not null,
    job_name         varchar default ''::character varying     not null,
    component_id     varchar                                   not null,
    header           jsonb   default '{}'::jsonb,
    job_detail       jsonb   default '{}'::jsonb,
    priority         integer default 0                         not null,
    expected_runtime bigint  default 0                         not null,
    parallelable     boolean default true                      not null,
    timeout          bigint  default 1000                      not null,
    component_url    varchar default ''::character varying     not null,
    repeat_number    integer default 0                         not null,
    interval         bigint  default 1000                      not null,
    plan_id          varchar default ''::character varying     not null,
    component_type   varchar default 'node'::character varying not null,
    phase            varchar                                   not null
);


create table if not exists job_result_benchmarks
(
    id                  bigserial
        constraint job_result_benchmarks_pk
            primary key,
    job_id              varchar                                        not null,
    worker_id           varchar                                        not null,
    provider_id         varchar                                        not null,
    provider_type       varchar                                        not null,
    execution_timestamp bigint           default 0                     not null,
    recorded_timestamp  bigint           default 0                     not null,
    request_rate        double precision default 0                     not null,
    transfer_rate       double precision default 0                     not null,
    average_latency     double precision default 0                     not null,
    histogram90         double precision default 0                     not null,
    histogram95         double precision default 0                     not null,
    histogram99         double precision default 0                     not null,
    error_code          integer          default 0                     not null,
    message             varchar          default ''::character varying not null,
    response_duration       integer          default 0                     not null,
    plan_id             varchar          default ''::character varying not null
);


create table if not exists job_result_pings
(
    id                  bigserial
        constraint job_result_pings_pk
            primary key,
    job_id              varchar                               not null,
    worker_id           varchar                               not null,
    provider_id         varchar                               not null,
    provider_type       varchar                               not null,
    execution_timestamp bigint  default 0                     not null,
    recorded_timestamp  bigint  default 0                     not null,
    plan_id             varchar default ''::character varying not null,
    response_durations      jsonb   default '[]'::jsonb           not null,
    error_number        bigint  default 0                     not null
);


create table if not exists job_result_latest_blocks
(
    id                  bigserial
        constraint job_result_latest_blocks_pk
            primary key,
    job_id              varchar                               not null,
    worker_id           varchar                               not null,
    provider_id         varchar                               not null,
    provider_type       varchar                               not null,
    execution_timestamp bigint  default 0                     not null,
    chain_id            varchar                               not null,
    block_number        bigint  default '-1'::integer         not null,
    block_timestamp     bigint  default 0                     not null,
    plan_id             varchar default ''::character varying not null,
    block_hash          varchar default ''::character varying not null,
    http_code           integer                               not null,
    error_code          integer                               not null,
    message             varchar default ''::character varying not null,
    response_duration       bigint  default 0                     not null
);


create table if not exists plans
(
    id           bigserial
        constraint plans_pk
            primary key,
    plan_id      varchar          not null,
    provider_id  varchar          not null,
    request_time bigint           not null,
    finish_time  bigint,
    result       varchar,
    message      varchar,
    status       varchar          not null,
    phase        varchar          not null,
    expiry_time  bigint default 0 not null
);


create table if not exists job_assignments
(
    id          serial
        primary key,
    job_id      varchar                               not null,
    worker_id   varchar                               not null,
    plan_id     varchar                               not null,
    status      varchar default ''::character varying not null,
    assign_time bigint                                not null,
    job_name    varchar default ''::character varying not null,
    job_type    varchar default ''::character varying not null
);


create table if not exists worker_provider_maps
(
    id                 serial
        primary key,
    worker_id          varchar not null,
    provider_id        varchar not null,
    ping_response_duration integer,
    ping_timestamp          bigint,
    bandwidth          bigint,
    bandwidth_timestamp     bigint,
    status             integer,
    last_connect_time  bigint,
    last_check         bigint,
    constraint worker_provider_maps_worker_provider_key
        unique (worker_id, provider_id)
);


create table if not exists job_result_http_requests
(
    id                  bigserial
        constraint job_result_http_requests_pk
            primary key,
    job_id              varchar                               not null,
    job_name            varchar                               not null,
    worker_id           varchar                               not null,
    provider_id         varchar                               not null,
    provider_type       varchar                               not null,
    execution_timestamp bigint  default 0                     not null,
    chain_id            varchar                               not null,
    plan_id             varchar default ''::character varying not null,
    http_code           integer                               not null,
    error_code          integer                               not null,
    message             varchar default ''::character varying not null,
    values              jsonb   default '{}'::jsonb           not null,
    response_duration       bigint  default 0                     not null
);
"#;
        let sqls = sqls.split(";");
        for sql in sqls {
            println!("sql: {}", sql);
            let stmt = Statement::from_string(manager.get_database_backend(), sql.to_owned());
            manager.get_connection().execute(stmt).await.map(|_| ())?;
        }
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        todo!()
    }
}
