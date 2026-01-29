import type {
  CodeDescription,
  CourseResponse,
  DbMeetingTime,
  InstructorResponse,
  SearchResponse as SearchResponseGenerated,
  ServiceInfo,
  ServiceStatus,
  StatusResponse,
  User,
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

// Client-side only — not generated from Rust
export type SortColumn = "course_code" | "title" | "instructor" | "time" | "seats";
export type SortDirection = "asc" | "desc";

export interface AdminStatus {
  userCount: number;
  sessionCount: number;
  courseCount: number;
  scrapeJobCount: number;
  services: { name: string; status: string }[];
}

export interface ScrapeJob {
  id: number;
  targetType: string;
  targetPayload: unknown;
  priority: string;
  executeAt: string;
  createdAt: string;
  lockedAt: string | null;
  retryCount: number;
  maxRetries: number;
}

export interface ScrapeJobsResponse {
  jobs: ScrapeJob[];
}

export interface AuditLogEntry {
  id: number;
  courseId: number;
  timestamp: string;
  fieldChanged: string;
  oldValue: string;
  newValue: string;
}

export interface AuditLogResponse {
  entries: AuditLogEntry[];
}

export interface SearchParams {
  term: string;
  subjects?: string[];
  q?: string;
  open_only?: boolean;
  limit?: number;
  offset?: number;
  sort_by?: SortColumn;
  sort_dir?: SortDirection;
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

  async getStatus(): Promise<StatusResponse> {
    return this.request<StatusResponse>("/status");
  }

  async searchCourses(params: SearchParams): Promise<SearchResponse> {
    const query = new URLSearchParams();
    query.set("term", params.term);
    if (params.subjects) {
      for (const s of params.subjects) {
        query.append("subject", s);
      }
    }
    if (params.q) query.set("q", params.q);
    if (params.open_only) query.set("open_only", "true");
    if (params.limit !== undefined) query.set("limit", String(params.limit));
    if (params.offset !== undefined) query.set("offset", String(params.offset));
    if (params.sort_by) query.set("sort_by", params.sort_by);
    if (params.sort_dir) query.set("sort_dir", params.sort_dir);
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

  // Admin endpoints
  async getAdminStatus(): Promise<AdminStatus> {
    return this.request<AdminStatus>("/admin/status");
  }

  async getAdminUsers(): Promise<User[]> {
    return this.request<User[]>("/admin/users");
  }

  async setUserAdmin(discordId: string, isAdmin: boolean): Promise<User> {
    const response = await this.fetchFn(`${this.baseUrl}/admin/users/${discordId}/admin`, {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ is_admin: isAdmin }),
    });
    if (!response.ok) throw new Error(`API request failed: ${response.status}`);
    return (await response.json()) as User;
  }

  async getAdminScrapeJobs(): Promise<ScrapeJobsResponse> {
    return this.request<ScrapeJobsResponse>("/admin/scrape-jobs");
  }

  async getAdminAuditLog(): Promise<AuditLogResponse> {
    return this.request<AuditLogResponse>("/admin/audit-log");
  }
}

export const client = new BannerApiClient();
