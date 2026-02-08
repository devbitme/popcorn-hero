import { writable } from "svelte/store";

export interface UserProfile {
	id: string;
	username: string;
	created_at: string;
}

export const currentUser = writable<UserProfile | null>(null);
export const isUserLoading = writable(true);
