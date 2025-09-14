import { useTheme } from "next-themes";
import { Button } from "@radix-ui/themes";
import { Sun, Moon, Monitor } from "lucide-react";
import { useMemo } from "react";

export function ThemeToggle() {
  const { theme, setTheme } = useTheme();

  const nextTheme = useMemo(() => {
    switch (theme) {
      case "light":
        return "dark";
      case "dark":
        return "system";
      case "system":
        return "light";
      default:
        console.error(`Invalid theme: ${theme}`);
        return "system";
    }
  }, [theme]);

  const icon = useMemo(() => {
    if (nextTheme === "system") {
      return <Monitor size={18} />;
    }
    return nextTheme === "dark" ? <Moon size={18} /> : <Sun size={18} />;
  }, [nextTheme]);

  return (
    <Button
      variant="ghost"
      size="3"
      onClick={() => setTheme(nextTheme)}
      style={{
        cursor: "pointer",
        backgroundColor: "transparent",
        border: "none",
        margin: "4px",
        padding: "7px",
        borderRadius: "6px",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        color: "var(--gray-11)",
        transition: "background-color 0.2s, color 0.2s",
        transform: "scale(1.25)",
      }}
      onMouseEnter={(e) => {
        e.currentTarget.style.backgroundColor = "var(--gray-4)";
      }}
      onMouseLeave={(e) => {
        e.currentTarget.style.backgroundColor = "transparent";
      }}
    >
      {icon}
      <span className="sr-only">Toggle theme</span>
    </Button>
  );
}
