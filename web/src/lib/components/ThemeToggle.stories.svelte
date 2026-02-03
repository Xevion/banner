<script module>
import { defineMeta } from "@storybook/addon-svelte-csf";
import ThemeToggle from "./ThemeToggle.svelte";

const { Story } = defineMeta({
  title: "Components/ThemeToggle",
  component: ThemeToggle,
  tags: ["autodocs"],
  parameters: {
    docs: {
      description: {
        component:
          "A theme toggle button that switches between light and dark mode with a smooth circular reveal animation using the View Transitions API. Shows a Sun icon in light mode and Moon icon in dark mode. The component reads from the global themeStore and has no props.",
      },
    },
  },
});
</script>

<Story name="Default" />

<Story name="In Context">
  <div class="flex items-center gap-4 p-4 bg-card border border-border rounded-lg">
    <span class="text-sm text-foreground">Toggle theme:</span>
    <ThemeToggle />
  </div>
</Story>
