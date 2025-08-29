pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "scrape_priority"))]
    pub struct ScrapePriority;

    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "target_type"))]
    pub struct TargetType;
}

use super::models::{ScrapePriorityMapping, TargetTypeMapping};

diesel::table! {
    use diesel::sql_types::*;
    use super::{ScrapePriorityMapping, TargetTypeMapping};

    scrape_jobs (id) {
        id -> Int4,
        target_type -> TargetTypeMapping,
        target_payload -> Jsonb,
        priority -> ScrapePriorityMapping,
        execute_at -> Timestamptz,
        created_at -> Timestamptz,
        locked_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    courses (id) {
        id -> Int4,
        crn -> Varchar,
        subject -> Varchar,
        course_number -> Varchar,
        title -> Varchar,
        term_code -> Varchar,
        enrollment -> Int4,
        max_enrollment -> Int4,
        wait_count -> Int4,
        wait_capacity -> Int4,
        last_scraped_at -> Timestamptz,
    }
}

diesel::table! {
    course_metrics (id) {
        id -> Int4,
        course_id -> Int4,
        timestamp -> Timestamptz,
        enrollment -> Int4,
        wait_count -> Int4,
        seats_available -> Int4,
    }
}

diesel::table! {
    course_audits (id) {
        id -> Int4,
        course_id -> Int4,
        timestamp -> Timestamptz,
        field_changed -> Varchar,
        old_value -> Text,
        new_value -> Text,
    }
}

diesel::joinable!(course_metrics -> courses (course_id));
diesel::joinable!(course_audits -> courses (course_id));

diesel::allow_tables_to_appear_in_same_query!(courses, course_metrics, course_audits, scrape_jobs,);
