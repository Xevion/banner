import { describe, it, expect } from "vitest";
import {
  formatTime,
  formatMeetingDays,
  formatMeetingTime,
  abbreviateInstructor,
  formatCreditHours,
  getPrimaryInstructor,
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
    expect(formatMeetingDays(makeMeetingTime({ monday: true, wednesday: true, friday: true }))).toBe(
      "MWF"
    );
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
        makeMeetingTime({ monday: true, wednesday: true, friday: true, begin_time: "0900", end_time: "0950" })
      )
    ).toBe("MWF 9:00 AM–9:50 AM");
  });
  it("returns TBA when no days", () => {
    expect(formatMeetingTime(makeMeetingTime({ begin_time: "0900", end_time: "0950" }))).toBe("TBA");
  });
  it("returns days + TBA when no times", () => {
    expect(formatMeetingTime(makeMeetingTime({ monday: true }))).toBe("M TBA");
  });
});

describe("abbreviateInstructor", () => {
  it("abbreviates standard name", () => expect(abbreviateInstructor("Heaps, John")).toBe("Heaps, J."));
  it("handles no comma", () => expect(abbreviateInstructor("Staff")).toBe("Staff"));
  it("handles multiple first names", () =>
    expect(abbreviateInstructor("Smith, Mary Jane")).toBe("Smith, M."));
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
      formatCreditHours({ creditHours: 3, creditHourLow: null, creditHourHigh: null } as CourseResponse)
    ).toBe("3");
  });
  it("returns range when variable", () => {
    expect(
      formatCreditHours({ creditHours: null, creditHourLow: 1, creditHourHigh: 3 } as CourseResponse)
    ).toBe("1–3");
  });
  it("returns dash when no credit info", () => {
    expect(
      formatCreditHours({ creditHours: null, creditHourLow: null, creditHourHigh: null } as CourseResponse)
    ).toBe("—");
  });
});
