import type { PageLoad } from "./$types";
import { BannerApiClient } from "$lib/api";

export const load: PageLoad = async ({ url, fetch }) => {
  const client = new BannerApiClient(undefined, fetch);
  try {
    const terms = await client.getTerms();
    return { terms, url };
  } catch (e) {
    console.error("Failed to load terms:", e);
    return { terms: [], url };
  }
};
