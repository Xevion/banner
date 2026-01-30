import type {
  CandidateResponse,
  CodeDescription,
  CourseResponse,
  DbMeetingTime,
  InstructorDetail,
  InstructorDetailResponse,
  InstructorListItem,
  InstructorResponse,
  InstructorStats,
  LinkedRmpProfile,
  ListInstructorsResponse,
  RescoreResponse,
  SearchResponse as SearchResponseGenerated,
  ServiceInfo,
  ServiceStatus,
  StatusResponse,
  TopCandidateResponse,
  User,
} from "$lib/bindings";

const API_BASE_URL = "/api";

// Re-export generated types under their canonical names
export type {
  CandidateResponse,
  CodeDescription,
  CourseResponse,
  DbMeetingTime,
  InstructorDetail,
  InstructorDetailResponse,
  InstructorListItem,
  InstructorResponse,
  InstructorStats,
  LinkedRmpProfile,
  ListInstructorsResponse,
  RescoreResponse,
  ServiceInfo,
  ServiceStatus,
  StatusResponse,
  TopCandidateResponse,
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
  queuedAt: string;
  status: "processing" | "staleLock" | "exhausted" | "scheduled" | "pending";
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
  subject: string | null;
  courseNumber: string | null;
  crn: string | null;
  courseTitle: string | null;
}

export interface AuditLogResponse {
  entries: AuditLogEntry[];
}

export interface MetricEntry {
  id: number;
  courseId: number;
  timestamp: string;
  enrollment: number;
  waitCount: number;
  seatsAvailable: number;
}

export interface MetricsResponse {
  metrics: MetricEntry[];
  count: number;
  timestamp: string;
}

export interface MetricsParams {
  course_id?: number;
  term?: string;
  crn?: string;
  range?: "1h" | "6h" | "24h" | "7d" | "30d";
  limit?: number;
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

// Admin instructor query params (client-only, not generated)
export interface AdminInstructorListParams {
  status?: string;
  search?: string;
  page?: number;
  per_page?: number;
  sort?: string;
}

export class BannerApiClient {
  private baseUrl: string;
  private fetchFn: typeof fetch;

  constructor(baseUrl: string = API_BASE_URL, fetchFn: typeof fetch = fetch) {
    this.baseUrl = baseUrl;
    this.fetchFn = fetchFn;
  }

  private buildInit(options?: { method?: string; body?: unknown }): RequestInit | undefined {
    if (!options) return undefined;
    const init: RequestInit = {};
    if (options.method) {
      init.method = options.method;
    }
    if (options.body !== undefined) {
      init.headers = { "Content-Type": "application/json" };
      init.body = JSON.stringify(options.body);
    } else if (options.method) {
      init.headers = { "Content-Type": "application/json" };
    }
    return Object.keys(init).length > 0 ? init : undefined;
  }

  private async request<T>(
    endpoint: string,
    options?: { method?: string; body?: unknown }
  ): Promise<T> {
    const init = this.buildInit(options);
    const args: [string, RequestInit?] = [`${this.baseUrl}${endpoint}`];
    if (init) args.push(init);

    const response = await this.fetchFn(...args);

    if (!response.ok) {
      throw new Error(`API request failed: ${response.status} ${response.statusText}`);
    }

    return (await response.json()) as T;
  }

  private async requestVoid(
    endpoint: string,
    options?: { method?: string; body?: unknown }
  ): Promise<void> {
    const init = this.buildInit(options);
    const args: [string, RequestInit?] = [`${this.baseUrl}${endpoint}`];
    if (init) args.push(init);

    const response = await this.fetchFn(...args);

    if (!response.ok) {
      throw new Error(`API request failed: ${response.status} ${response.statusText}`);
    }
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
    return this.request<User>(`/admin/users/${discordId}/admin`, {
      method: "PUT",
      body: { is_admin: isAdmin },
    });
  }

  async getAdminScrapeJobs(): Promise<ScrapeJobsResponse> {
    return this.request<ScrapeJobsResponse>("/admin/scrape-jobs");
  }

  /**
   * Fetch the audit log with conditional request support.
   *
   * Returns `null` when the server responds 304 (data unchanged).
   * Stores and sends `Last-Modified` / `If-Modified-Since` automatically.
   */
  async getAdminAuditLog(): Promise<AuditLogResponse | null> {
    const headers: Record<string, string> = {};
    if (this._auditLastModified) {
      headers["If-Modified-Since"] = this._auditLastModified;
    }

    const response = await this.fetchFn(`${this.baseUrl}/admin/audit-log`, { headers });

    if (response.status === 304) {
      return null;
    }

    if (!response.ok) {
      throw new Error(`API request failed: ${response.status} ${response.statusText}`);
    }

    const lastMod = response.headers.get("Last-Modified");
    if (lastMod) {
      this._auditLastModified = lastMod;
    }

    return (await response.json()) as AuditLogResponse;
  }

  /** Stored `Last-Modified` value for audit log conditional requests. */
  private _auditLastModified: string | null = null;

  async getMetrics(params?: MetricsParams): Promise<MetricsResponse> {
    const query = new URLSearchParams();
    if (params?.course_id !== undefined) query.set("course_id", String(params.course_id));
    if (params?.term) query.set("term", params.term);
    if (params?.crn) query.set("crn", params.crn);
    if (params?.range) query.set("range", params.range);
    if (params?.limit !== undefined) query.set("limit", String(params.limit));
    const qs = query.toString();
    return this.request<MetricsResponse>(`/metrics${qs ? `?${qs}` : ""}`);
  }

  // Admin instructor endpoints

  async getAdminInstructors(params?: AdminInstructorListParams): Promise<ListInstructorsResponse> {
    const query = new URLSearchParams();
    if (params?.status) query.set("status", params.status);
    if (params?.search) query.set("search", params.search);
    if (params?.page !== undefined) query.set("page", String(params.page));
    if (params?.per_page !== undefined) query.set("per_page", String(params.per_page));
    if (params?.sort) query.set("sort", params.sort);
    const qs = query.toString();
    return this.request<ListInstructorsResponse>(`/admin/instructors${qs ? `?${qs}` : ""}`);
  }

  async getAdminInstructor(id: number): Promise<InstructorDetailResponse> {
    return this.request<InstructorDetailResponse>(`/admin/instructors/${id}`);
  }

  async matchInstructor(id: number, rmpLegacyId: number): Promise<InstructorDetailResponse> {
    return this.request<InstructorDetailResponse>(`/admin/instructors/${id}/match`, {
      method: "POST",
      body: { rmpLegacyId },
    });
  }

  async rejectCandidate(id: number, rmpLegacyId: number): Promise<void> {
    return this.requestVoid(`/admin/instructors/${id}/reject-candidate`, {
      method: "POST",
      body: { rmpLegacyId },
    });
  }

  async rejectAllCandidates(id: number): Promise<void> {
    return this.requestVoid(`/admin/instructors/${id}/reject-all`, {
      method: "POST",
    });
  }

  async unmatchInstructor(id: number, rmpLegacyId?: number): Promise<void> {
    return this.requestVoid(`/admin/instructors/${id}/unmatch`, {
      method: "POST",
      ...(rmpLegacyId !== undefined ? { body: { rmpLegacyId } } : {}),
    });
  }

  async rescoreInstructors(): Promise<RescoreResponse> {
    return this.request<RescoreResponse>("/admin/rmp/rescore", {
      method: "POST",
    });
  }
}

export const client = new BannerApiClient();
