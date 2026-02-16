<script lang="ts">
	import { FolderOpen, Loader2 } from "@lucide/svelte";
	import { invoke } from "@tauri-apps/api/core";
	import { listen } from "@tauri-apps/api/event";
	import { onMount } from "svelte";
	import MediaCard from "$lib/components/app/media-card.svelte";
	import { getTitle, isUnidentified, toAssetUrl } from "$lib/helpers/media";
	import { m } from "$lib/paraglide/messages.js";
	import { currentUser } from "$lib/stores/user";
	import type { MediaWithMetadata } from "$lib/types/media";

	let allItems = $state<MediaWithMetadata[]>([]);
	let isLoading = $state(true);

	async function loadLibrary() {
		if (!$currentUser) return;
		try {
			const library = await invoke<MediaWithMetadata[]>("get_library_with_metadata", {
				userId: $currentUser.id
			});
			allItems = library.filter(isUnidentified);
		} catch (error) {
			console.error("Failed to load library:", error);
		} finally {
			isLoading = false;
		}
	}

	onMount(() => {
		loadLibrary();

		const unlisten = listen("media-change", () => {
			loadLibrary();
		});

		return () => {
			unlisten.then((fn) => fn());
		};
	});
</script>

{#if isLoading}
	<div class="flex h-[80vh] items-center justify-center">
		<Loader2 class="h-8 w-8 animate-spin text-muted-foreground" />
	</div>
{:else if allItems.length === 0}
	<div class="flex h-[80vh] flex-col items-center justify-center gap-4 text-center px-8">
		<div class="rounded-full bg-muted p-6">
			<FolderOpen class="h-12 w-12 text-muted-foreground/50" />
		</div>
		<h2 class="text-2xl font-semibold text-foreground">{m.nav_library()}</h2>
		<p class="text-muted-foreground max-w-md">
			Tous vos médias ont été identifiés ! Aucun fichier non reconnu à afficher.
		</p>
	</div>
{:else}
	<div class="p-8 pb-12">
		<div class="mb-6">
			<h1 class="text-2xl font-bold text-foreground">{m.nav_library()}</h1>
			<p class="text-sm text-muted-foreground mt-1">
				{allItems.length} fichier{allItems.length > 1 ? "s" : ""} non identifié{allItems.length > 1 ? "s" : ""}
			</p>
		</div>

		<div class="grid grid-cols-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 xl:grid-cols-8 gap-4">
			{#each allItems as item (item.id)}
				<MediaCard
					title={getTitle(item)}
					year={item.metadata?.year}
					rating={item.metadata?.rating}
					runtime={item.metadata?.runtime_minutes}
					genres={item.metadata?.genres ?? []}
					posterUrl={toAssetUrl(item.poster_path)}
					overview={item.metadata?.overview}
				/>
			{/each}
		</div>
	</div>
{/if}