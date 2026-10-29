import { describe, expect, it } from "vitest";
import { dayPhrase, fmtBytes, relativeAge } from "./human";

describe("fmtBytes", () => {
  it("stays small until it matters", () => {
    expect(fmtBytes(120)).toBe("120 B");
    expect(fmtBytes(2048)).toBe("2 KB");
    expect(fmtBytes(1024 * 1024 * 5)).toBe("5.0 MB");
  });
});

describe("dayPhrase", () => {
  it("speaks like a person", () => {
    expect(dayPhrase(0)).toBe("today");
    expect(dayPhrase(1)).toBe("tomorrow");
    expect(dayPhrase(4)).toBe("in 4 days");
    expect(dayPhrase(-1)).toBe("1 day ago");
    expect(dayPhrase(-3)).toBe("3 days ago");
  });
});

describe("relativeAge", () => {
  it("uses the real store age, not a filename", () => {
    expect(relativeAge({ ageDays: 0 })).toBe("untouched since today");
    expect(relativeAge({ ageDays: 1 })).toBe("untouched for a day");
    expect(relativeAge({ ageDays: 45 })).toBe("untouched for 1 month");
    expect(relativeAge({ ageDays: 400 })).toBe("untouched for 1.1 years");
    expect(relativeAge({ ageDays: -4 })).toBe("untouched since today");
  });
});
