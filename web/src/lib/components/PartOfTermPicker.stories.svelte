<script module>
import { defineMeta } from "@storybook/addon-svelte-csf";
import PartOfTermPicker from "./PartOfTermPicker.svelte";

const mockPartsOfTerm = [
  { filterValue: "1", description: "Full Term" },
  { filterValue: "8W1", description: "First 8 Week" },
  { filterValue: "8W2", description: "Second 8 Week" },
  { filterValue: "SUM1", description: "Summer 1" },
  { filterValue: "SUM2", description: "Summer 2" },
];

const { Story } = defineMeta({
  title: "Components/PartOfTermPicker",
  component: PartOfTermPicker,
  tags: ["autodocs"],
});
</script>

<Story
  name="Default"
  args={{ partOfTerm: [], partsOfTerm: mockPartsOfTerm }}
/>

<Story
  name="Some Selected"
  args={{ partOfTerm: ['1', '8W1'], partsOfTerm: mockPartsOfTerm }}
/>

<Story
  name="Mobile Layout"
  args={{ partOfTerm: [], partsOfTerm: mockPartsOfTerm, mobile: true }}
/>

<Story
  name="Empty Options"
  args={{ partOfTerm: [], partsOfTerm: [] }}
/>
