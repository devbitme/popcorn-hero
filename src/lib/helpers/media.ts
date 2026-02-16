import { convertFileSrc } from "@tauri-apps/api/core";
import type { MediaWithMetadata } from "$lib/types/media";

/** Convert a local file path to a WebView-loadable URL */
export function toAssetUrl(filePath: string | null | undefined): string | null {
	if (!filePath) return null;
	return convertFileSrc(filePath);
}

/** Check if a media entry is a TV series based on metadata media_type */
export function isTvShow(item: MediaWithMetadata): boolean {
	if (item.metadata?.media_type === "tv") return true;
	if (item.metadata?.media_type === "movie") return false;
	// Fallback for old metadata without media_type field
	if (item.metadata?.season_number != null || item.metadata?.episode_number != null) return true;
	// Fallback: check filename for TV patterns (S01E02, etc.)
	return /[Ss]\d{1,2}[Ee]\d{1,3}/.test(item.filename);
}

/** Check if a media entry is a movie based on metadata media_type */
export function isMovie(item: MediaWithMetadata): boolean {
	if (item.metadata?.media_type === "movie") return true;
	if (item.metadata?.media_type === "tv") return false;
	// Fallback: if metadata was fetched from an API provider (not "local") and it's not a TV show, treat as movie
	if (item.metadata && item.metadata.provider !== "local" && !isTvShow(item)) return true;
	return false;
}

/** Check if a media entry is unidentified (no API metadata, local-only, or unknown type) */
export function isUnidentified(item: MediaWithMetadata): boolean {
	return !isMovie(item) && !isTvShow(item);
}

/** Get display title for a media entry */
export function getTitle(item: MediaWithMetadata): string {
	return item.metadata?.title ?? item.filename.replace(/\.[^.]+$/, "");
}

/** Group media items by genre, returning sorted genre groups */
export function groupByGenre(items: MediaWithMetadata[]): Map<string, MediaWithMetadata[]> {
	const genreMap = new Map<string, MediaWithMetadata[]>();

	for (const item of items) {
		const genres = item.metadata?.genres ?? [];
		if (genres.length === 0) {
			const list = genreMap.get("Other") ?? [];
			list.push(item);
			genreMap.set("Other", list);
		} else {
			for (const genre of genres) {
				const list = genreMap.get(genre) ?? [];
				list.push(item);
				genreMap.set(genre, list);
			}
		}
	}

	return genreMap;
}

/** Group TV episodes by show title, returning map of show → episodes */
export function groupByShow(items: MediaWithMetadata[]): Map<string, MediaWithMetadata[]> {
	const showMap = new Map<string, MediaWithMetadata[]>();

	for (const item of items) {
		const showTitle = item.metadata?.title ?? item.filename.replace(/[Ss]\d{1,2}[Ee]\d{1,3}.*$/, "").replace(/[._-]+/g, " ").trim();
		const list = showMap.get(showTitle) ?? [];
		list.push(item);
		showMap.set(showTitle, list);
	}

	// Sort episodes within each show
	for (const [, episodes] of showMap) {
		episodes.sort((a, b) => {
			const sa = a.metadata?.season_number ?? 0;
			const sb = b.metadata?.season_number ?? 0;
			if (sa !== sb) return sa - sb;
			const ea = a.metadata?.episode_number ?? 0;
			const eb = b.metadata?.episode_number ?? 0;
			return ea - eb;
		});
	}

	return showMap;
}

/** Get the best rated items for hero selection */
export function getBestRated(items: MediaWithMetadata[], count = 5): MediaWithMetadata[] {
	return [...items]
		.filter((i) => i.metadata?.rating != null && i.backdrop_path != null)
		.sort((a, b) => (b.metadata?.rating ?? 0) - (a.metadata?.rating ?? 0))
		.slice(0, count);
}

/** Get recently added items */
export function getRecentlyAdded(items: MediaWithMetadata[], count = 20): MediaWithMetadata[] {
	return [...items]
		.filter((i) => i.metadata?.fetched_at)
		.sort((a, b) => {
			const da = a.metadata?.fetched_at ?? "";
			const db = b.metadata?.fetched_at ?? "";
			return db.localeCompare(da);
		})
		.slice(0, count);
}
