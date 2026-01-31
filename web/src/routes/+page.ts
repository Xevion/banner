import { BannerApiClient } from "$lib/api";
import type { PageLoad } from "./$types";

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
