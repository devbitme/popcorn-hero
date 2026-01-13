<script lang="ts">
  import Plus from "@lucide/svelte/icons/plus";
  import { Spring } from "svelte/motion";
  import config from "$/popcorn-hero.config.js";
  import { dev } from "$app/environment";
  import { Button } from "$lib/components/ui/button/index.js";
  import * as ButtonGroup from "$lib/components/ui/button-group/index.js";

  const isEnabled = dev && (config.devToolbar?.enabled ?? true);
  const autoHide = config.devToolbar?.autoHide ?? true;

  let isVisible = $state(!autoHide);
  let hideTimeout: ReturnType<typeof setTimeout> | null = null;

  const HOVER_DELAY = 2000; // 2 seconds like Astro

  // Spring animation with bounce effect like Astro's cubic-bezier(0.485, -0.050, 0.285, 1.505)
  const position = new Spring(autoHide ? -40 : 0, {
    stiffness: 0.08,
    damping: 0.25,
  });

  const opacity = new Spring(autoHide ? 0.2 : 1, {
    stiffness: 0.1,
    damping: 0.5,
  });

  function handleMouseEnter() {
    if (!autoHide) return;
    if (hideTimeout) {
      clearTimeout(hideTimeout);
      hideTimeout = null;
    }
    isVisible = true;
    position.target = 0;
    opacity.target = 1;
  }

  function handleMouseLeave() {
    if (!autoHide) return;
    hideTimeout = setTimeout(() => {
      isVisible = false;
      position.target = -40;
      opacity.target = 0.2;
    }, HOVER_DELAY);
  }
</script>

{#if isEnabled}
  <!-- Hitbox area to detect hover -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="fixed bottom-0 left-1/2 -translate-x-1/2 z-2000000010 flex flex-col items-center cursor-pointer"
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
      class="h-10 overflow-hidden pointer-events-auto rounded-full border border-[#343841] bg-linear-to-b from-[#13151A] to-[#13151ae0]"
      style:margin-bottom="{position.current}px"
      onmouseenter={handleMouseEnter}
      onmouseleave={handleMouseLeave}
    >
      <div class="h-full flex items-center">
        <!-- OK Item -->
        <div
          class="flex justify-center items-center h-full text-sm font-medium text-[#9198AD]"
          style:opacity={opacity.current}
        >
          <ButtonGroup.Root>
            <Button
              class="cursor-pointer transition-colors duration-200 hover:bg-white/10 hover:text-white rounded-l-full pl-4"
            >
              A
            </Button>
            <ButtonGroup.Separator class="bg-white/10" />
            <Button
              class="cursor-pointer transition-colors duration-200 hover:bg-white/10 hover:text-white"
            >
              B
            </Button>
            <ButtonGroup.Separator class="bg-white/10" />
            <Button
              class="cursor-pointer transition-colors duration-200 hover:bg-white/10 hover:text-white rounded-r-full pr-4"
            >
              C
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
