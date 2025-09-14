import { createFileRoute } from "@tanstack/react-router";
import { useState, useEffect } from "react";
import { client, type StatusResponse, type Status } from "../lib/api";
import { Card, Flex, Text, Tooltip, Skeleton } from "@radix-ui/themes";
import {
  CheckCircle,
  XCircle,
  Clock,
  Bot,
  Globe,
  Hourglass,
  Activity,
  MessageCircle,
  Circle,
  WifiOff,
} from "lucide-react";
import TimeAgo from "react-timeago";
import { ThemeToggle } from "../components/ThemeToggle";
import "../App.css";

export const Route = createFileRoute("/")({
  component: App,
});

// Constants
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
  borderTop: "1px solid #e2e8f0",
} as const;

// Service icon mapping
const SERVICE_ICONS: Record<string, typeof Bot> = {
  bot: Bot,
  banner: Globe,
  discord: MessageCircle,
};

// Types
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

// Helper functions
const getStatusIcon = (status: Status | "Unreachable"): StatusIcon => {
  const statusMap: Record<Status | "Unreachable", StatusIcon> = {
    Active: { icon: CheckCircle, color: "green" },
    Connected: { icon: CheckCircle, color: "green" },
    Healthy: { icon: CheckCircle, color: "green" },
    Disabled: { icon: Circle, color: "gray" },
    Error: { icon: XCircle, color: "red" },
    Unreachable: { icon: WifiOff, color: "red" },
  };

  return statusMap[status];
};

const getOverallHealth = (state: StatusState): Status | "Unreachable" => {
  if (state.mode === "timeout") return "Unreachable";
  if (state.mode === "error") return "Error";
  if (state.mode === "response") return state.status.status;
  return "Error";
};

const getServices = (state: StatusState): Service[] => {
  if (state.mode !== "response") return [];

  return Object.entries(state.status.services).map(
    ([serviceId, serviceInfo]) => ({
      name: serviceInfo.name,
      status: serviceInfo.status,
      icon: SERVICE_ICONS[serviceId] || SERVICE_ICONS.default,
    })
  );
};

// Status Component
const StatusDisplay = ({ status }: { status: Status | "Unreachable" }) => {
  const { icon: Icon, color } = getStatusIcon(status);

  return (
    <Flex align="center" gap="2">
      <Text
        size="2"
        style={{
          color: status === "Disabled" ? "#8B949E" : undefined,
          opacity: status === "Disabled" ? 0.7 : undefined,
        }}
      >
        {status}
      </Text>
      <Icon color={color} size={16} />
    </Flex>
  );
};

// Service Status Component
const ServiceStatus = ({ service }: { service: Service }) => {
  return (
    <Flex align="center" justify="between">
      <Flex align="center" gap="2">
        <service.icon size={18} />
        <Text>{service.name}</Text>
      </Flex>
      <StatusDisplay status={service.status} />
    </Flex>
  );
};

// Skeleton Service Component
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

// Timing Row Component
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

  // Helper variables for state checking
  const isLoading = state.mode === "loading";
  const hasError = state.mode === "error";
  const hasTimeout = state.mode === "timeout";
  const hasResponse = state.mode === "response";
  const shouldShowSkeleton = isLoading || hasError;
  const shouldShowTiming = hasResponse && state.timing.health !== null;
  const shouldShowLastFetch = hasResponse || hasError || hasTimeout;

  useEffect(() => {
    let timeoutId: NodeJS.Timeout;

    const fetchData = async () => {
      try {
        const startTime = Date.now();

        // Create a timeout promise
        const timeoutPromise = new Promise<never>((_, reject) => {
          setTimeout(
            () => reject(new Error("Request timeout")),
            REQUEST_TIMEOUT
          );
        });

        // Race between the API call and timeout
        const statusData = await Promise.race([
          client.getStatus(),
          timeoutPromise,
        ]);

        const endTime = Date.now();
        const responseTime = endTime - startTime;

        setState({
          mode: "response",
          status: statusData,
          timing: { health: responseTime, status: responseTime },
          lastFetch: new Date(),
        });
      } catch (err) {
        const errorMessage =
          err instanceof Error ? err.message : "Failed to fetch data";

        // Check if it's a timeout error
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
      timeoutId = setTimeout(fetchData, REFRESH_INTERVAL);
    };

    // Start the first request immediately
    fetchData();

    return () => {
      if (timeoutId) {
        clearTimeout(timeoutId);
      }
    };
  }, []);

  const overallHealth = getOverallHealth(state);
  const { color: overallColor } = getStatusIcon(overallHealth);
  const services = getServices(state);

  return (
    <div className="App">
      {/* Theme Toggle - Fixed position in top right */}
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
                <Text size="4">System Status</Text>
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
                  Array.from({ length: 3 }).map((_, index) => (
                    <SkeletonService key={index} />
                  ))
                : services.map((service) => (
                    <ServiceStatus key={service.name} service={service} />
                  ))}
            </Flex>

            <Flex direction="column" gap="2" style={BORDER_STYLES}>
              {isLoading ? (
                <TimingRow icon={Hourglass} name="Response Time">
                  <Skeleton height="18px" width="50px" />
                </TimingRow>
              ) : shouldShowTiming ? (
                <TimingRow icon={Hourglass} name="Response Time">
                  <Text size="2">{state.timing.health}ms</Text>
                </TimingRow>
              ) : null}

              {shouldShowLastFetch ? (
                <TimingRow icon={Clock} name="Last Updated">
                  {isLoading ? (
                    <Text
                      size="2"
                      style={{ paddingBottom: "2px" }}
                      color="gray"
                    >
                      Loading...
                    </Text>
                  ) : (
                    <Tooltip
                      content={`as of ${state.lastFetch.toLocaleTimeString()}`}
                    >
                      <abbr
                        style={{
                          cursor: "pointer",
                          textDecoration: "underline",
                          textDecorationStyle: "dotted",
                          textDecorationColor: "#CBCED1",
                          textUnderlineOffset: "6px",
                        }}
                      >
                        <Text size="2">
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
        <Flex
          justify="center"
          style={{ marginTop: "12px" }}
          gap="2"
          align="center"
        >
          {__APP_VERSION__ && (
            <Text
              size="1"
              style={{
                color: "#8B949E",
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
                backgroundColor: "#8B949E",
                opacity: 0.3,
              }}
            />
          )}
          <Text
            size="1"
            style={{
              color: "#8B949E",
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
