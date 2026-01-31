import { BannerApiClient } from "$lib/api";
import type { PageLoad } from "./$types";

export const load: PageLoad = async ({ url, fetch }) => {
  const client = new BannerApiClient(undefined, fetch);
  try {
    const urlTerm = url.searchParams.get("term");
    // Backend defaults to latest term if not specified
    const searchOptions = await client.getSearchOptions(urlTerm ?? undefined);
    return { searchOptions, url };
  } catch (e) {
    console.error("Failed to load search options:", e);
    return { searchOptions: null, url };
  }
};
