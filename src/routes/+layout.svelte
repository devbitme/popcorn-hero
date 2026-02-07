<script lang="ts">
	import "@/app.css";
	import { attachConsole } from "@tauri-apps/plugin-log";
	import { onMount } from "svelte";
	import LanguageSwitcher from "@/lib/components/app/language-switcher.svelte";
	import DevToolbar from "@/lib/components/dev/dev-toolbar.svelte";

	let { children } = $props();

	onMount(() => {
		// Attach Rust logs to the browser console
		attachConsole();
	});
</script>

<LanguageSwitcher />

<main>
	{@render children()}
</main>

<DevToolbar />
