import type { CourseResponse, DbMeetingTime, InstructorResponse } from "$lib/api";
import {
  abbreviateInstructor,
  formatCreditHours,
  formatDate,
  formatDateShort,
  formatLocationDisplay,
  formatMeetingDays,
  formatMeetingDaysLong,
  formatMeetingDaysVerbose,
  formatMeetingTime,
  formatMeetingTimeTooltip,
  formatMeetingTimesTooltip,
  formatTime,
  formatTimeRange,
  getPrimaryInstructor,
  isAsyncOnline,
  isMeetingTimeTBA,
  isTimeTBA,
} from "$lib/course";
import { describe, expect, it } from "vitest";

function makeMeetingTime(overrides: Partial<DbMeetingTime> = {}): DbMeetingTime {
  return {
    begin_time: null,
    end_time: null,
    start_date: "2024-08-26",
    end_date: "2024-12-12",
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

describe("formatTime", () => {
  it("converts 0900 to 9:00 AM", () => expect(formatTime("0900")).toBe("9:00 AM"));
  it("converts 1330 to 1:30 PM", () => expect(formatTime("1330")).toBe("1:30 PM"));
  it("converts 0000 to 12:00 AM", () => expect(formatTime("0000")).toBe("12:00 AM"));
  it("converts 1200 to 12:00 PM", () => expect(formatTime("1200")).toBe("12:00 PM"));
  it("converts 2359 to 11:59 PM", () => expect(formatTime("2359")).toBe("11:59 PM"));
  it("returns TBA for null", () => expect(formatTime(null)).toBe("TBA"));
  it("returns TBA for empty string", () => expect(formatTime("")).toBe("TBA"));
  it("returns TBA for short string", () => expect(formatTime("09")).toBe("TBA"));
});

describe("formatMeetingDays", () => {
  it("returns MWF for mon/wed/fri", () => {
    expect(
      formatMeetingDays(makeMeetingTime({ monday: true, wednesday: true, friday: true }))
    ).toBe("MWF");
  });
  it("returns TTh for tue/thu", () => {
    expect(formatMeetingDays(makeMeetingTime({ tuesday: true, thursday: true }))).toBe("TTh");
  });
  it("returns MW for mon/wed", () => {
    expect(formatMeetingDays(makeMeetingTime({ monday: true, wednesday: true }))).toBe("MW");
  });
  it("returns MTWThF for all weekdays", () => {
    expect(
      formatMeetingDays(
        makeMeetingTime({
          monday: true,
          tuesday: true,
          wednesday: true,
          thursday: true,
          friday: true,
        })
      )
    ).toBe("MTWThF");
  });
  it("returns partial abbreviation for single day", () => {
    expect(formatMeetingDays(makeMeetingTime({ monday: true }))).toBe("Mon");
    expect(formatMeetingDays(makeMeetingTime({ thursday: true }))).toBe("Thu");
    expect(formatMeetingDays(makeMeetingTime({ saturday: true }))).toBe("Sat");
  });
  it("concatenates codes for other multi-day combos", () => {
    expect(formatMeetingDays(makeMeetingTime({ monday: true, friday: true }))).toBe("MF");
    expect(formatMeetingDays(makeMeetingTime({ tuesday: true, saturday: true }))).toBe("TSa");
    expect(
      formatMeetingDays(makeMeetingTime({ wednesday: true, friday: true, sunday: true }))
    ).toBe("WFSu");
    expect(
      formatMeetingDays(
        makeMeetingTime({ monday: true, tuesday: true, wednesday: true, thursday: true })
      )
    ).toBe("MTWTh");
  });
  it("returns empty string when no days", () => {
    expect(formatMeetingDays(makeMeetingTime())).toBe("");
  });
});

describe("formatTimeRange", () => {
  it("elides AM when both times are AM", () => {
    expect(formatTimeRange("0900", "0950")).toBe("9:00–9:50 AM");
  });
  it("elides PM when both times are PM", () => {
    expect(formatTimeRange("1315", "1430")).toBe("1:15–2:30 PM");
  });
  it("keeps both markers when crossing noon", () => {
    expect(formatTimeRange("1130", "1220")).toBe("11:30 AM–12:20 PM");
  });
  it("returns TBA for null begin", () => {
    expect(formatTimeRange(null, "0950")).toBe("TBA");
  });
  it("returns TBA for null end", () => {
    expect(formatTimeRange("0900", null)).toBe("TBA");
  });
  it("handles midnight and noon", () => {
    expect(formatTimeRange("0000", "0050")).toBe("12:00–12:50 AM");
    expect(formatTimeRange("1200", "1250")).toBe("12:00–12:50 PM");
  });
});

describe("formatMeetingTime", () => {
  it("formats a standard meeting time with elided AM/PM", () => {
    expect(
      formatMeetingTime(
        makeMeetingTime({
          monday: true,
          wednesday: true,
          friday: true,
          begin_time: "0900",
          end_time: "0950",
        })
      )
    ).toBe("MWF 9:00–9:50 AM");
  });
  it("keeps both markers when crossing noon", () => {
    expect(
      formatMeetingTime(
        makeMeetingTime({
          tuesday: true,
          thursday: true,
          begin_time: "1130",
          end_time: "1220",
        })
      )
    ).toBe("TTh 11:30 AM–12:20 PM");
  });
  it("returns TBA when no days", () => {
    expect(formatMeetingTime(makeMeetingTime({ begin_time: "0900", end_time: "0950" }))).toBe(
      "TBA"
    );
  });
  it("returns days + TBA when no times", () => {
    expect(formatMeetingTime(makeMeetingTime({ monday: true }))).toBe("Mon TBA");
  });
});

describe("abbreviateInstructor", () => {
  it("returns short names unabbreviated", () =>
    expect(abbreviateInstructor("Li, Bo")).toBe("Li, Bo"));
  it("returns names within budget unabbreviated", () =>
    expect(abbreviateInstructor("Heaps, John")).toBe("Heaps, John"));
  it("handles no comma", () => expect(abbreviateInstructor("Staff")).toBe("Staff"));

  // Progressive abbreviation with multiple given names
  it("abbreviates trailing given names first", () =>
    expect(abbreviateInstructor("Ramirez, Maria Elena")).toBe("Ramirez, Maria E."));
  it("abbreviates all given names when needed", () =>
    expect(abbreviateInstructor("Ramirez, Maria Elena", 16)).toBe("Ramirez, M. E."));
  it("falls back to first initial only", () =>
    expect(abbreviateInstructor("Ramirez, Maria Elena", 12)).toBe("Ramirez, M."));

  // Single given name that exceeds budget
  it("abbreviates single given name when over budget", () =>
    expect(abbreviateInstructor("Bartholomew, Christopher", 18)).toBe("Bartholomew, C."));

  // Respects custom maxLen
  it("keeps full name when within custom budget", () =>
    expect(abbreviateInstructor("Ramirez, Maria Elena", 30)).toBe("Ramirez, Maria Elena"));
  it("always abbreviates when budget is tiny", () =>
    expect(abbreviateInstructor("Heaps, John", 5)).toBe("Heaps, J."));
});

describe("getPrimaryInstructor", () => {
  it("returns primary instructor", () => {
    const instructors: InstructorResponse[] = [
      {
        instructorId: 1,
        bannerId: "1",
        displayName: "A",
        email: "a@utsa.edu",
        isPrimary: false,
        rmpRating: null,
        rmpNumRatings: null,
        rmpLegacyId: null,
      },
      {
        instructorId: 2,
        bannerId: "2",
        displayName: "B",
        email: "b@utsa.edu",
        isPrimary: true,
        rmpRating: null,
        rmpNumRatings: null,
        rmpLegacyId: null,
      },
    ];
    expect(getPrimaryInstructor(instructors)?.displayName).toBe("B");
  });
  it("returns first instructor when no primary", () => {
    const instructors: InstructorResponse[] = [
      {
        instructorId: 3,
        bannerId: "1",
        displayName: "A",
        email: "a@utsa.edu",
        isPrimary: false,
        rmpRating: null,
        rmpNumRatings: null,
        rmpLegacyId: null,
      },
    ];
    expect(getPrimaryInstructor(instructors)?.displayName).toBe("A");
  });
  it("returns undefined for empty array", () => {
    expect(getPrimaryInstructor([])).toBeUndefined();
  });
});

describe("formatCreditHours", () => {
  it("returns creditHours when set", () => {
    expect(
      formatCreditHours({
        creditHours: 3,
        creditHourLow: null,
        creditHourHigh: null,
      } as CourseResponse)
    ).toBe("3");
  });
  it("returns range when variable", () => {
    expect(
      formatCreditHours({
        creditHours: null,
        creditHourLow: 1,
        creditHourHigh: 3,
      } as CourseResponse)
    ).toBe("1–3");
  });
  it("returns dash when no credit info", () => {
    expect(
      formatCreditHours({
        creditHours: null,
        creditHourLow: null,
        creditHourHigh: null,
      } as CourseResponse)
    ).toBe("—");
  });
});

describe("isMeetingTimeTBA", () => {
  it("returns true when no days set", () => {
    expect(isMeetingTimeTBA(makeMeetingTime())).toBe(true);
  });
  it("returns false when any day is set", () => {
    expect(isMeetingTimeTBA(makeMeetingTime({ monday: true }))).toBe(false);
  });
  it("returns false when multiple days set", () => {
    expect(isMeetingTimeTBA(makeMeetingTime({ tuesday: true, thursday: true }))).toBe(false);
  });
});

describe("isTimeTBA", () => {
  it("returns true when begin_time is null", () => {
    expect(isTimeTBA(makeMeetingTime())).toBe(true);
  });
  it("returns true when begin_time is empty", () => {
    expect(isTimeTBA(makeMeetingTime({ begin_time: "" }))).toBe(true);
  });
  it("returns true when begin_time is short", () => {
    expect(isTimeTBA(makeMeetingTime({ begin_time: "09" }))).toBe(true);
  });
  it("returns false when begin_time is valid", () => {
    expect(isTimeTBA(makeMeetingTime({ begin_time: "0900" }))).toBe(false);
  });
});

describe("formatDate", () => {
  it("formats standard date", () => {
    expect(formatDate("2024-08-26")).toBe("August 26, 2024");
  });
  it("formats December date", () => {
    expect(formatDate("2024-12-12")).toBe("December 12, 2024");
  });
  it("formats January 1st", () => {
    expect(formatDate("2026-01-01")).toBe("January 1, 2026");
  });
  it("formats MM/DD/YYYY date", () => {
    expect(formatDate("01/20/2026")).toBe("January 20, 2026");
  });
  it("formats MM/DD/YYYY with May", () => {
    expect(formatDate("05/13/2026")).toBe("May 13, 2026");
  });
  it("returns original string for invalid input", () => {
    expect(formatDate("bad-date")).toBe("bad-date");
  });
});

describe("formatMeetingDaysLong", () => {
  it("returns full plural for single day", () => {
    expect(formatMeetingDaysLong(makeMeetingTime({ thursday: true }))).toBe("Thursdays");
  });
  it("returns full plural for Monday only", () => {
    expect(formatMeetingDaysLong(makeMeetingTime({ monday: true }))).toBe("Mondays");
  });
  it("returns semi-abbreviated for multiple days", () => {
    expect(
      formatMeetingDaysLong(makeMeetingTime({ monday: true, wednesday: true, friday: true }))
    ).toBe("Mon, Wed, Fri");
  });
  it("returns semi-abbreviated for TR", () => {
    expect(formatMeetingDaysLong(makeMeetingTime({ tuesday: true, thursday: true }))).toBe(
      "Tue, Thur"
    );
  });
  it("returns empty string when no days", () => {
    expect(formatMeetingDaysLong(makeMeetingTime())).toBe("");
  });
});

describe("formatDateShort", () => {
  it("formats YYYY-MM-DD to short", () => {
    expect(formatDateShort("2024-08-26")).toBe("Aug 26, 2024");
  });
  it("formats MM/DD/YYYY to short", () => {
    expect(formatDateShort("12/12/2024")).toBe("Dec 12, 2024");
  });
  it("returns original for invalid", () => {
    expect(formatDateShort("bad")).toBe("bad");
  });
});

describe("formatMeetingDaysVerbose", () => {
  it("returns plural for single day", () => {
    expect(formatMeetingDaysVerbose(makeMeetingTime({ thursday: true }))).toBe("Thursdays");
  });
  it("joins two days with ampersand", () => {
    expect(formatMeetingDaysVerbose(makeMeetingTime({ tuesday: true, thursday: true }))).toBe(
      "Tuesdays & Thursdays"
    );
  });
  it("uses Oxford-style ampersand for 3+ days", () => {
    expect(
      formatMeetingDaysVerbose(makeMeetingTime({ monday: true, wednesday: true, friday: true }))
    ).toBe("Mondays, Wednesdays & Fridays");
  });
  it("returns empty string when no days", () => {
    expect(formatMeetingDaysVerbose(makeMeetingTime())).toBe("");
  });
});

describe("formatMeetingTimeTooltip", () => {
  it("formats full tooltip with location and dates", () => {
    const mt = makeMeetingTime({
      tuesday: true,
      thursday: true,
      begin_time: "1615",
      end_time: "1730",
      building_description: "Main Hall",
      room: "2.206",
    });
    expect(formatMeetingTimeTooltip(mt)).toBe(
      "Tuesdays & Thursdays, 4:15–5:30 PM\nMain Hall 2.206, Aug 26, 2024 – Dec 12, 2024"
    );
  });
  it("handles TBA days and times", () => {
    expect(formatMeetingTimeTooltip(makeMeetingTime())).toBe("TBA\nAug 26, 2024 – Dec 12, 2024");
  });
  it("handles days with TBA times", () => {
    expect(formatMeetingTimeTooltip(makeMeetingTime({ monday: true }))).toBe(
      "Mondays, TBA\nAug 26, 2024 – Dec 12, 2024"
    );
  });
});

describe("formatMeetingTimesTooltip", () => {
  it("returns TBA for empty array", () => {
    expect(formatMeetingTimesTooltip([])).toBe("TBA");
  });
  it("joins multiple meetings with blank line", () => {
    const mts = [
      makeMeetingTime({
        monday: true,
        wednesday: true,
        friday: true,
        begin_time: "0900",
        end_time: "0950",
      }),
      makeMeetingTime({
        thursday: true,
        begin_time: "1300",
        end_time: "1400",
        building_description: "Lab",
        room: "101",
      }),
    ];
    const result = formatMeetingTimesTooltip(mts);
    expect(result).toContain("Mondays, Wednesdays & Fridays, 9:00–9:50 AM");
    expect(result).toContain("Thursdays, 1:00–2:00 PM\nLab 101");
    expect(result).toContain("\n\n");
  });
});

describe("isAsyncOnline", () => {
  it("returns true for INT building with no times", () => {
    const course = {
      meetingTimes: [
        makeMeetingTime({
          building: "INT",
          building_description: "Internet Class",
          begin_time: null,
          end_time: null,
        }),
      ],
    } as CourseResponse;
    expect(isAsyncOnline(course)).toBe(true);
  });
  it("returns false for INT building with meeting times", () => {
    const course = {
      meetingTimes: [
        makeMeetingTime({
          building: "INT",
          building_description: "Internet Class",
          tuesday: true,
          thursday: true,
          begin_time: "1000",
          end_time: "1115",
        }),
      ],
    } as CourseResponse;
    expect(isAsyncOnline(course)).toBe(false);
  });
  it("returns false for non-INT building", () => {
    const course = {
      meetingTimes: [
        makeMeetingTime({
          building: "MH",
          building_description: "Main Hall",
          begin_time: null,
          end_time: null,
        }),
      ],
    } as CourseResponse;
    expect(isAsyncOnline(course)).toBe(false);
  });
  it("returns false for empty meeting times", () => {
    const course = { meetingTimes: [] } as unknown as CourseResponse;
    expect(isAsyncOnline(course)).toBe(false);
  });
});

describe("formatLocationDisplay", () => {
  it("returns 'Online' for INT building", () => {
    const course = {
      meetingTimes: [
        makeMeetingTime({
          building: "INT",
          building_description: "Internet Class",
        }),
      ],
      campus: "9",
    } as CourseResponse;
    expect(formatLocationDisplay(course)).toBe("Online");
  });
  it("returns building and room for physical location", () => {
    const course = {
      meetingTimes: [
        makeMeetingTime({
          building: "MH",
          building_description: "Main Hall",
          room: "2.206",
        }),
      ],
      campus: "11",
    } as CourseResponse;
    expect(formatLocationDisplay(course)).toBe("MH 2.206");
  });
  it("returns building only when no room", () => {
    const course = {
      meetingTimes: [
        makeMeetingTime({
          building: "MH",
          building_description: "Main Hall",
          room: null,
        }),
      ],
      campus: "11",
    } as CourseResponse;
    expect(formatLocationDisplay(course)).toBe("MH");
  });
});
