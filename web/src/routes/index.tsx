import { createFileRoute } from "@tanstack/react-router";
import { useState, useEffect } from "react";
import {
  apiClient,
  type HealthResponse,
  type StatusResponse,
} from "../lib/api";
import logo from "../logo.svg";
import "../App.css";

export const Route = createFileRoute("/")({
  component: App,
});

function App() {
  const [health, setHealth] = useState<HealthResponse | null>(null);
  const [status, setStatus] = useState<StatusResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchData = async () => {
      try {
        setLoading(true);
        const [healthData, statusData] = await Promise.all([
          apiClient.getHealth(),
          apiClient.getStatus(),
        ]);
        setHealth(healthData);
        setStatus(statusData);
        setError(null);
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to fetch data");
      } finally {
        setLoading(false);
      }
    };

    fetchData();
  }, []);

  return (
    <div className="App">
      <header className="App-header">
        <img src={logo} className="App-logo" alt="logo" />
        <h1>Banner Discord Bot Dashboard</h1>

        {loading && <p>Loading...</p>}

        {error && (
          <div style={{ color: "red", margin: "20px 0" }}>
            <p>Error: {error}</p>
          </div>
        )}

        {health && (
          <div style={{ margin: "20px 0", textAlign: "left" }}>
            <h3>Health Status</h3>
            <p>Status: {health.status}</p>
            <p>Timestamp: {new Date(health.timestamp).toLocaleString()}</p>
          </div>
        )}

        {status && (
          <div style={{ margin: "20px 0", textAlign: "left" }}>
            <h3>System Status</h3>
            <p>Overall: {status.status}</p>
            <p>Bot: {status.bot.status}</p>
            <p>Cache: {status.cache.status}</p>
            <p>Banner API: {status.banner_api.status}</p>
          </div>
        )}

        <div style={{ marginTop: "40px" }}>
          <a
            className="App-link"
            href="https://tanstack.com"
            target="_blank"
            rel="noopener noreferrer"
          >
            Learn TanStack Router
          </a>
        </div>
      </header>
    </div>
  );
}
