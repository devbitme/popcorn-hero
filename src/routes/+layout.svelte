<script lang="ts">
	import "@/app.css";
	import LanguageSwitcher from "@/lib/components/app/language-switcher.svelte";
	import DevToolbar from "@/lib/components/dev/dev-toolbar.svelte";

	let { children } = $props();
</script>

<LanguageSwitcher />

<main>
	{@render children()}
</main>

<DevToolbar />
