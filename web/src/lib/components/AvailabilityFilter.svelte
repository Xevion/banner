<script lang="ts">
import { CampusValues } from "$lib/filterValues";

let {
  campus = $bindable<string[]>([]),
}: {
  campus: string[];
} = $props();

// Campus filter values by availability category
const CAMPUS_STUDENTS: string[] = [
  CampusValues.Main,
  CampusValues.Downtown,
  CampusValues.Southwest,
  CampusValues.Laredo,
  CampusValues.Internet,
];
const ONLINE_PROGRAMS: string[] = [CampusValues.OnlinePrograms];

// Determine which availability option is effectively selected based on campus filter
// This is a UI convenience - internally we still use campus codes
const availabilitySelection = $derived.by(() => {
  if (campus.length === 0) return "all";

  const hasCampusStudent = campus.some((c) => CAMPUS_STUDENTS.includes(c));
  const hasOnlinePrograms = campus.some((c) => ONLINE_PROGRAMS.includes(c));

  if (hasCampusStudent && hasOnlinePrograms) return "all";
  if (hasCampusStudent) return "campus";
  if (hasOnlinePrograms) return "online";
  return "all";
});

function selectAvailability(option: "campus" | "online" | "all") {
  if (option === "campus") {
    // Set campus filter to all campus-student-accessible campuses
    campus = [...CAMPUS_STUDENTS];
  } else if (option === "online") {
    // Set campus filter to online programs only
    campus = [...ONLINE_PROGRAMS];
  } else {
    // Clear campus filter to show all
    campus = [];
  }
}

function toggleAvailability(option: "campus" | "online") {
  if (availabilitySelection === option) {
    // Already selected, clear
    selectAvailability("all");
  } else {
    selectAvailability(option);
  }
}
</script>

<div class="flex flex-col gap-2">
  <span class="text-xs font-medium text-muted-foreground select-none">Availability</span>

  <div class="flex flex-wrap gap-1">
    <button
      type="button"
      aria-pressed={availabilitySelection === "campus"}
      class="inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium transition-colors cursor-pointer select-none
             {availabilitySelection === 'campus'
        ? 'bg-primary text-primary-foreground'
        : 'bg-muted text-muted-foreground hover:bg-muted/80'}"
      onclick={() => toggleAvailability("campus")}
      title="Courses available to traditional campus students (Main, Downtown, Southwest, Laredo, and Internet campuses)"
    >
      Campus Students
    </button>

    <button
      type="button"
      aria-pressed={availabilitySelection === "online"}
      class="inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium transition-colors cursor-pointer select-none
             {availabilitySelection === 'online'
        ? 'bg-primary text-primary-foreground'
        : 'bg-muted text-muted-foreground hover:bg-muted/80'}"
      onclick={() => toggleAvailability("online")}
      title="Courses restricted to online degree program students only"
    >
      Online Programs
    </button>
  </div>

  {#if availabilitySelection !== "all"}
    <p class="text-xs text-muted-foreground/70 italic">
      {#if availabilitySelection === "campus"}
        Showing courses available to campus students
      {:else}
        Showing courses for online degree programs only
      {/if}
    </p>
  {/if}
</div>
