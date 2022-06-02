table! {
    job_assignments (id) {
        id -> Int4,
        url -> Varchar,
        zone -> Varchar,
        specification -> Nullable<Jsonb>,
        description -> Nullable<Text>,
    }
}

table! {
    jobs (id) {
        id -> Int4,
        url -> Varchar,
        zone -> Varchar,
        specification -> Nullable<Jsonb>,
        description -> Nullable<Text>,
    }
}

table! {
    providers (id) {
        id -> Int4,
        url -> Varchar,
        zone -> Varchar,
        specification -> Nullable<Jsonb>,
        description -> Nullable<Text>,
    }
}

table! {
    workers (id) {
        id -> Int4,
        url -> Varchar,
        zone -> Varchar,
        specification -> Nullable<Jsonb>,
        description -> Nullable<Text>,
    }
}

allow_tables_to_appear_in_same_query!(
    job_assignments,
    jobs,
    providers,
    workers,
);
