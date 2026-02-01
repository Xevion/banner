// columns.ts
import type { CourseResponse } from "$lib/api";
import type { ColumnDef } from "@tanstack/table-core";
import type { Component } from "svelte";
import {
  abbreviateInstructor,
  formatLocationDisplay,
  formatMeetingDays,
  formatTimeRange,
  getPrimaryInstructor,
  openSeats,
} from "$lib/course";

import CrnCell from "./cells/CrnCell.svelte";
import CourseCodeCell from "./cells/CourseCodeCell.svelte";
import TitleCell from "./cells/TitleCell.svelte";
import InstructorCell from "./cells/InstructorCell.svelte";
import TimeCell from "./cells/TimeCell.svelte";
import LocationCell from "./cells/LocationCell.svelte";
import SeatsCell from "./cells/SeatsCell.svelte";

export const COLUMN_DEFS: ColumnDef<CourseResponse, unknown>[] = [
  {
    id: "crn",
    accessorKey: "crn",
    header: "CRN",
    enableSorting: false,
  },
  {
    id: "course_code",
    accessorFn: (row) => `${row.subject} ${row.courseNumber}`,
    header: "Course",
    enableSorting: true,
  },
  {
    id: "title",
    accessorKey: "title",
    header: "Title",
    enableSorting: true,
  },
  {
    id: "instructor",
    accessorFn: (row) => {
      const primary = getPrimaryInstructor(row.instructors);
      if (!primary) return "Staff";
      return abbreviateInstructor(primary.displayName);
    },
    header: "Instructor",
    enableSorting: true,
  },
  {
    id: "time",
    accessorFn: (row) => {
      if (row.meetingTimes.length === 0) return "";
      const mt = row.meetingTimes[0];
      return `${formatMeetingDays(mt)} ${formatTimeRange(mt.begin_time, mt.end_time)}`;
    },
    header: "Time",
    enableSorting: true,
  },
  {
    id: "location",
    accessorFn: (row) => formatLocationDisplay(row) ?? "",
    header: "Location",
    enableSorting: false,
  },
  {
    id: "seats",
    accessorFn: (row) => openSeats(row),
    header: "Seats",
    enableSorting: true,
  },
];

/** Column ID to Svelte cell component. Used by the row renderer. */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const CELL_COMPONENTS: Record<string, Component<any>> = {
  crn: CrnCell,
  course_code: CourseCodeCell,
  title: TitleCell,
  instructor: InstructorCell,
  time: TimeCell,
  location: LocationCell,
  seats: SeatsCell,
};
