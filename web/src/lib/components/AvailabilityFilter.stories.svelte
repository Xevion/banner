<script module>
import { defineMeta } from "@storybook/addon-svelte-csf";
import AvailabilityFilter from "./AvailabilityFilter.svelte";

const { Story } = defineMeta({
  title: "Components/AvailabilityFilter",
  component: AvailabilityFilter,
  tags: ["autodocs"],
});
</script>

<Story
  name="Default"
  args={{ campus: [] }}
/>

<Story
  name="Campus Selected"
  args={{ campus: ['M', 'D', 'S', 'L', 'I'] }}
/>

<Story
  name="Online Selected"
  args={{ campus: ['W'] }}
/>
