import type { ScrapeJob } from "$lib/api";

export type ScrapeJobStatus = "processing" | "staleLock" | "exhausted" | "scheduled" | "pending";

export type ScrapeJobEvent =
  | { type: "init"; jobs: ScrapeJob[] }
  | { type: "jobCreated"; job: ScrapeJob }
  | { type: "jobLocked"; id: number; lockedAt: string; status: ScrapeJobStatus }
  | { type: "jobCompleted"; id: number }
  | {
      type: "jobRetried";
      id: number;
      retryCount: number;
      queuedAt: string;
      status: ScrapeJobStatus;
    }
  | { type: "jobExhausted"; id: number }
  | { type: "jobDeleted"; id: number };

export type ConnectionState = "connected" | "reconnecting" | "disconnected";

const PRIORITY_ORDER: Record<string, number> = {
  critical: 0,
  high: 1,
  medium: 2,
  low: 3,
};

const MAX_RECONNECT_DELAY = 30_000;
const MAX_RECONNECT_ATTEMPTS = 10;

function sortJobs(jobs: Iterable<ScrapeJob>): ScrapeJob[] {
  return Array.from(jobs).sort((a, b) => {
    const pa = PRIORITY_ORDER[a.priority.toLowerCase()] ?? 2;
    const pb = PRIORITY_ORDER[b.priority.toLowerCase()] ?? 2;
    if (pa !== pb) return pa - pb;
    return new Date(a.executeAt).getTime() - new Date(b.executeAt).getTime();
  });
}

export class ScrapeJobsStore {
  private ws: WebSocket | null = null;
  private jobs = new Map<number, ScrapeJob>();
  private _connectionState: ConnectionState = "disconnected";
  private _initialized = false;
  private onUpdate: () => void;
  private reconnectAttempts = 0;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private intentionalClose = false;

  /** Cached sorted array, invalidated on data mutations. */
  private cachedJobs: ScrapeJob[] = [];
  private cacheDirty = false;

  constructor(onUpdate: () => void) {
    this.onUpdate = onUpdate;
  }

  getJobs(): ScrapeJob[] {
    if (this.cacheDirty) {
      this.cachedJobs = sortJobs(this.jobs.values());
      this.cacheDirty = false;
    }
    return this.cachedJobs;
  }

  getConnectionState(): ConnectionState {
    return this._connectionState;
  }

  isInitialized(): boolean {
    return this._initialized;
  }

  connect(): void {
    this.intentionalClose = false;
    const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
    const url = `${protocol}//${window.location.host}/api/admin/scrape-jobs/ws`;

    try {
      this.ws = new WebSocket(url);
    } catch {
      this.scheduleReconnect();
      return;
    }

    this.ws.onopen = () => {
      this._connectionState = "connected";
      this.reconnectAttempts = 0;
      this.onUpdate();
    };

    this.ws.onmessage = (event) => {
      try {
        const parsed = JSON.parse(event.data as string) as ScrapeJobEvent;
        this.handleEvent(parsed);
      } catch {
        // Ignore malformed messages
      }
    };

    this.ws.onclose = () => {
      this.ws = null;
      if (!this.intentionalClose) {
        this.scheduleReconnect();
      }
    };

    this.ws.onerror = () => {
      // onclose will fire after onerror, so reconnect is handled there
    };
  }

  handleEvent(event: ScrapeJobEvent): void {
    switch (event.type) {
      case "init":
        this.jobs.clear();
        for (const job of event.jobs) {
          this.jobs.set(job.id, job);
        }
        this._initialized = true;
        break;
      case "jobCreated":
        this.jobs.set(event.job.id, event.job);
        break;
      case "jobLocked": {
        const job = this.jobs.get(event.id);
        if (job) {
          this.jobs.set(event.id, { ...job, lockedAt: event.lockedAt, status: event.status });
        }
        break;
      }
      case "jobCompleted":
        this.jobs.delete(event.id);
        break;
      case "jobRetried": {
        const job = this.jobs.get(event.id);
        if (job) {
          this.jobs.set(event.id, {
            ...job,
            retryCount: event.retryCount,
            queuedAt: event.queuedAt,
            status: event.status,
            lockedAt: null,
          });
        }
        break;
      }
      case "jobExhausted": {
        const job = this.jobs.get(event.id);
        if (job) {
          this.jobs.set(event.id, { ...job, status: "exhausted" });
        }
        break;
      }
      case "jobDeleted":
        this.jobs.delete(event.id);
        break;
    }
    this.cacheDirty = true;
    this.onUpdate();
  }

  disconnect(): void {
    this.intentionalClose = true;
    if (this.reconnectTimer !== null) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
    this._connectionState = "disconnected";
    this.onUpdate();
  }

  resync(): void {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify({ type: "resync" }));
    }
  }

  /** Attempt to reconnect after being disconnected. Resets attempt counter. */
  retry(): void {
    this.reconnectAttempts = 0;
    this._connectionState = "reconnecting";
    this.onUpdate();
    this.connect();
  }

  private scheduleReconnect(): void {
    if (this.reconnectAttempts >= MAX_RECONNECT_ATTEMPTS) {
      this._connectionState = "disconnected";
      this.onUpdate();
      return;
    }

    this._connectionState = "reconnecting";
    this.onUpdate();

    const delay = Math.min(1000 * 2 ** this.reconnectAttempts, MAX_RECONNECT_DELAY);
    this.reconnectAttempts++;

    this.reconnectTimer = setTimeout(() => {
      this.reconnectTimer = null;
      this.connect();
    }, delay);
  }
}
