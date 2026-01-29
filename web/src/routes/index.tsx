import { Card, Flex, Skeleton, Text, Tooltip } from "@radix-ui/themes";
import { createFileRoute } from "@tanstack/react-router";
import {
  Activity,
  Bot,
  CheckCircle,
  Circle,
  Clock,
  Globe,
  Hourglass,
  MessageCircle,
  WifiOff,
  XCircle,
} from "lucide-react";
import { useEffect, useState } from "react";
import TimeAgo from "react-timeago";
import { ThemeToggle } from "../components/ThemeToggle";
import { type Status, type StatusResponse, client } from "../lib/api";
import "../App.css";

const REFRESH_INTERVAL = import.meta.env.DEV ? 3000 : 30000;
const REQUEST_TIMEOUT = 10000; // 10 seconds

const CARD_STYLES = {
  padding: "24px",
  maxWidth: "400px",
  width: "100%",
} as const;

const BORDER_STYLES = {
  marginTop: "16px",
  paddingTop: "16px",
  borderTop: "1px solid var(--gray-7)",
} as const;

const SERVICE_ICONS: Record<string, typeof Bot> = {
  bot: Bot,
  banner: Globe,
  discord: MessageCircle,
  database: Activity,
  web: Globe,
  scraper: Clock,
};

interface ResponseTiming {
  health: number | null;
  status: number | null;
}

interface StatusIcon {
  icon: typeof CheckCircle;
  color: string;
}

interface Service {
  name: string;
  status: Status;
  icon: typeof Bot;
}

type StatusState =
  | {
      mode: "loading";
    }
  | {
      mode: "response";
      timing: ResponseTiming;
      lastFetch: Date;
      status: StatusResponse;
    }
  | {
      mode: "error";
      lastFetch: Date;
    }
  | {
      mode: "timeout";
      lastFetch: Date;
    };

const formatNumber = (num: number): string => {
  return num.toLocaleString();
};

const getStatusIcon = (status: Status | "Unreachable"): StatusIcon => {
  const statusMap: Record<Status | "Unreachable", StatusIcon> = {
    active: { icon: CheckCircle, color: "green" },
    connected: { icon: CheckCircle, color: "green" },
    starting: { icon: Hourglass, color: "orange" },
    disabled: { icon: Circle, color: "gray" },
    error: { icon: XCircle, color: "red" },
    Unreachable: { icon: WifiOff, color: "red" },
  };

  return statusMap[status];
};

const getOverallHealth = (state: StatusState): Status | "Unreachable" => {
  if (state.mode === "timeout") return "Unreachable";
  if (state.mode === "error") return "error";
  if (state.mode === "response") return state.status.status;
  return "error";
};

const getServices = (state: StatusState): Service[] => {
  if (state.mode !== "response") return [];

  return Object.entries(state.status.services).map(([serviceId, serviceInfo]) => ({
    name: serviceInfo.name,
    status: serviceInfo.status,
    icon: SERVICE_ICONS[serviceId] || SERVICE_ICONS.default,
  }));
};

const StatusDisplay = ({ status }: { status: Status | "Unreachable" }) => {
  const { icon: Icon, color } = getStatusIcon(status);

  return (
    <Flex align="center" gap="2">
      <Text
        size="2"
        style={{
          color: status === "disabled" ? "var(--gray-11)" : undefined,
          opacity: status === "disabled" ? 0.7 : undefined,
        }}
      >
        {status}
      </Text>
      <Icon color={color} size={16} />
    </Flex>
  );
};

const ServiceStatus = ({ service }: { service: Service }) => {
  return (
    <Flex align="center" justify="between">
      <Flex align="center" gap="2">
        <service.icon size={18} />
        <Text style={{ color: "var(--gray-11)" }}>{service.name}</Text>
      </Flex>
      <StatusDisplay status={service.status} />
    </Flex>
  );
};

const SkeletonService = () => {
  return (
    <Flex align="center" justify="between">
      <Flex align="center" gap="2">
        <Skeleton height="24px" width="18px" />
        <Skeleton height="24px" width="60px" />
      </Flex>
      <Flex align="center" gap="2">
        <Skeleton height="20px" width="50px" />
        <Skeleton height="20px" width="16px" />
      </Flex>
    </Flex>
  );
};

const TimingRow = ({
  icon: Icon,
  name,
  children,
}: {
  icon: React.ComponentType<{ size?: number }>;
  name: string;
  children: React.ReactNode;
}) => (
  <Flex align="center" justify="between">
    <Flex align="center" gap="2">
      <Icon size={13} />
      <Text size="2" color="gray">
        {name}
      </Text>
    </Flex>
    {children}
  </Flex>
);

function App() {
  const [state, setState] = useState<StatusState>({ mode: "loading" });

  // State helpers
  const isLoading = state.mode === "loading";
  const hasError = state.mode === "error";
  const hasTimeout = state.mode === "timeout";
  const hasResponse = state.mode === "response";
  const shouldShowSkeleton = isLoading || hasError;
  const shouldShowTiming = hasResponse && state.timing.health !== null;
  const shouldShowLastFetch = hasResponse || hasError || hasTimeout;

  useEffect(() => {
    let timeoutId: NodeJS.Timeout | null = null;
    let requestTimeoutId: NodeJS.Timeout | null = null;

    const fetchData = async () => {
      try {
        const startTime = Date.now();

        // Create a timeout promise with cleanup tracking
        const timeoutPromise = new Promise<never>((_, reject) => {
          requestTimeoutId = setTimeout(() => {
            reject(new Error("Request timeout"));
          }, REQUEST_TIMEOUT);
        });

        // Race between the API call and timeout
        const statusData = await Promise.race([client.getStatus(), timeoutPromise]);

        // Clear the timeout if the request succeeded
        if (requestTimeoutId) {
          clearTimeout(requestTimeoutId);
          requestTimeoutId = null;
        }

        const endTime = Date.now();
        const responseTime = endTime - startTime;

        setState({
          mode: "response",
          status: statusData,
          timing: { health: responseTime, status: responseTime },
          lastFetch: new Date(),
        });
      } catch (err) {
        // Clear the timeout on error as well
        if (requestTimeoutId) {
          clearTimeout(requestTimeoutId);
          requestTimeoutId = null;
        }

        const errorMessage = err instanceof Error ? err.message : "Failed to fetch data";

        if (errorMessage === "Request timeout") {
          setState({
            mode: "timeout",
            lastFetch: new Date(),
          });
        } else {
          setState({
            mode: "error",
            lastFetch: new Date(),
          });
        }
      }

      // Schedule the next request after the current one completes
      timeoutId = setTimeout(() => void fetchData(), REFRESH_INTERVAL);
    };

    // Start the first request immediately
    void fetchData();

    return () => {
      if (timeoutId) {
        clearTimeout(timeoutId);
      }
      if (requestTimeoutId) {
        clearTimeout(requestTimeoutId);
      }
    };
  }, []);

  const overallHealth = getOverallHealth(state);
  const { color: overallColor } = getStatusIcon(overallHealth);
  const services = getServices(state);

  return (
    <div className="App">
      <div
        style={{
          position: "fixed",
          top: "20px",
          right: "20px",
          zIndex: 1000,
        }}
      >
        <ThemeToggle />
      </div>

      <Flex
        direction="column"
        align="center"
        justify="center"
        style={{ minHeight: "100vh", padding: "20px" }}
      >
        <Card style={CARD_STYLES}>
          <Flex direction="column" gap="4">
            {/* Overall Status */}
            <Flex align="center" justify="between">
              <Flex align="center" gap="2">
                <Activity
                  color={isLoading ? undefined : overallColor}
                  size={18}
                  className={isLoading ? "animate-pulse" : ""}
                  style={{
                    opacity: isLoading ? 0.3 : 1,
                    transition: "opacity 2s ease-in-out, color 2s ease-in-out",
                  }}
                />
                <Text size="4" style={{ color: "var(--gray-12)" }}>
                  System Status
                </Text>
              </Flex>
              {isLoading ? (
                <Skeleton height="20px" width="80px" />
              ) : (
                <StatusDisplay status={overallHealth} />
              )}
            </Flex>

            {/* Individual Services */}
            <Flex direction="column" gap="3" style={{ marginTop: "16px" }}>
              {shouldShowSkeleton
                ? // Show skeleton for 3 services during initial loading only
                  Array.from({ length: 3 }).map((_, index) => <SkeletonService key={index} />)
                : services.map((service) => <ServiceStatus key={service.name} service={service} />)}
            </Flex>

            <Flex direction="column" gap="2" style={BORDER_STYLES}>
              {isLoading ? (
                <TimingRow icon={Hourglass} name="Response Time">
                  <Skeleton height="18px" width="50px" />
                </TimingRow>
              ) : shouldShowTiming ? (
                <TimingRow icon={Hourglass} name="Response Time">
                  <Text size="2" style={{ color: "var(--gray-11)" }}>
                    {formatNumber(state.timing.health!)}ms
                  </Text>
                </TimingRow>
              ) : null}

              {shouldShowLastFetch ? (
                <TimingRow icon={Clock} name="Last Updated">
                  {isLoading ? (
                    <Text size="2" style={{ paddingBottom: "2px" }} color="gray">
                      Loading...
                    </Text>
                  ) : (
                    <Tooltip content={`as of ${state.lastFetch.toLocaleTimeString()}`}>
                      <abbr
                        style={{
                          cursor: "pointer",
                          textDecoration: "underline",
                          textDecorationStyle: "dotted",
                          textDecorationColor: "var(--gray-6)",
                          textUnderlineOffset: "6px",
                        }}
                      >
                        <Text size="2" style={{ color: "var(--gray-11)" }}>
                          <TimeAgo date={state.lastFetch} />
                        </Text>
                      </abbr>
                    </Tooltip>
                  )}
                </TimingRow>
              ) : isLoading ? (
                <TimingRow icon={Clock} name="Last Updated">
                  <Text size="2" color="gray">
                    Loading...
                  </Text>
                </TimingRow>
              ) : null}
            </Flex>
          </Flex>
        </Card>
        <Flex justify="center" style={{ marginTop: "12px" }} gap="2" align="center">
          {__APP_VERSION__ && (
            <Text
              size="1"
              style={{
                color: "var(--gray-11)",
              }}
            >
              v{__APP_VERSION__}
            </Text>
          )}
          {__APP_VERSION__ && (
            <div
              style={{
                width: "1px",
                height: "12px",
                backgroundColor: "var(--gray-10)",
                opacity: 0.3,
              }}
            />
          )}
          <Text
            size="1"
            style={{
              color: "var(--gray-11)",
              textDecoration: "none",
            }}
          >
            <a
              href={
                hasResponse && state.status.commit
                  ? `https://github.com/Xevion/banner/commit/${state.status.commit}`
                  : "https://github.com/Xevion/banner"
              }
              target="_blank"
              rel="noopener noreferrer"
              style={{
                color: "inherit",
                textDecoration: "none",
              }}
            >
              GitHub
            </a>
          </Text>
        </Flex>
      </Flex>
    </div>
  );
}

export const Route = createFileRoute("/")({
  component: App,
});
