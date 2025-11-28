<script lang="ts">
	import "@/app.css";
	import LanguageSwitcher from "@/lib/components/app/language-switcher.svelte";

	let { children } = $props();
</script>

<LanguageSwitcher />

<main>
	{@render children()}
</main>
