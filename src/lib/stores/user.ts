import { writable } from "svelte/store";

export interface UserProfile {
	id: string;
	username: string;
	created_at: string;
}

function createPersistentUser() {
	const SESSION_KEY = "popcorn_hero_current_user";

	// Restore from sessionStorage if available
	let initial: UserProfile | null = null;
	if (typeof sessionStorage !== "undefined") {
		const stored = sessionStorage.getItem(SESSION_KEY);
		if (stored) {
			try {
				initial = JSON.parse(stored);
			} catch {
				sessionStorage.removeItem(SESSION_KEY);
			}
		}
	}

	const store = writable<UserProfile | null>(initial);

	// Sync to sessionStorage on every change
	store.subscribe((value) => {
		if (typeof sessionStorage !== "undefined") {
			if (value) {
				sessionStorage.setItem(SESSION_KEY, JSON.stringify(value));
			} else {
				sessionStorage.removeItem(SESSION_KEY);
			}
		}
	});

	return store;
}

export const currentUser = createPersistentUser();
export const isUserLoading = writable(true);
