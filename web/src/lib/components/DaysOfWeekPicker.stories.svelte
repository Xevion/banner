<script module>
import { defineMeta } from "@storybook/addon-svelte-csf";
import DaysOfWeekPicker from "./DaysOfWeekPicker.svelte";

const { Story } = defineMeta({
  title: "Components/DaysOfWeekPicker",
  component: DaysOfWeekPicker,
  tags: ["autodocs"],
});
</script>

<Story
  name="Default"
  args={{ days: [] }}
/>

<Story
  name="Some Selected"
  args={{ days: ['monday', 'wednesday', 'friday'] }}
/>

<Story
  name="All Selected"
  args={{ days: ['monday', 'tuesday', 'wednesday', 'thursday', 'friday'] }}
/>

<Story
  name="Mobile Layout"
  args={{ days: ['monday', 'wednesday'], mobile: true }}
/>
