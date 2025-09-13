import { Outlet, createRootRoute } from "@tanstack/react-router";
import { TanStackRouterDevtoolsPanel } from "@tanstack/react-router-devtools";
import { TanstackDevtools } from "@tanstack/react-devtools";
import { Theme } from "@radix-ui/themes";
import "@radix-ui/themes/styles.css";

export const Route = createRootRoute({
  component: () => (
    <Theme appearance="light" accentColor="blue" grayColor="gray">
      <Outlet />
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
    </Theme>
  ),
});
