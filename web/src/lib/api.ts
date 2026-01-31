import { authStore } from "$lib/auth.svelte";
import type {
  AdminStatusResponse,
  ApiError,
  AuditLogEntry,
  AuditLogResponse,
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
  MetricEntry,
  MetricsParams as MetricsParamsGenerated,
  MetricsResponse,
  RescoreResponse,
  ScrapeJobDto,
  ScrapeJobEvent,
  ScrapeJobsResponse,
  ScraperStatsResponse,
  SearchParams as SearchParamsGenerated,
  SearchResponse as SearchResponseGenerated,
  ServiceInfo,
  ServiceStatus,
  SortColumn,
  SortDirection,
  StatusResponse,
  SubjectDetailResponse,
  SubjectResultEntry,
  SubjectSummary,
  SubjectsResponse,
  TermResponse,
  TimeRange,
  TimelineRequest,
  TimelineResponse,
  TimelineSlot,
  TimeseriesPoint,
  TimeseriesResponse,
  TopCandidateResponse,
  User,
} from "$lib/bindings";

const API_BASE_URL = "/api";

// Re-export generated types under their canonical names
export type {
  AdminStatusResponse,
  ApiError,
  AuditLogEntry,
  AuditLogResponse,
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
  MetricEntry,
  MetricsResponse,
  RescoreResponse,
  ScrapeJobDto,
  ScrapeJobEvent,
  ScrapeJobsResponse,
  ScraperStatsResponse,
  ServiceInfo,
  ServiceStatus,
  SortColumn,
  SortDirection,
  StatusResponse,
  SubjectDetailResponse,
  SubjectResultEntry,
  SubjectSummary,
  SubjectsResponse,
  TermResponse,
  TimelineRequest,
  TimelineResponse,
  TimelineSlot,
  TimeRange,
  TimeseriesPoint,
  TimeseriesResponse,
  TopCandidateResponse,
};

// Semantic aliases
export type Term = TermResponse;
export type Subject = CodeDescription;
export type ReferenceEntry = CodeDescription;

// Re-export with simplified names
export type SearchResponse = SearchResponseGenerated;
export type SearchParams = SearchParamsGenerated;
export type MetricsParams = MetricsParamsGenerated;

export type ScraperPeriod = "1h" | "6h" | "24h" | "7d" | "30d";

// Admin instructor query params (client-only, not generated)
export interface AdminInstructorListParams {
  status?: string;
  search?: string;
  page?: number;
  per_page?: number;
  sort?: string;
}

/**
 * API error class that wraps the structured ApiError response from the backend.
 */
export class ApiErrorClass extends Error {
  public readonly code: string;
  public readonly details: unknown | null;

  constructor(apiError: ApiError) {
    super(apiError.message);
    this.name = "ApiError";
    this.code = apiError.code;
    this.details = apiError.details;
  }

  isNotFound(): boolean {
    return this.code === "NOT_FOUND";
  }

  isBadRequest(): boolean {
    return (
      this.code === "BAD_REQUEST" || this.code === "INVALID_TERM" || this.code === "INVALID_RANGE"
    );
  }

  isInternalError(): boolean {
    return this.code === "INTERNAL_ERROR";
  }
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

    if (response.status === 401) {
      authStore.handleUnauthorized();
    }

    if (!response.ok) {
      let apiError: ApiError;
      try {
        apiError = (await response.json()) as ApiError;
      } catch {
        apiError = {
          code: "UNKNOWN_ERROR",
          message: `API request failed: ${response.status} ${response.statusText}`,
          details: null,
        };
      }
      throw new ApiErrorClass(apiError);
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

    if (response.status === 401) {
      authStore.handleUnauthorized();
    }

    if (!response.ok) {
      let apiError: ApiError;
      try {
        apiError = (await response.json()) as ApiError;
      } catch {
        apiError = {
          code: "UNKNOWN_ERROR",
          message: `API request failed: ${response.status} ${response.statusText}`,
          details: null,
        };
      }
      throw new ApiErrorClass(apiError);
    }
  }

  async getStatus(): Promise<StatusResponse> {
    return this.request<StatusResponse>("/status");
  }

  async searchCourses(params: Partial<SearchParams> & { term: string }): Promise<SearchResponse> {
    const query = new URLSearchParams();
    query.set("term", params.term);
    if (params.subject && params.subject.length > 0) {
      for (const s of params.subject) {
        query.append("subject", s);
      }
    }
    if (params.q) query.set("q", params.q);
    if (params.openOnly) query.set("open_only", "true");
    if (params.courseNumberLow !== undefined && params.courseNumberLow !== null) {
      query.set("course_number_low", String(params.courseNumberLow));
    }
    if (params.courseNumberHigh !== undefined && params.courseNumberHigh !== null) {
      query.set("course_number_high", String(params.courseNumberHigh));
    }
    if (params.instructionalMethod && params.instructionalMethod.length > 0) {
      for (const m of params.instructionalMethod) {
        query.append("instructional_method", m);
      }
    }
    if (params.campus && params.campus.length > 0) {
      for (const c of params.campus) {
        query.append("campus", c);
      }
    }
    if (params.waitCountMax !== undefined && params.waitCountMax !== null) {
      query.set("wait_count_max", String(params.waitCountMax));
    }
    if (params.days && params.days.length > 0) {
      for (const d of params.days) {
        query.append("days", d);
      }
    }
    if (params.timeStart) query.set("time_start", params.timeStart);
    if (params.timeEnd) query.set("time_end", params.timeEnd);
    if (params.partOfTerm && params.partOfTerm.length > 0) {
      for (const p of params.partOfTerm) {
        query.append("part_of_term", p);
      }
    }
    if (params.attributes && params.attributes.length > 0) {
      for (const a of params.attributes) {
        query.append("attributes", a);
      }
    }
    if (params.creditHourMin !== undefined && params.creditHourMin !== null) {
      query.set("credit_hour_min", String(params.creditHourMin));
    }
    if (params.creditHourMax !== undefined && params.creditHourMax !== null) {
      query.set("credit_hour_max", String(params.creditHourMax));
    }
    if (params.instructor) query.set("instructor", params.instructor);
    if (params.limit !== undefined) query.set("limit", String(params.limit));
    if (params.offset !== undefined) query.set("offset", String(params.offset));
    if (params.sortBy) query.set("sort_by", params.sortBy);
    if (params.sortDir) query.set("sort_dir", params.sortDir);
    return this.request<SearchResponse>(`/courses/search?${query.toString()}`);
  }

  async getTerms(): Promise<Term[]> {
    return this.request<Term[]>("/terms");
  }

  async getSubjects(term: string): Promise<Subject[]> {
    return this.request<Subject[]>(`/subjects?term=${encodeURIComponent(term)}`);
  }

  async getReference(category: string): Promise<ReferenceEntry[]> {
    return this.request<ReferenceEntry[]>(`/reference/${encodeURIComponent(category)}`);
  }

  // Admin endpoints
  async getAdminStatus(): Promise<AdminStatusResponse> {
    return this.request<AdminStatusResponse>("/admin/status");
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

  async getTimeline(ranges: TimeRange[]): Promise<TimelineResponse> {
    return this.request<TimelineResponse>("/timeline", {
      method: "POST",
      body: { ranges } satisfies TimelineRequest,
    });
  }

  async getMetrics(params?: Partial<MetricsParams>): Promise<MetricsResponse> {
    const query = new URLSearchParams();
    if (params?.courseId !== undefined && params.courseId !== null) {
      query.set("course_id", String(params.courseId));
    }
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

  // Scraper analytics endpoints

  async getScraperStats(period?: ScraperPeriod): Promise<ScraperStatsResponse> {
    const qs = period ? `?period=${period}` : "";
    return this.request<ScraperStatsResponse>(`/admin/scraper/stats${qs}`);
  }

  async getScraperTimeseries(period?: ScraperPeriod, bucket?: string): Promise<TimeseriesResponse> {
    const query = new URLSearchParams();
    if (period) query.set("period", period);
    if (bucket) query.set("bucket", bucket);
    const qs = query.toString();
    return this.request<TimeseriesResponse>(`/admin/scraper/timeseries${qs ? `?${qs}` : ""}`);
  }

  async getScraperSubjects(): Promise<SubjectsResponse> {
    return this.request<SubjectsResponse>("/admin/scraper/subjects");
  }

  async getScraperSubjectDetail(subject: string, limit?: number): Promise<SubjectDetailResponse> {
    const qs = limit !== undefined ? `?limit=${limit}` : "";
    return this.request<SubjectDetailResponse>(
      `/admin/scraper/subjects/${encodeURIComponent(subject)}${qs}`
    );
  }
}

export const client = new BannerApiClient();
