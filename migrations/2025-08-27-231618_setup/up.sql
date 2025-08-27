-- Your SQL goes here
CREATE TABLE "courses"(
	"id" INT4 NOT NULL PRIMARY KEY,
	"crn" VARCHAR NOT NULL,
	"subject" VARCHAR NOT NULL,
	"course_number" VARCHAR NOT NULL,
	"title" VARCHAR NOT NULL,
	"term_code" VARCHAR NOT NULL,
	"enrollment" INT4 NOT NULL,
	"max_enrollment" INT4 NOT NULL,
	"wait_count" INT4 NOT NULL,
	"wait_capacity" INT4 NOT NULL,
	"last_scraped_at" TIMESTAMPTZ NOT NULL
);

CREATE TABLE "course_metrics"(
	"id" INT4 NOT NULL PRIMARY KEY,
	"course_id" INT4 NOT NULL,
	"timestamp" TIMESTAMPTZ NOT NULL,
	"enrollment" INT4 NOT NULL,
	"wait_count" INT4 NOT NULL,
	"seats_available" INT4 NOT NULL,
	FOREIGN KEY ("course_id") REFERENCES "courses"("id")
);

CREATE TABLE "course_audits"(
	"id" INT4 NOT NULL PRIMARY KEY,
	"course_id" INT4 NOT NULL,
	"timestamp" TIMESTAMPTZ NOT NULL,
	"field_changed" VARCHAR NOT NULL,
	"old_value" TEXT NOT NULL,
	"new_value" TEXT NOT NULL,
	FOREIGN KEY ("course_id") REFERENCES "courses"("id")
);

