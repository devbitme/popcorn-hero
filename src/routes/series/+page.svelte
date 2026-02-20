<script lang="ts">
	import { Loader2, Tv } from "@lucide/svelte";
	import { invoke } from "@tauri-apps/api/core";
	import { listen } from "@tauri-apps/api/event";
	import { onMount } from "svelte";
	import { toast } from "svelte-sonner";
	import MediaCard from "$lib/components/app/media-card.svelte";
	import MediaHero from "$lib/components/app/media-hero.svelte";
	import MediaRow from "$lib/components/app/media-row.svelte";
	import {
		getBestRated, 
		getTitle,
		groupByShow,
		isTvShow,
		toAssetUrl
	} from "$lib/helpers/media";
	import { m } from "$lib/paraglide/messages.js";
	import { currentUser } from "$lib/stores/user";
	import type { MediaWithMetadata } from "$lib/types/media";

	async function playMedia(item: MediaWithMetadata) {
		try {
			await invoke("player_open_vlc", { path: item.path });
		} catch (e) {
			toast.error(String(e));
		}
	}

	let allSeries = $state<MediaWithMetadata[]>([]);
	let isLoading = $state(true);
	let heroIndex = $state(0);

	// Derived data
	let heroItems = $derived(getBestRated(allSeries));
	let featuredSeries = $derived(heroItems[heroIndex] ?? null);
	let showGroups = $derived(groupByShow(allSeries));
	let sortedShows = $derived(
		[...showGroups.entries()].sort(([a], [b]) => a.localeCompare(b))
	);

	// Unique shows for "All Shows" grid (one entry per show, best poster)
	let uniqueShows = $derived.by(() => {
		const seen = new Map<string, MediaWithMetadata>();
		for (const [showTitle, episodes] of showGroups) {
			// Pick the episode with the best poster/metadata as representative
			const best =
				episodes.find((e) => e.poster_path) ??
				episodes.find((e) => e.metadata?.rating) ??
				episodes[0];
			if (best) seen.set(showTitle, best);
		}
		return [...seen.values()];
	});

	async function loadSeries() {
		if (!$currentUser) return;
		try {
			const library = await invoke<MediaWithMetadata[]>("get_library_with_metadata", {
				userId: $currentUser.id
			});
			allSeries = library.filter(isTvShow);
		} catch (error) {
			console.error("Failed to load series:", error);
		} finally {
			isLoading = false;
		}
	}

	onMount(() => {
		loadSeries();

		// Rotate hero every 8 seconds
		const heroTimer = setInterval(() => {
			if (heroItems.length > 1) {
				heroIndex = (heroIndex + 1) % heroItems.length;
			}
		}, 8000);

		// Reload on media changes
		const unlisten = listen("media-change", () => {
			loadSeries();
		});

		return () => {
			clearInterval(heroTimer);
			unlisten.then((fn) => fn());
		};
	});

	function formatEpisode(item: MediaWithMetadata): string {
		const s = item.metadata?.season_number;
		const e = item.metadata?.episode_number;
		if (s != null && e != null) return `S${String(s).padStart(2, "0")}E${String(e).padStart(2, "0")}`;
		return "";
	}
</script>

{#if isLoading}
	<div class="flex h-[80vh] items-center justify-center">
		<Loader2 class="h-8 w-8 animate-spin text-muted-foreground" />
	</div>
{:else if allSeries.length === 0}
	<!-- Empty state -->
	<div class="flex h-[80vh] flex-col items-center justify-center gap-4 text-center px-8">
		<div class="rounded-full bg-muted p-6">
			<Tv class="h-12 w-12 text-muted-foreground/50" />
		</div>
		<h2 class="text-2xl font-semibold text-foreground">{m.nav_series()}</h2>
		<p class="text-muted-foreground max-w-md">
			Aucune série trouvée. Ajoutez des dossiers média dans les paramètres puis lancez un scan pour indexer vos séries.
		</p>
	</div>
{:else}
	<div class="pb-8">
		<!-- Hero section -->
		{#if featuredSeries}
			{@const meta = featuredSeries.metadata}
			<MediaHero
				title={getTitle(featuredSeries)}
				overview={meta?.overview}
				year={meta?.year}
				rating={meta?.rating}
				runtime={meta?.runtime_minutes}
				genres={meta?.genres ?? []}
				tagline={meta?.tagline}
				backdropUrl={toAssetUrl(featuredSeries.backdrop_path)}
				posterUrl={toAssetUrl(featuredSeries.poster_path)}
			/>

			{#if heroItems.length > 1}
				<div class="flex justify-center gap-2 -mt-6 relative z-20">
					{#each heroItems as _, i}
						<button
							class="h-1.5 rounded-full transition-all duration-300 cursor-pointer {i === heroIndex
								? 'w-6 bg-primary'
								: 'w-1.5 bg-white/40 hover:bg-white/60'}"
							onclick={() => (heroIndex = i)}
							aria-label="Slide {i + 1}"
						></button>
					{/each}
				</div>
			{/if}
		{/if}

		<!-- All Shows grid -->
		{#if uniqueShows.length > 0}
			<div class="mt-6">
				<MediaRow title="Toutes les séries">
					{#each uniqueShows as item (item.id)}
					<div class="w-35 md:w-40 lg:w-45 shrink-0">
						<MediaCard
							title={getTitle(item)}
							year={item.metadata?.year}
							rating={item.metadata?.rating}
							genres={item.metadata?.genres ?? []}
							posterUrl={toAssetUrl(item.poster_path)}
							overview={item.metadata?.overview}						onclick={() => playMedia(item)}						/>
					</div>
					{/each}
				</MediaRow>
			</div>
		{/if}

		<!-- Per-show episode rows -->
		{#each sortedShows as [showTitle, episodes] (showTitle)}
			{#if episodes.length > 1}
				<MediaRow title="{showTitle} ({episodes.length} épisodes)">
					{#each episodes as ep (ep.id)}
					<div class="w-35 md:w-40 lg:w-45 shrink-0">
						<MediaCard
							title={(ep.metadata?.episode_title ?? formatEpisode(ep)) || getTitle(ep)}
							year={ep.metadata?.year}
							rating={ep.metadata?.rating}
							runtime={ep.metadata?.runtime_minutes}
							genres={[formatEpisode(ep)].filter(Boolean)}
							posterUrl={toAssetUrl(ep.still_path) ?? toAssetUrl(ep.poster_path)}
							overview={ep.metadata?.episode_overview ?? ep.metadata?.overview}						onclick={() => playMedia(ep)}						/>
					</div>
					{/each}
				</MediaRow>
			{/if}
		{/each}
	</div>
{/if}
