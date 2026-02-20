<script lang="ts">
	import { Loader2, Popcorn } from "@lucide/svelte";
	import { invoke } from "@tauri-apps/api/core";
	import { listen } from "@tauri-apps/api/event";
	import { onMount } from "svelte";
	import { toast } from "svelte-sonner";
	import MediaCard from "$lib/components/app/media-card.svelte";
	import MediaHero from "$lib/components/app/media-hero.svelte";
	import MediaRow from "$lib/components/app/media-row.svelte";
	import {
		getBestRated,
		getRecentlyAdded, 
		getTitle,
		groupByGenre,
		isMovie,
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

	let allMovies = $state<MediaWithMetadata[]>([]);
	let isLoading = $state(true);
	let heroIndex = $state(0);

	// Derived data
	let heroItems = $derived(getBestRated(allMovies));
	let featuredMovie = $derived(heroItems[heroIndex] ?? null);
	let recentlyAdded = $derived(getRecentlyAdded(allMovies));
	let genreGroups = $derived(groupByGenre(allMovies));
	let sortedGenres = $derived(
		[...genreGroups.entries()].sort(([a], [b]) => {
			if (a === "Other") return 1;
			if (b === "Other") return -1;
			return a.localeCompare(b);
		})
	);

	async function loadMovies() {
		if (!$currentUser) return;
		try {
			const library = await invoke<MediaWithMetadata[]>("get_library_with_metadata", {
				userId: $currentUser.id
			});
			allMovies = library.filter(isMovie);
		} catch (error) {
			console.error("Failed to load movies:", error);
		} finally {
			isLoading = false;
		}
	}

	onMount(() => {
		loadMovies();

		// Rotate hero every 8 seconds
		const heroTimer = setInterval(() => {
			if (heroItems.length > 1) {
				heroIndex = (heroIndex + 1) % heroItems.length;
			}
		}, 8000);

		// Reload on media changes
		const unlisten = listen("media-change", () => {
			loadMovies();
		});

		return () => {
			clearInterval(heroTimer);
			unlisten.then((fn) => fn());
		};
	});
</script>

{#if isLoading}
	<div class="flex h-[80vh] items-center justify-center">
		<Loader2 class="h-8 w-8 animate-spin text-muted-foreground" />
	</div>
{:else if allMovies.length === 0}
	<!-- Empty state -->
	<div class="flex h-[80vh] flex-col items-center justify-center gap-4 text-center px-8">
		<div class="rounded-full bg-muted p-6">
			<Popcorn class="h-12 w-12 text-muted-foreground/50" />
		</div>
		<h2 class="text-2xl font-semibold text-foreground">{m.nav_movies()}</h2>
		<p class="text-muted-foreground max-w-md">
			Aucun film trouvé. Ajoutez des dossiers média dans les paramètres puis lancez un scan pour indexer vos films.
		</p>
	</div>
{:else}
	<div class="pb-8">
		<!-- Hero section -->
		{#if featuredMovie}
			{@const meta = featuredMovie.metadata}
			<MediaHero
				title={getTitle(featuredMovie)}
				overview={meta?.overview}
				year={meta?.year}
				rating={meta?.rating}
				runtime={meta?.runtime_minutes}
				genres={meta?.genres ?? []}
				tagline={meta?.tagline}
				backdropUrl={toAssetUrl(featuredMovie.backdrop_path)}
				posterUrl={toAssetUrl(featuredMovie.poster_path)}
			/>

			<!-- Hero dots navigation -->
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

		<!-- Recently Added -->
		{#if recentlyAdded.length > 0}
			<div class="mt-6">
				<MediaRow title="Ajoutés récemment">
					{#each recentlyAdded as item (item.id)}
					<div class="w-35 md:w-40 lg:w-45 shrink-0">
						<MediaCard
							title={getTitle(item)}
							year={item.metadata?.year}
							rating={item.metadata?.rating}
							runtime={item.metadata?.runtime_minutes}
							genres={item.metadata?.genres ?? []}
							posterUrl={toAssetUrl(item.poster_path)}
							overview={item.metadata?.overview}						onclick={() => playMedia(item)}						/>
					</div>
				{/each}
			</MediaRow>
		</div>
	{/if}

		<!-- Genre rows -->
		{#each sortedGenres as [genre, movies] (genre)}
			{#if movies.length > 0}
				<MediaRow title={genre}>
					{#each movies as item (item.id)}
						<div class="w-35 md:w-40 lg:w-45 shrink-0">
							<MediaCard
								title={getTitle(item)}
								year={item.metadata?.year}
								rating={item.metadata?.rating}
								runtime={item.metadata?.runtime_minutes}
								genres={item.metadata?.genres ?? []}
								posterUrl={toAssetUrl(item.poster_path)}
								overview={item.metadata?.overview}
								onclick={() => playMedia(item)}
							/>
						</div>
					{/each}
				</MediaRow>
			{/if}
		{/each}
	</div>
{/if}
