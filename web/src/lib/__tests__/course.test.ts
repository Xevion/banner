import { describe, it, expect } from "vitest";
import { formatMeetingTimeSummary } from "$lib/course";
import type { CourseResponse, DbMeetingTime } from "$lib/api";

function makeMeetingTime(overrides: Partial<DbMeetingTime> = {}): DbMeetingTime {
  return {
    begin_time: null,
    end_time: null,
    start_date: "2025-01-13",
    end_date: "2025-05-08",
    monday: false,
    tuesday: false,
    wednesday: false,
    thursday: false,
    friday: false,
    saturday: false,
    sunday: false,
    building: null,
    building_description: null,
    room: null,
    campus: null,
    meeting_type: "CLAS",
    meeting_schedule_type: "LEC",
    ...overrides,
  };
}

function makeCourse(overrides: Partial<CourseResponse> = {}): CourseResponse {
  return {
    crn: "12345",
    subject: "CS",
    courseNumber: "1234",
    title: "Test Course",
    termCode: "202510",
    sequenceNumber: null,
    instructionalMethod: null,
    campus: null,
    enrollment: 10,
    maxEnrollment: 30,
    waitCount: 0,
    waitCapacity: 0,
    creditHours: 3,
    creditHourLow: null,
    creditHourHigh: null,
    crossList: null,
    crossListCapacity: null,
    crossListCount: null,
    linkIdentifier: null,
    isSectionLinked: null,
    partOfTerm: null,
    meetingTimes: [],
    attributes: [],
    instructors: [],
    ...overrides,
  };
}

describe("formatMeetingTimeSummary", () => {
  it("returns 'Async' for async online courses", () => {
    const course = makeCourse({
      meetingTimes: [makeMeetingTime({ building: "INT" })],
    });
    expect(formatMeetingTimeSummary(course)).toBe("Async");
  });

  it("returns 'TBA' for courses with no meeting times", () => {
    const course = makeCourse({ meetingTimes: [] });
    expect(formatMeetingTimeSummary(course)).toBe("TBA");
  });

  it("returns 'TBA' when days and times are all TBA", () => {
    const course = makeCourse({
      meetingTimes: [makeMeetingTime()],
    });
    expect(formatMeetingTimeSummary(course)).toBe("TBA");
  });

  it("returns formatted days and time for normal meeting", () => {
    const course = makeCourse({
      meetingTimes: [
        makeMeetingTime({
          monday: true,
          wednesday: true,
          friday: true,
          begin_time: "0900",
          end_time: "0950",
        }),
      ],
    });
    expect(formatMeetingTimeSummary(course)).toBe("MWF 9:00–9:50 AM");
  });

  it("returns formatted days with TBA time", () => {
    const course = makeCourse({
      meetingTimes: [
        makeMeetingTime({
          tuesday: true,
          thursday: true,
        }),
      ],
    });
    // Days are set but time is TBA — not both TBA, so it enters the final branch
    expect(formatMeetingTimeSummary(course)).toBe("TTh TBA");
  });
});
