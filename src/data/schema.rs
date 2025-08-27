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

diesel::allow_tables_to_appear_in_same_query!(courses, course_metrics, course_audits,);
