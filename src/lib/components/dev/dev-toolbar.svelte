<script lang="ts">
  import ClipboardClock from "@lucide/svelte/icons/clipboard-clock";
  import Github from "@lucide/svelte/icons/github";
  import UserCog from "@lucide/svelte/icons/user-cog";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { Spring } from "svelte/motion";
  import config from "$/popcorn-hero.config.js";
  import { browser, dev } from "$app/environment";
  import { Button } from "$lib/components/ui/button/index.js";
  import * as ButtonGroup from "$lib/components/ui/button-group/index.js";

  const isEnabled = dev && (config.devToolbar?.enabled ?? true);
  const hoverDelay = config.devToolbar?.hoverDelay ?? 1500;
  const storageKey = "devToolbar:pinned";

  // Initialize from localStorage
  const initialPinned = browser && localStorage.getItem(storageKey) === "true";

  // State
  let isPinned = $state(initialPinned);
  let hideTimeout = $state<ReturnType<typeof setTimeout> | null>(null);

  // Spring animations
  const position = new Spring(initialPinned ? 0 : -40, {
    stiffness: 0.08,
    damping: 0.25,
  });

  const opacity = new Spring(initialPinned ? 1 : 0.2, {
    stiffness: 0.1,
    damping: 0.5,
  });

  // Derived state: toolbar is hidden when position is below threshold
  const isHidden = $derived(position.current < -20);

  // Persist isPinned to localStorage
  $effect(() => {
    if (browser) {
      localStorage.setItem(storageKey, String(isPinned));
    }
  });

  // Cleanup timeout on component destroy
  $effect(() => {
    return () => {
      if (hideTimeout) clearTimeout(hideTimeout);
    };
  });

  function clearHideTimeout() {
    if (hideTimeout) {
      clearTimeout(hideTimeout);
      hideTimeout = null;
    }
  }

  function show() {
    position.target = 0;
    opacity.target = 1;
  }

  function hide() {
    position.target = -40;
    opacity.target = 0.2;
  }

  function handleMouseEnter() {
    clearHideTimeout();
    show();
  }

  function handleMouseLeave() {
    if (isPinned) return;
    hideTimeout = setTimeout(hide, hoverDelay);
  }

  function togglePinned() {
    isPinned = !isPinned;
    if (isPinned) {
      clearHideTimeout();
      show();
    } else {
      handleMouseLeave();
    }
  }

  // Common button styles
  const iconButtonClass = "cursor-pointer transition-colors px-2 duration-200 hover:bg-white/20";
</script>

{#if isEnabled}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="fixed bottom-0 left-1/2 -translate-x-1/2 z-999999 flex flex-col items-center cursor-pointer"
    role="toolbar"
    aria-label="Development Toolbar"
  >
    <!-- Hitbox above the bar -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="w-full pointer-events-auto {isHidden ? 'h-10.5' : 'h-0'}"
      onmouseenter={handleMouseEnter}
    ></div>

    <!-- Dev toolbar bar -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="overflow-hidden pointer-events-auto rounded-full border border-primary bg-primary shadow-lg shadow-primary/50"
      style:margin-bottom="{position.current}px"
      onmouseenter={handleMouseEnter}
      onmouseleave={handleMouseLeave}
    >
      <div class="h-full flex items-center">
        <div
          class="flex justify-center items-center h-full"
          style:opacity={opacity.current}
        >
          <ButtonGroup.Root>
            <Button class="cursor-pointer rounded-l-full px-2 group" onclick={togglePinned}>
              <img
                src="./logo-dev-toolbar.svg"
                class="size-6 transition-opacity duration-200 {isPinned ? 'opacity-100' : 'opacity-60 group-hover:opacity-100'}"
                alt="logo dev toolbar"
              />
            </Button>
            <Button class={iconButtonClass}>
              <UserCog class="size-6" strokeWidth={1.5} />
            </Button>
            <Button class={iconButtonClass}>
              <ClipboardClock class="size-6" strokeWidth={1.5} />
            </Button>
            <Button class="{iconButtonClass} rounded-r-full" onclick={() => openUrl("https://github.com/devbitme/popcorn-hero")}>
              <Github class="size-6" strokeWidth={1.5} />
            </Button>
          </ButtonGroup.Root>
        </div>
      </div>
    </div>

    <!-- Hitbox below the bar -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="w-full pointer-events-auto h-4"
      onmouseenter={handleMouseEnter}
    ></div>
  </div>
{/if}
