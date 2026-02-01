<script lang="ts">
import type { CourseResponse } from "$lib/api";
import CourseDetailTabs from "./course-detail/CourseDetailTabs.svelte";

let { course }: { course: CourseResponse } = $props();
</script>

<div class="bg-muted/40 border-b border-border">
  <CourseDetailTabs {course} />
</div>
