import { describe, it, expect, vi, beforeEach } from "vitest";
import { BannerApiClient } from "./api";

// Mock fetch
global.fetch = vi.fn();

describe("BannerApiClient", () => {
  let apiClient: BannerApiClient;

  beforeEach(() => {
    apiClient = new BannerApiClient();
    vi.clearAllMocks();
  });

  it("should fetch health data", async () => {
    const mockHealth = {
      status: "healthy",
      timestamp: "2024-01-01T00:00:00Z",
    };

    vi.mocked(fetch).mockResolvedValueOnce({
      ok: true,
      json: () => Promise.resolve(mockHealth),
    } as Response);

    const result = await apiClient.getHealth();

    expect(fetch).toHaveBeenCalledWith("/api/health");
    expect(result).toEqual(mockHealth);
  });

  it("should fetch status data", async () => {
    const mockStatus = {
      status: "active",
      version: "0.3.4",
      commit: "abc1234",
      services: {
        web: { name: "web", status: "active" },
        database: { name: "database", status: "connected" },
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

    await expect(apiClient.getHealth()).rejects.toThrow(
      "API request failed: 500 Internal Server Error"
    );
  });
});
