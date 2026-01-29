import { describe, it, expect } from "vitest";
import {
  formatTime,
  formatMeetingDays,
  formatMeetingTime,
  abbreviateInstructor,
  formatCreditHours,
  getPrimaryInstructor,
  isMeetingTimeTBA,
  isTimeTBA,
  formatDate,
  formatMeetingDaysLong,
} from "$lib/course";
import type { DbMeetingTime, CourseResponse, InstructorResponse } from "$lib/api";

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
  it("returns TR for tue/thu", () => {
    expect(formatMeetingDays(makeMeetingTime({ tuesday: true, thursday: true }))).toBe("TR");
  });
  it("returns empty string when no days", () => {
    expect(formatMeetingDays(makeMeetingTime())).toBe("");
  });
  it("returns all days", () => {
    expect(
      formatMeetingDays(
        makeMeetingTime({
          monday: true,
          tuesday: true,
          wednesday: true,
          thursday: true,
          friday: true,
          saturday: true,
          sunday: true,
        })
      )
    ).toBe("MTWRFSU");
  });
});

describe("formatMeetingTime", () => {
  it("formats a standard meeting time", () => {
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
    ).toBe("MWF 9:00 AM–9:50 AM");
  });
  it("returns TBA when no days", () => {
    expect(formatMeetingTime(makeMeetingTime({ begin_time: "0900", end_time: "0950" }))).toBe(
      "TBA"
    );
  });
  it("returns days + TBA when no times", () => {
    expect(formatMeetingTime(makeMeetingTime({ monday: true }))).toBe("M TBA");
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
      { bannerId: "1", displayName: "A", email: null, isPrimary: false },
      { bannerId: "2", displayName: "B", email: null, isPrimary: true },
    ];
    expect(getPrimaryInstructor(instructors)?.displayName).toBe("B");
  });
  it("returns first instructor when no primary", () => {
    const instructors: InstructorResponse[] = [
      { bannerId: "1", displayName: "A", email: null, isPrimary: false },
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
