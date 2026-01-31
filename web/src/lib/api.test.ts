import { beforeEach, describe, expect, it, vi } from "vitest";
import { BannerApiClient } from "./api";

global.fetch = vi.fn();

describe("BannerApiClient", () => {
  let apiClient: BannerApiClient;

  beforeEach(() => {
    apiClient = new BannerApiClient();
    vi.clearAllMocks();
  });

  it("should fetch status data", async () => {
    const mockStatus = {
      status: "active" as const,
      version: "0.3.4",
      commit: "abc1234",
      services: {
        web: { name: "web", status: "active" as const },
        database: { name: "database", status: "connected" as const },
      },
    };

    vi.mocked(fetch).mockResolvedValueOnce({
      ok: true,
      json: () => Promise.resolve(mockStatus),
    } as Response);

    const result = await apiClient.getStatus();

    expect(fetch).toHaveBeenCalledWith("/api/status");
    expect(result).toEqual(mockStatus);
  });

  it("should handle API errors", async () => {
    vi.mocked(fetch).mockResolvedValueOnce({
      ok: false,
      status: 500,
      statusText: "Internal Server Error",
    } as Response);

    await expect(apiClient.getStatus()).rejects.toThrow(
      "API request failed: 500 Internal Server Error"
    );
  });

  it("should search courses with all params", async () => {
    const mockResponse = {
      courses: [],
      totalCount: 0,
    };

    vi.mocked(fetch).mockResolvedValueOnce({
      ok: true,
      json: () => Promise.resolve(mockResponse),
    } as Response);

    const result = await apiClient.searchCourses({
      term: "202420",
      subject: ["CS"],
      q: "data",
      openOnly: true,
      limit: 25,
      offset: 50,
    });

    expect(fetch).toHaveBeenCalledWith(
      "/api/courses/search?term=202420&subject=CS&q=data&open_only=true&limit=25&offset=50"
    );
    expect(result).toEqual(mockResponse);
  });

  it("should search courses with minimal params", async () => {
    const mockResponse = {
      courses: [],
      totalCount: 0,
    };

    vi.mocked(fetch).mockResolvedValueOnce({
      ok: true,
      json: () => Promise.resolve(mockResponse),
    } as Response);

    await apiClient.searchCourses({ term: "202420" });

    expect(fetch).toHaveBeenCalledWith("/api/courses/search?term=202420");
  });

  it("should fetch reference data", async () => {
    const mockRef = [
      { code: "F", description: "Face to Face" },
      { code: "OL", description: "Online" },
    ];

    vi.mocked(fetch).mockResolvedValueOnce({
      ok: true,
      json: () => Promise.resolve(mockRef),
    } as Response);

    const result = await apiClient.getReference("instructional_methods");

    expect(fetch).toHaveBeenCalledWith("/api/reference/instructional_methods");
    expect(result).toEqual(mockRef);
  });
});
