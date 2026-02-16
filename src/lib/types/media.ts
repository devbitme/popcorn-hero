/** A media entry from Rust with metadata and image paths */
export interface MediaWithMetadata {
	id: string;
	path: string;
	filename: string;
	extension: string;
	size_bytes: number;
	metadata: VideoMetadata | null;
	poster_path: string | null;
	backdrop_path: string | null;
	still_path: string | null;
}

export interface VideoMetadata {
	title: string;
	original_title?: string;
	year?: number;
	overview?: string;
	tagline?: string;
	genres: string[];
	runtime_minutes?: number;
	rating?: number;
	vote_count?: number;
	release_date?: string;
	imdb_id?: string;
	tmdb_id?: number;
	cast: CastMember[];
	crew: CrewMember[];
	studios: string[];
	language?: string;
	status?: string;
	season_number?: number;
	episode_number?: number;
	episode_title?: string;
	episode_overview?: string;
	episode_still?: string;
	images: MetadataImages;
	file_size_bytes?: number;
	container?: string;
	file_path?: string;
	provider: string;
	fetched_at: string;
	/** Media type: "movie", "tv", or "unknown" */
	media_type?: string;
}

export interface CastMember {
	name: string;
	character?: string;
	profile_path?: string;
}

export interface CrewMember {
	name: string;
	job?: string;
	profile_path?: string;
}

export interface MetadataImages {
	poster?: string;
	backdrop?: string;
}
