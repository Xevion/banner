<script module>
import { defineMeta } from "@storybook/addon-svelte-csf";
import { fn } from "storybook/test";
import Pagination from "./Pagination.svelte";

const { Story } = defineMeta({
  title: "Components/Pagination",
  component: Pagination,
  tags: ["autodocs"],
});
</script>

<Story
  name="First Page"
  args={{ totalCount: 100, offset: 0, limit: 25, onPageChange: fn() }}
/>

<Story
  name="Middle Page"
  args={{ totalCount: 100, offset: 50, limit: 25, onPageChange: fn() }}
/>

<Story
  name="Last Page"
  args={{ totalCount: 100, offset: 75, limit: 25, onPageChange: fn() }}
/>

<Story
  name="Many Pages"
  args={{ totalCount: 1000, offset: 250, limit: 25, onPageChange: fn() }}
/>

<Story
  name="Single Page"
  args={{ totalCount: 20, offset: 0, limit: 25, onPageChange: fn() }}
/>

<Story
  name="Loading"
  args={{ totalCount: 100, offset: 50, limit: 25, loading: true, onPageChange: fn() }}
/>

<Story
  name="Empty"
  args={{ totalCount: 0, offset: 0, limit: 25, onPageChange: fn() }}
/>
