<script lang="ts">
	import { ChevronLeft, ChevronRight } from "@lucide/svelte";
	import type { Snippet } from "svelte";

	interface Props {
		title: string;
		children: Snippet;
	}

	let { title, children }: Props = $props();

	let scrollContainer: HTMLDivElement | undefined = $state();
	let canScrollLeft = $state(false);
	let canScrollRight = $state(true);

	function updateScrollState() {
		if (!scrollContainer) return;
		canScrollLeft = scrollContainer.scrollLeft > 10;
		canScrollRight =
			scrollContainer.scrollLeft < scrollContainer.scrollWidth - scrollContainer.clientWidth - 10;
	}

	function scroll(direction: "left" | "right") {
		if (!scrollContainer) return;
		const cardWidth = scrollContainer.clientWidth * 0.8;
		scrollContainer.scrollBy({
			left: direction === "left" ? -cardWidth : cardWidth,
			behavior: "smooth"
		});
	}
</script>

<section class="relative py-4">
	<!-- Section title -->
	<h2 class="text-xl font-semibold text-foreground mb-3 px-8">{title}</h2>

	<!-- Scroll container -->
	<div class="group relative">
		<!-- Left arrow -->
		{#if canScrollLeft}
			<button
				class="absolute left-0 top-0 bottom-0 z-20 w-12 flex items-center justify-center bg-linear-to-r from-background to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-200 cursor-pointer"
				onclick={() => scroll("left")}
			>
				<ChevronLeft class="h-8 w-8 text-foreground" />
			</button>
		{/if}

		<!-- Right arrow -->
		{#if canScrollRight}
			<button
				class="absolute right-0 top-0 bottom-0 z-20 w-12 flex items-center justify-center bg-linear-to-l from-background to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-200 cursor-pointer"
				onclick={() => scroll("right")}
			>
				<ChevronRight class="h-8 w-8 text-foreground" />
			</button>
		{/if}

		<!-- Scrollable list -->
		<div
			bind:this={scrollContainer}
			class="flex gap-4 overflow-x-auto px-8 pb-4 scrollbar-hide scroll-smooth"
			onscroll={updateScrollState}
		>
			{@render children()}
		</div>
	</div>
</section>

<style>
	.scrollbar-hide {
		-ms-overflow-style: none;
		scrollbar-width: none;
	}
	.scrollbar-hide::-webkit-scrollbar {
		display: none;
	}
</style>
