import { describe, expect, it } from "vitest";
import { formatDiffPath, jsonDiff, tryParseJson } from "./diff";

describe("jsonDiff", () => {
  describe("scalars", () => {
    it("returns empty array for identical primitives", () => {
      expect(jsonDiff(42, 42)).toEqual([]);
      expect(jsonDiff("hello", "hello")).toEqual([]);
      expect(jsonDiff(true, true)).toEqual([]);
      expect(jsonDiff(null, null)).toEqual([]);
    });

    it("returns single entry for different primitives", () => {
      expect(jsonDiff("Open", "Closed")).toEqual([{ path: "", oldVal: "Open", newVal: "Closed" }]);
      expect(jsonDiff(25, 30)).toEqual([{ path: "", oldVal: 25, newVal: 30 }]);
      expect(jsonDiff(true, false)).toEqual([{ path: "", oldVal: true, newVal: false }]);
    });

    it("returns entry when types differ", () => {
      expect(jsonDiff(1, "1")).toEqual([{ path: "", oldVal: 1, newVal: "1" }]);
      expect(jsonDiff(null, 0)).toEqual([{ path: "", oldVal: null, newVal: 0 }]);
    });
  });

  describe("objects", () => {
    it("detects changed key", () => {
      expect(jsonDiff({ a: 1 }, { a: 2 })).toEqual([{ path: ".a", oldVal: 1, newVal: 2 }]);
    });

    it("detects added key", () => {
      expect(jsonDiff({}, { a: 1 })).toEqual([{ path: ".a", oldVal: undefined, newVal: 1 }]);
    });

    it("detects removed key", () => {
      expect(jsonDiff({ a: 1 }, {})).toEqual([{ path: ".a", oldVal: 1, newVal: undefined }]);
    });

    it("handles deeply nested changes", () => {
      const oldVal = { a: { b: { c: 1 } } };
      const newVal = { a: { b: { c: 2 } } };
      expect(jsonDiff(oldVal, newVal)).toEqual([{ path: ".a.b.c", oldVal: 1, newVal: 2 }]);
    });

    it("returns empty for identical objects", () => {
      expect(jsonDiff({ a: 1, b: "x" }, { a: 1, b: "x" })).toEqual([]);
    });
  });

  describe("arrays", () => {
    it("detects changed element", () => {
      expect(jsonDiff([1, 2, 3], [1, 99, 3])).toEqual([{ path: "[1]", oldVal: 2, newVal: 99 }]);
    });

    it("detects added element (new array longer)", () => {
      expect(jsonDiff([1], [1, 2])).toEqual([{ path: "[1]", oldVal: undefined, newVal: 2 }]);
    });

    it("detects removed element (new array shorter)", () => {
      expect(jsonDiff([1, 2], [1])).toEqual([{ path: "[1]", oldVal: 2, newVal: undefined }]);
    });

    it("returns empty for identical arrays", () => {
      expect(jsonDiff([1, 2, 3], [1, 2, 3])).toEqual([]);
    });
  });

  describe("mixed nesting", () => {
    it("handles array of objects", () => {
      const oldVal = [{ name: "Alice" }, { name: "Bob" }];
      const newVal = [{ name: "Alice" }, { name: "Charlie" }];
      expect(jsonDiff(oldVal, newVal)).toEqual([
        { path: "[1].name", oldVal: "Bob", newVal: "Charlie" },
      ]);
    });

    it("handles object with nested arrays", () => {
      const oldVal = { meetingTimes: [{ beginTime: "0900" }] };
      const newVal = { meetingTimes: [{ beginTime: "1000" }] };
      expect(jsonDiff(oldVal, newVal)).toEqual([
        { path: ".meetingTimes[0].beginTime", oldVal: "0900", newVal: "1000" },
      ]);
    });

    it("handles type change from object to array", () => {
      expect(jsonDiff({ a: 1 }, [1])).toEqual([{ path: "", oldVal: { a: 1 }, newVal: [1] }]);
    });
  });
});

describe("tryParseJson", () => {
  it("parses valid JSON object", () => {
    expect(tryParseJson('{"a":1}')).toEqual({ a: 1 });
  });

  it("parses valid JSON array", () => {
    expect(tryParseJson("[1,2,3]")).toEqual([1, 2, 3]);
  });

  it("parses plain string numbers", () => {
    expect(tryParseJson("42")).toBe(42);
    expect(tryParseJson("3.14")).toBe(3.14);
  });

  it("returns null for invalid JSON", () => {
    expect(tryParseJson("not json")).toBeNull();
    expect(tryParseJson("{broken")).toBeNull();
  });

  it("parses boolean and null literals", () => {
    expect(tryParseJson("true")).toBe(true);
    expect(tryParseJson("null")).toBeNull();
  });
});

describe("formatDiffPath", () => {
  it("strips leading dot", () => {
    expect(formatDiffPath(".a.b.c")).toBe("a.b.c");
  });

  it("returns (root) for empty path", () => {
    expect(formatDiffPath("")).toBe("(root)");
  });

  it("preserves bracket notation", () => {
    expect(formatDiffPath("[0].name")).toBe("[0].name");
  });

  it("handles mixed paths", () => {
    expect(formatDiffPath(".meetingTimes[0].beginTime")).toBe("meetingTimes[0].beginTime");
  });
});
