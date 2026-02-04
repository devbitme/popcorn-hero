<script lang="ts">
  import ClipboardClock from "@lucide/svelte/icons/clipboard-clock";
  import Github from "@lucide/svelte/icons/github";
  import UserCog from "@lucide/svelte/icons/user-cog";
  import { Spring } from "svelte/motion";
  import config from "$/popcorn-hero.config.js";
  import { dev } from "$app/environment";
  import { Button } from "$lib/components/ui/button/index.js";
  import * as ButtonGroup from "$lib/components/ui/button-group/index.js";

  const isEnabled = dev && (config.devToolbar?.enabled ?? true);

  let isPinned = $state(false);
  let isVisible = $state(false);
  let hideTimeout: ReturnType<typeof setTimeout> | null = null;

  const HOVER_DELAY = 2000; // 2 seconds like Astro

  // Spring animation with bounce effect like Astro's cubic-bezier(0.485, -0.050, 0.285, 1.505)
  const position = new Spring(-40, {
    stiffness: 0.08,
    damping: 0.25,
  });

  const opacity = new Spring(0.2, {
    stiffness: 0.1,
    damping: 0.5,
  });

  function handleMouseEnter() {
    if (isPinned) return;
    if (hideTimeout) {
      clearTimeout(hideTimeout);
      hideTimeout = null;
    }
    isVisible = true;
    position.target = 0;
    opacity.target = 1;
  }

  function handleMouseLeave() {
    if (isPinned) return;
    hideTimeout = setTimeout(() => {
      isVisible = false;
      position.target = -40;
      opacity.target = 0.2;
    }, HOVER_DELAY);
  }

  function togglePinned() {
    isPinned = !isPinned;
    if (isPinned) {
      // Pinned: always visible, full opacity
      if (hideTimeout) {
        clearTimeout(hideTimeout);
        hideTimeout = null;
      }
      isVisible = true;
      position.target = 0;
      opacity.target = 1;
    } else {
      // Unpinned: start autohide behavior
      handleMouseLeave();
    }
  }
</script>

{#if isEnabled}
  <!-- Hitbox area to detect hover -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="fixed bottom-0 left-1/2 -translate-x-1/2 z-999999 flex flex-col items-center cursor-pointer"
    role="toolbar"
    aria-label="Development Toolbar"
  >
    <!-- Hitbox above the bar -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="w-full pointer-events-auto {isVisible ? 'h-0' : 'h-10.5'}"
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
        <!-- OK Item -->
        <div
          class="flex justify-center items-center h-full"
          style:opacity={opacity.current}
        >
          <ButtonGroup.Root>
            <Button class="cursor-pointer rounded-l-full px-2 group" onclick={togglePinned}>
              <img
                src="./logo-dev-toolbar.svg"
                class="size-6 transition-opacity duration-200 {isPinned ? 'opacity-100' : 'opacity-80 group-hover:opacity-100'}"
                alt="logo dev toolbar"
              />
            </Button>
            <Button
              class="cursor-pointer transition-colors px-2 duration-200 hover:bg-white/20"
            >
              <UserCog class="size-6" strokeWidth={1.5} />
            </Button>
            <Button
              class="cursor-pointer transition-colors px-2 duration-200 hover:bg-white/20"
            >
              <ClipboardClock class="size-6" strokeWidth={1.5} />
            </Button>
            <Button
              class="cursor-pointer transition-colors px-2 duration-200 hover:bg-white/20 rounded-r-full"
            >
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
