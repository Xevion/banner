import { Button } from "@radix-ui/themes";
import { Monitor, Moon, Sun } from "lucide-react";
import { useTheme } from "next-themes";
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
    <Button variant="ghost" size="3" onClick={() => setTheme(nextTheme)} className="theme-toggle">
      {icon}
      <span className="sr-only">Toggle theme</span>
    </Button>
  );
}
