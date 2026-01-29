import type {
  CodeDescription,
  CourseResponse,
  DbMeetingTime,
  InstructorResponse,
  SearchResponse as SearchResponseGenerated,
  ServiceInfo,
  ServiceStatus,
  StatusResponse,
} from "$lib/bindings";

const API_BASE_URL = "/api";

// Re-export generated types under their canonical names
export type {
  CodeDescription,
  CourseResponse,
  DbMeetingTime,
  InstructorResponse,
  ServiceInfo,
  ServiceStatus,
  StatusResponse,
};

// Semantic aliases — these all share the CodeDescription shape
export type Term = CodeDescription;
export type Subject = CodeDescription;
export type ReferenceEntry = CodeDescription;

// SearchResponse re-exported (aliased to strip the "Generated" suffix)
export type SearchResponse = SearchResponseGenerated;

// Health/metrics endpoints return ad-hoc JSON — keep manual types
export interface HealthResponse {
  status: string;
  timestamp: string;
}

export interface MetricsResponse {
  banner_api: {
    status: string;
  };
  timestamp: string;
}

// Client-side only — not generated from Rust
export interface SearchParams {
  term: string;
  subject?: string;
  q?: string;
  open_only?: boolean;
  limit?: number;
  offset?: number;
}

export class BannerApiClient {
  private baseUrl: string;
  private fetchFn: typeof fetch;

  constructor(baseUrl: string = API_BASE_URL, fetchFn: typeof fetch = fetch) {
    this.baseUrl = baseUrl;
    this.fetchFn = fetchFn;
  }

  private async request<T>(endpoint: string): Promise<T> {
    const response = await this.fetchFn(`${this.baseUrl}${endpoint}`);

    if (!response.ok) {
      throw new Error(`API request failed: ${response.status} ${response.statusText}`);
    }

    return (await response.json()) as T;
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

  async searchCourses(params: SearchParams): Promise<SearchResponse> {
    const query = new URLSearchParams();
    query.set("term", params.term);
    if (params.subject) query.set("subject", params.subject);
    if (params.q) query.set("q", params.q);
    if (params.open_only) query.set("open_only", "true");
    if (params.limit !== undefined) query.set("limit", String(params.limit));
    if (params.offset !== undefined) query.set("offset", String(params.offset));
    return this.request<SearchResponse>(`/courses/search?${query.toString()}`);
  }

  async getTerms(): Promise<Term[]> {
    return this.request<Term[]>("/terms");
  }

  async getSubjects(termCode: string): Promise<Subject[]> {
    return this.request<Subject[]>(`/subjects?term=${encodeURIComponent(termCode)}`);
  }

  async getReference(category: string): Promise<ReferenceEntry[]> {
    return this.request<ReferenceEntry[]>(`/reference/${encodeURIComponent(category)}`);
  }
}

export const client = new BannerApiClient();
