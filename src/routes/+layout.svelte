<script lang="ts">
	import "@/app.css";
	import { invoke } from "@tauri-apps/api/core";
	import { listen } from "@tauri-apps/api/event";
	import { attachConsole, info, warn } from "@tauri-apps/plugin-log";
	import { onMount } from "svelte";
	import { toast } from "svelte-sonner";
	import LanguageSwitcher from "@/lib/components/app/language-switcher.svelte";
	import UserCreateForm from "@/lib/components/app/user-create-form.svelte";
	import UserLoginForm from "@/lib/components/app/user-login-form.svelte";
	import DevToolbar from "@/lib/components/dev/dev-toolbar.svelte";
	import { Toaster } from "@/lib/components/ui/sonner";
	import { currentUser, isUserLoading } from "@/lib/stores/user";
	import { m } from "$lib/paraglide/messages.js";

	let { children } = $props();

	let existingUser = $state<{ id: string; username: string } | null>(null);
	let previousUserId = $state<string | null>(null);

	interface MediaChangePayload {
		kind: string;
		added: string[];
		modified: string[];
		removed: string[];
		total: number;
	}

	onMount(async () => {
		// Attach Rust logs to the browser console
		attachConsole();

		try {
			const user = await invoke<{ id: string; username: string } | null>("check_user_exists");
			existingUser = user;
		} catch (error) {
			console.error("Failed to check user:", error);
		} finally {
			isUserLoading.set(false);
		}

		// Listen for media library changes from the watcher
		listen<MediaChangePayload>("media-change", (event) => {
			const { kind, added, modified, removed, total } = event.payload;
			info(
				`[Layout] Media library changed: ${kind}, +${added.length} ~${modified.length} -${removed.length}, ${total} total`
			);

			// Only show toasts for incremental changes (not on initial full scan)
			if (kind !== "incremental") return;

			if (added.length > 0) {
				toast.success(m.watcher_files_added({ count: String(added.length) }), {
					description: added.join(", ")
				});
			}

			if (modified.length > 0) {
				toast.info(m.watcher_files_modified({ count: String(modified.length) }), {
					description: modified.join(", ")
				});
			}

			if (removed.length > 0) {
				toast.warning(m.watcher_files_removed({ count: String(removed.length) }), {
					description: removed.join(", ")
				});
			}
		});
	});

	// Start/stop watcher when user logs in/out
	$effect(() => {
		const userId = $currentUser?.id ?? null;

		if (userId === previousUserId) return;

		// Stop watcher for previous user
		if (previousUserId) {
			invoke("stop_media_watcher", { userId: previousUserId }).catch((e: unknown) => {
				warn("[Layout] Failed to stop media watcher: " + String(e));
			});
		}

		// Start watcher for new user (respects scan_on_startup & live_scan settings internally)
		if (userId) {
			invoke("start_media_watcher", { userId }).catch((e: unknown) => {
				warn("[Layout] Failed to start media watcher: " + String(e));
			});
			info("[Layout] Media watcher init for user " + userId);
		}

		previousUserId = userId;
	});
</script>

<Toaster richColors />

{#if $isUserLoading}
	<!-- Loading state -->
{:else if $currentUser}
	<div class="h-screen overflow-y-auto">
		<LanguageSwitcher />
		<main>
			{@render children()}
		</main>
	</div>
{:else if existingUser}
	<div class="h-screen overflow-hidden flex flex-col dark:bg-background">
		<LanguageSwitcher />
		<UserLoginForm userId={existingUser.id} username={existingUser.username} />
	</div>
{:else}
	<div class="h-screen overflow-hidden flex flex-col">
		<LanguageSwitcher />
		<UserCreateForm />
	</div>
{/if}

<DevToolbar />
