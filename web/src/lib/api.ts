// API client for Banner backend
const API_BASE_URL = "/api";

export interface HealthResponse {
  status: string;
  timestamp: string;
}

export interface StatusResponse {
  status: string;
  bot: {
    status: string;
    uptime: string;
  };
  cache: {
    status: string;
    courses: string;
    subjects: string;
  };
  banner_api: {
    status: string;
  };
  timestamp: string;
}

export interface MetricsResponse {
  banner_api: {
    status: string;
  };
  timestamp: string;
}

export class BannerApiClient {
  private baseUrl: string;

  constructor(baseUrl: string = API_BASE_URL) {
    this.baseUrl = baseUrl;
  }

  private async request<T>(endpoint: string): Promise<T> {
    const response = await fetch(`${this.baseUrl}${endpoint}`);

    if (!response.ok) {
      throw new Error(
        `API request failed: ${response.status} ${response.statusText}`
      );
    }

    return response.json();
  }

  async getHealth(): Promise<HealthResponse> {
    return this.request<HealthResponse>("/health");
  }

  async getStatus(): Promise<StatusResponse> {
    return this.request<StatusResponse>("/status");
  }

  async getMetrics(): Promise<MetricsResponse> {
    return this.request<MetricsResponse>("/metrics");
  }
}

// Export a default instance
export const apiClient = new BannerApiClient();
