<script lang="ts">
  import * as Icon from "svelte-flag-icons";
  import { buttonVariants } from "$lib/components/ui/button/index.js";
  import * as Popover from "$lib/components/ui/popover/index.js";
  import { getLocale, locales, setLocale } from "$lib/paraglide/runtime.js";

  // Exceptions for locale to flag icon mapping (most use capitalized locale code)
  const flagExceptions: Record<string, string> = {
    en: "Gb",
    ja: "Jp",
  };

  // Get flag component for a given locale
  function getFlagComponent(locale: string) {
    const flagCode =
      flagExceptions[locale] ||
      locale.charAt(0).toUpperCase() + locale.slice(1);
    return (Icon as any)[flagCode];
  }
</script>

<div class="absolute top-2 right-2">
  <!-- Language Switcher Component - allows users to switch between available locales -->
  <Popover.Root>
    <Popover.Trigger
      class={`bg-iroh-300! hover:bg-iroh-200! transition-colors p-2! cursor-pointer! size-10 ${buttonVariants({ variant: "outline" })}`}
      ><svelte:component
        this={getFlagComponent(getLocale())}
        size="24"
      /></Popover.Trigger
    >
    <Popover.Content class="w-auto p-1 bg-carbon-950">
      <ul>
        {#each locales.filter((locale) => locale !== getLocale()) as locale}
          <li>
            <button
              class="w-full text-left rounded hover:bg-gray-200 cursor-pointer px-2 py-1"
              on:click={() => {
                setLocale(locale, { reload: true });
              }}
            >
              <svelte:component this={getFlagComponent(locale)} size="18" />
            </button>
          </li>
        {/each}
      </ul>
    </Popover.Content>
  </Popover.Root>
</div>
