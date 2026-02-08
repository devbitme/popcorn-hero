<script lang="ts">
	import "@/app.css";
	import { invoke } from "@tauri-apps/api/core";
	import { attachConsole } from "@tauri-apps/plugin-log";
	import { onMount } from "svelte";
	import LanguageSwitcher from "@/lib/components/app/language-switcher.svelte";
	import UserCreateForm from "@/lib/components/app/user-create-form.svelte";
	import UserLoginForm from "@/lib/components/app/user-login-form.svelte";
	import DevToolbar from "@/lib/components/dev/dev-toolbar.svelte";
	import { Toaster } from "@/lib/components/ui/sonner";
	import { currentUser, isUserLoading } from "@/lib/stores/user";

	let { children } = $props();

	let existingUser = $state<{ id: string; username: string } | null>(null);

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
	});
</script>

<Toaster richColors />
<LanguageSwitcher />

{#if $isUserLoading}
	<!-- Loading state -->
{:else if $currentUser}
	<main>
		{@render children()}
	</main>
{:else if existingUser}
	<UserLoginForm userId={existingUser.id} username={existingUser.username} />
{:else}
	<UserCreateForm />
{/if}

<DevToolbar />
