import posthog from "posthog-js";
import { browser } from "$app/environment";

export const load = async () => {
  if (browser) {
    posthog.init("phc_MMjGWRVYMiWQmyv023W0JCybGThryFfGUwS5nF2aXnt", {
      api_host: "https://us.i.posthog.com",
      person_profiles: "identified_only",
    });
  }
  return;
};
