<script lang="ts">
	import { Clock, Info, Star } from "@lucide/svelte";
	import { Badge } from "$lib/components/ui/badge";
	import { Button } from "$lib/components/ui/button";

	interface Props {
		title: string;
		overview?: string | null;
		year?: number | null;
		rating?: number | null;
		runtime?: number | null;
		genres?: string[];
		backdropUrl?: string | null;
		posterUrl?: string | null;
		tagline?: string | null;
	}

	let {
		title,
		overview = null,
		year = null,
		rating = null,
		runtime = null,
		genres = [],
		backdropUrl = null,
		posterUrl = null,
		tagline = null
	}: Props = $props();

	let imageLoaded = $state(false);

	function formatRuntime(minutes: number): string {
		const h = Math.floor(minutes / 60);
		const m = minutes % 60;
		return h > 0 ? `${h}h ${m}m` : `${m}m`;
	}
</script>

<div class="relative w-full h-[50vh] min-h-80 overflow-hidden">
	<!-- Backdrop image -->
	{#if backdropUrl}
		<img
			src={backdropUrl}
			alt={title}
			class="absolute inset-0 h-full w-full object-cover transition-opacity duration-700"
			class:opacity-0={!imageLoaded}
			class:opacity-100={imageLoaded}
			onload={() => (imageLoaded = true)}
		/>
	{/if}

	<!-- Gradient overlays -->
	<div class="absolute inset-0 bg-linear-to-r from-background via-background/70 to-transparent"></div>
	<div class="absolute inset-0 bg-linear-to-t from-background via-transparent to-background/30"></div>

	<!-- Content -->
	<div class="relative z-10 flex h-full items-end pb-12 px-8">
		<div class="flex gap-6 max-w-3xl">
			<!-- Mini poster -->
			{#if posterUrl}
				<div class="hidden md:block shrink-0 w-36 rounded-lg overflow-hidden shadow-2xl">
					<img src={posterUrl} alt={title} class="w-full h-auto object-cover" />
				</div>
			{/if}

			<!-- Info -->
			<div class="flex flex-col justify-end gap-3">
				<h1 class="text-3xl md:text-4xl font-bold text-foreground drop-shadow-lg">{title}</h1>

				{#if tagline}
					<p class="text-sm italic text-muted-foreground">{tagline}</p>
				{/if}

				<!-- Meta row -->
				<div class="flex items-center gap-3 flex-wrap">
					{#if year}
						<span class="text-sm font-medium text-foreground/90">{year}</span>
					{/if}
					{#if rating}
						<div class="flex items-center gap-1">
							<Star class="h-4 w-4 fill-yellow-400 text-yellow-400" />
							<span class="text-sm font-semibold text-foreground">{rating.toFixed(1)}</span>
						</div>
					{/if}
					{#if runtime}
						<div class="flex items-center gap-1">
							<Clock class="h-3.5 w-3.5 text-muted-foreground" />
							<span class="text-sm text-muted-foreground">{formatRuntime(runtime)}</span>
						</div>
					{/if}
				</div>

				<!-- Genres -->
				{#if genres.length > 0}
					<div class="flex flex-wrap gap-1.5">
						{#each genres.slice(0, 4) as genre}
							<Badge variant="outline" class="border-foreground/30 text-foreground/90 text-xs backdrop-blur-sm">
								{genre}
							</Badge>
						{/each}
					</div>
				{/if}

				<!-- Overview -->
				{#if overview}
					<p class="text-sm text-foreground/80 line-clamp-3 max-w-xl leading-relaxed">{overview}</p>
				{/if}
			</div>
		</div>
	</div>
</div>
