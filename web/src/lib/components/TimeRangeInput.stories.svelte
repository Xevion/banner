<script module>
import { defineMeta } from "@storybook/addon-svelte-csf";
import TimeRangeInput from "./TimeRangeInput.svelte";

const { Story } = defineMeta({
  title: "Components/TimeRangeInput",
  component: TimeRangeInput,
  tags: ["autodocs"],
});
</script>

<Story name="Default" args={{ timeStart: null, timeEnd: null }} />

<Story name="With Start Time" args={{ timeStart: '09:00', timeEnd: null }} />

<Story name="With End Time" args={{ timeStart: null, timeEnd: '17:00' }} />

<Story name="Full Range" args={{ timeStart: '08:00', timeEnd: '14:00' }} />
