<script lang="ts">
	import { Clock, Film, Star } from "@lucide/svelte";
	import { Badge } from "$lib/components/ui/badge";

	interface Props {
		title: string;
		year?: number | null;
		rating?: number | null;
		runtime?: number | null;
		genres?: string[];
		posterUrl?: string | null;
		overview?: string | null;
		onclick?: () => void;
	}

	let {
		title,
		year = null,
		rating = null,
		runtime = null,
		genres = [],
		posterUrl = null,
		overview = null,
		onclick
	}: Props = $props();

	let isHovered = $state(false);
	let imageLoaded = $state(false);
	let imageError = $state(false);

	function formatRuntime(minutes: number): string {
		const h = Math.floor(minutes / 60);
		const m = minutes % 60;
		return h > 0 ? `${h}h ${m}m` : `${m}m`;
	}
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="media-card group relative cursor-pointer"
	onmouseenter={() => (isHovered = true)}
	onmouseleave={() => (isHovered = false)}
	{onclick}
>
	<!-- Poster container -->
	<div class="relative aspect-2/3 w-full overflow-hidden rounded-lg bg-muted shadow-md transition-all duration-300 group-hover:shadow-xl group-hover:scale-105 group-hover:z-10">
		{#if posterUrl && !imageError}
			<img
				src={posterUrl}
				alt={title}
				class="h-full w-full object-cover transition-all duration-300"
				class:opacity-0={!imageLoaded}
				class:opacity-100={imageLoaded}
				onload={() => (imageLoaded = true)}
				onerror={() => (imageError = true)}
				loading="lazy"
			/>
		{/if}

		<!-- Fallback when no poster -->
		{#if !posterUrl || imageError}
			<div class="flex h-full w-full flex-col items-center justify-center gap-2 bg-muted p-4 text-center">
				<Film class="h-10 w-10 text-muted-foreground/40" />
				<span class="text-xs font-medium text-muted-foreground/60 line-clamp-3">{title}</span>
				{#if year}
					<span class="text-xs text-muted-foreground/40">{year}</span>
				{/if}
			</div>
		{/if}

		<!-- Hover overlay -->
		<div
			class="absolute inset-0 flex flex-col justify-end bg-linear-to-t from-black/90 via-black/40 to-transparent p-3 transition-opacity duration-300"
			class:opacity-0={!isHovered}
			class:opacity-100={isHovered}
		>
			<!-- Rating & runtime -->
			<div class="mb-1.5 flex items-center gap-2">
				{#if rating}
					<div class="flex items-center gap-1">
						<Star class="h-3.5 w-3.5 fill-yellow-400 text-yellow-400" />
						<span class="text-xs font-semibold text-white">{rating.toFixed(1)}</span>
					</div>
				{/if}
				{#if runtime}
					<div class="flex items-center gap-1">
						<Clock class="h-3 w-3 text-white/70" />
						<span class="text-xs text-white/70">{formatRuntime(runtime)}</span>
					</div>
				{/if}
			</div>

			<!-- Genres -->
			{#if genres.length > 0}
				<div class="flex flex-wrap gap-1">
					{#each genres.slice(0, 2) as genre}
						<Badge variant="secondary" class="bg-white/20 text-white text-[10px] px-1.5 py-0 border-0 backdrop-blur-sm">
							{genre}
						</Badge>
					{/each}
				</div>
			{/if}
		</div>
	</div>

	<!-- Title & year below poster -->
	<div class="mt-2 px-0.5">
		<h3 class="text-sm font-medium text-foreground line-clamp-1">{title}</h3>
		{#if year}
			<p class="text-xs text-muted-foreground">{year}</p>
		{/if}
	</div>
</div>
