import { Outlet, createRootRoute } from "@tanstack/react-router";
import { TanStackRouterDevtoolsPanel } from "@tanstack/react-router-devtools";
import { TanstackDevtools } from "@tanstack/react-devtools";
import { Theme } from "@radix-ui/themes";
import "@radix-ui/themes/styles.css";
import { ThemeProvider } from "next-themes";

export const Route = createRootRoute({
  component: () => (
    <ThemeProvider
      attribute="class"
      defaultTheme="system"
      enableSystem
      disableTransitionOnChange={false}
    >
      <Theme accentColor="blue" grayColor="gray">
        <Outlet />
        {import.meta.env.DEV ? (
          <TanstackDevtools
            config={{
              position: "bottom-left",
            }}
            plugins={[
              {
                name: "Tanstack Router",
                render: <TanStackRouterDevtoolsPanel />,
              },
            ]}
          />
        ) : null}
      </Theme>
    </ThemeProvider>
  ),
});
