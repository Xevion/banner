import { createFileRoute } from "@tanstack/react-router";
import { useState, useEffect } from "react";
import { apiClient, type StatusResponse } from "../lib/api";
import { Card, Flex, Text, Tooltip } from "@radix-ui/themes";
import {
  CheckCircle,
  AlertCircle,
  XCircle,
  Clock,
  Bot,
  Database,
  Globe,
  Hourglass,
  Activity,
} from "lucide-react";
import TimeAgo from "react-timeago";
import "../App.css";

export const Route = createFileRoute("/")({
  component: App,
});

// Constants
const REFRESH_INTERVAL = import.meta.env.DEV ? 3000 : 30000;
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

// Types
type HealthStatus = "healthy" | "warning" | "error" | "unknown";
type ServiceStatus = "running" | "connected" | "disconnected" | "error";

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
  status: ServiceStatus;
  icon: typeof Bot;
}

// Helper functions
const getStatusIcon = (status: string): StatusIcon => {
  const statusMap: Record<string, StatusIcon> = {
    healthy: { icon: CheckCircle, color: "green" },
    running: { icon: CheckCircle, color: "green" },
    connected: { icon: CheckCircle, color: "green" },
    warning: { icon: AlertCircle, color: "orange" },
    error: { icon: XCircle, color: "red" },
    disconnected: { icon: XCircle, color: "red" },
  };

  return statusMap[status] || { icon: XCircle, color: "red" };
};

const getOverallHealth = (
  status: StatusResponse | null,
  error: string | null
): HealthStatus => {
  if (error) return "error";
  if (!status) return "unknown";

  const allHealthy =
    status.bot.status === "running" &&
    status.cache.status === "connected" &&
    status.banner_api.status === "connected";

  return allHealthy ? "healthy" : "warning";
};

const getServices = (status: StatusResponse | null): Service[] => {
  if (!status) return [];

  return [
    { name: "Bot", status: status.bot.status as ServiceStatus, icon: Bot },
    {
      name: "Cache",
      status: status.cache.status as ServiceStatus,
      icon: Database,
    },
    {
      name: "Banner API",
      status: status.banner_api.status as ServiceStatus,
      icon: Globe,
    },
  ];
};

// Service Status Component
const ServiceStatus = ({ service }: { service: Service }) => {
  const { icon: Icon, color } = getStatusIcon(service.status);

  return (
    <Flex align="center" justify="between">
      <Flex align="center" gap="2">
        <service.icon size={18} />
        <Text>{service.name}</Text>
      </Flex>
      <Flex align="center" gap="2">
        <Icon color={color} size={16} />
        <Text size="2">{service.status}</Text>
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
  const [status, setStatus] = useState<StatusResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [timing, setTiming] = useState<ResponseTiming>({
    health: null,
    status: null,
  });
  const [lastFetch, setLastFetch] = useState<Date | null>(null);

  useEffect(() => {
    const fetchData = async () => {
      try {
        const startTime = Date.now();
        const statusData = await apiClient.getStatus();
        const endTime = Date.now();
        const responseTime = endTime - startTime;

        setStatus(statusData);
        setTiming({ health: responseTime, status: responseTime });
        setLastFetch(new Date());
        setError(null);
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to fetch data");
        setLastFetch(new Date());
      }
    };

    fetchData();
    const interval = setInterval(fetchData, REFRESH_INTERVAL);
    return () => clearInterval(interval);
  }, []);

  const overallHealth = getOverallHealth(status, error);
  const { color: overallColor } = getStatusIcon(overallHealth);
  const services = getServices(status);

  return (
    <div className="App">
      <Flex
        direction="column"
        align="center"
        justify="center"
        style={{ minHeight: "100vh", padding: "20px" }}
      >
        {status && (
          <Card style={CARD_STYLES}>
            <Flex direction="column" gap="4">
              {/* Overall Status */}
              <Flex align="center" justify="between">
                <Flex align="center" gap="2">
                  <Activity color={overallColor} size={18} />
                  <Text size="4">System Status</Text>
                </Flex>
              </Flex>

              {/* Individual Services */}
              <Flex direction="column" gap="3" style={{ marginTop: "16px" }}>
                {services.map((service) => (
                  <ServiceStatus key={service.name} service={service} />
                ))}
              </Flex>

              <Flex direction="column" gap="2" style={BORDER_STYLES}>
                {timing.health && (
                  <TimingRow icon={Hourglass} name="Response Time">
                    <Text size="2">{timing.health}ms</Text>
                  </TimingRow>
                )}

                {lastFetch && (
                  <TimingRow icon={Clock} name="Last Updated">
                    <Tooltip
                      content={`as of ${lastFetch.toLocaleTimeString()}`}
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
                          <TimeAgo date={lastFetch} />
                        </Text>
                      </abbr>
                    </Tooltip>
                  </TimingRow>
                )}
              </Flex>
            </Flex>
          </Card>
        )}
        {(status?.git?.commit || status?.version) && (
          <Flex
            justify="center"
            style={{ marginTop: "12px" }}
            gap="2"
            align="center"
          >
            {status?.version && (
              <Text
                size="1"
                style={{
                  color: "#8B949E",
                }}
              >
                v{status.version}
              </Text>
            )}
            {status?.version && status?.git?.commit && (
              <div
                style={{
                  width: "1px",
                  height: "12px",
                  backgroundColor: "#8B949E",
                  opacity: 0.3,
                }}
              />
            )}
            {status?.git?.commit && (
              <Text
                size="1"
                style={{
                  color: "#8B949E",
                  textDecoration: "none",
                }}
              >
                <a
                  href={`https://github.com/Xevion/banner/commit/${status.git.commit}`}
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
            )}
          </Flex>
        )}
      </Flex>
    </div>
  );
}
