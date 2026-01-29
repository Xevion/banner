import type { PageLoad } from "./$types";
import { BannerApiClient } from "$lib/api";

export const load: PageLoad = async ({ url, fetch }) => {
  const client = new BannerApiClient(undefined, fetch);
  const terms = await client.getTerms();
  return { terms, url };
};
