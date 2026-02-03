<script module>
import { defineMeta } from "@storybook/addon-svelte-csf";
import RangeSlider from "./RangeSlider.svelte";

const { Story } = defineMeta({
  title: "Components/RangeSlider",
  component: RangeSlider,
  tags: ["autodocs"],
  parameters: {
    layout: "padded",
  },
  argTypes: {
    min: { control: "number" },
    max: { control: "number" },
    label: { control: "text" },
    dual: { control: "boolean" },
    pips: { control: "boolean" },
    pipstep: { control: "number" },
    float: { control: "boolean" },
    hoverable: { control: "boolean" },
  },
});
</script>

<!-- Default Dual: Two-thumb range slider -->
<Story
  name="Default Dual"
  args={{
    min: 0,
    max: 100,
    label: 'Credit Hours',
  }}
/>

<!-- Single Thumb: Single thumb slider -->
<Story
  name="Single Thumb"
  args={{
    min: 0,
    max: 100,
    label: 'Maximum Credits',
    dual: false,
  }}
/>

<!-- With Pips: Show pip markers -->
<Story
  name="With Pips"
  args={{
    min: 0,
    max: 100,
    label: 'Credit Hours',
    pips: true,
    pipstep: 25,
  }}
/>

<!-- Custom Format: Format values as currency -->
<Story
  name="Custom Format"
  args={{
    min: 0,
    max: 1000,
    label: "Budget Range",
    // @ts-expect-error - formatValue type inference issue
    formatValue: (v) => `$${v}`,
  }}
/>

<!-- GPA Range: Example with decimal formatting -->
<Story
  name="GPA Range"
  args={{
    min: 0,
    max: 4,
    label: "GPA Range",
    step: 0.1,
    // @ts-expect-error - formatValue type inference issue
    formatValue: (v) => v.toFixed(1),
  }}
/>
