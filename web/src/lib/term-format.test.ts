import { describe, it, expect } from "vitest";
import { termToFriendly, termToBanner } from "./term-format";

describe("termToFriendly", () => {
  it("converts spring term correctly", () => {
    expect(termToFriendly("202610")).toBe("spring-26");
    expect(termToFriendly("202510")).toBe("spring-25");
  });

  it("converts summer term correctly", () => {
    expect(termToFriendly("202620")).toBe("summer-26");
    expect(termToFriendly("202520")).toBe("summer-25");
  });

  it("converts fall term correctly", () => {
    expect(termToFriendly("202630")).toBe("fall-26");
    expect(termToFriendly("202530")).toBe("fall-25");
  });

  it("returns null for invalid codes", () => {
    expect(termToFriendly("20261")).toBe(null);
    expect(termToFriendly("2026100")).toBe(null);
    expect(termToFriendly("202640")).toBe(null); // Invalid semester code
    expect(termToFriendly("")).toBe(null);
  });
});

describe("termToBanner", () => {
  it("converts spring term correctly", () => {
    expect(termToBanner("spring-26")).toBe("202610");
    expect(termToBanner("spring-25")).toBe("202510");
  });

  it("converts summer term correctly", () => {
    expect(termToBanner("summer-26")).toBe("202620");
    expect(termToBanner("summer-25")).toBe("202520");
  });

  it("converts fall term correctly", () => {
    expect(termToBanner("fall-26")).toBe("202630");
    expect(termToBanner("fall-25")).toBe("202530");
  });

  it("returns null for invalid formats", () => {
    expect(termToBanner("winter-26")).toBe(null);
    expect(termToBanner("spring26")).toBe(null);
    expect(termToBanner("spring-2026")).toBe(null);
    expect(termToBanner("26-spring")).toBe(null);
    expect(termToBanner("")).toBe(null);
  });
});

describe("round-trip conversion", () => {
  it("converts back and forth correctly", () => {
    const bannerCodes = ["202610", "202620", "202630", "202510"];

    for (const code of bannerCodes) {
      const friendly = termToFriendly(code);
      expect(friendly).not.toBeNull();
      const backToBanner = termToBanner(friendly!);
      expect(backToBanner).toBe(code);
    }
  });
});
