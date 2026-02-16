<script lang="ts">
	import CircleUserIcon from "@lucide/svelte/icons/circle-user";
	import Moon from "@lucide/svelte/icons/moon";
	import Sun from "@lucide/svelte/icons/sun";
	import { info } from "@tauri-apps/plugin-log";
	import * as Icon from "svelte-flag-icons";
	import { page } from "$app/stores";
	import { Button } from "$lib/components/ui/button/index.js";
	import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
	import { m } from "$lib/paraglide/messages.js";
	import { getLocale, locales, setLocale } from "$lib/paraglide/runtime.js";
	import { currentUser } from "$lib/stores/user";

	let isDark = $state(typeof document !== "undefined" && document.documentElement.classList.contains("dark"));

	function toggleDarkMode() {
		isDark = !isDark;
		document.documentElement.classList.toggle("dark", isDark);
		localStorage.setItem("theme", isDark ? "dark" : "light");
		info(`[Theme] Switched to ${isDark ? "dark" : "light"} mode`);
	}

	// Restore theme on load
	if (typeof document !== "undefined") {
		const saved = localStorage.getItem("theme");
		if (saved === "dark" || (!saved && window.matchMedia("(prefers-color-scheme: dark)").matches)) {
			document.documentElement.classList.add("dark");
			isDark = true;
		}
	}

	// Exceptions for locale to flag icon mapping
	const flagExceptions: Record<string, string> = {
		en: "Gb",
		ja: "Jp",
	};

	function getFlagComponent(locale: string) {
		const flagCode =
			flagExceptions[locale] ||
			locale.charAt(0).toUpperCase() + locale.slice(1);
		return (Icon as any)[flagCode];
	}
</script>

<nav class="sticky top-0 w-full flex items-center justify-between px-8 py-3 z-50 bg-background/80 backdrop-blur-md text-foreground border-b border-border/50">
	<!-- Left: Navigation links -->
	{#if $currentUser}
	<div class="flex items-center gap-6">
		<a href="/" class="text-sm text-foreground hover:text-foreground/70 transition-colors {$page.url.pathname === '/' ? 'font-bold' : 'font-medium'}">{m.nav_home()}</a>
		<a href="/series" class="text-sm text-foreground hover:text-foreground/70 transition-colors {$page.url.pathname === '/series' ? 'font-bold' : 'font-medium'}">{m.nav_series()}</a>
		<a href="/movies" class="text-sm text-foreground hover:text-foreground/70 transition-colors {$page.url.pathname === '/movies' ? 'font-bold' : 'font-medium'}">{m.nav_movies()}</a>
		<a href="/library" class="text-sm text-foreground hover:text-foreground/70 transition-colors {$page.url.pathname === '/library' ? 'font-bold' : 'font-medium'}">{m.nav_library()}</a>
	</div>
	{:else}
	<div></div>
	{/if}

	<!-- Right: User + Language -->
	<div class="flex items-center gap-2">
		<!-- Avatar / Account Menu -->
		{#if $currentUser}
		<DropdownMenu.Root>
			<DropdownMenu.Trigger>
				{#snippet child({ props })}
<Button {...props} variant="ghost" size="icon-lg" class="cursor-pointer rounded-full text-foreground hover:text-foreground/70 hover:bg-foreground/10">
						<CircleUserIcon class="size-6" strokeWidth={1.5} />
					</Button>
				{/snippet}
			</DropdownMenu.Trigger>
			<DropdownMenu.Content align="end">
				<DropdownMenu.Item>
					<a href="/account">{m.user_account()}</a>
				</DropdownMenu.Item>
				<DropdownMenu.Item>
					<a href="/settings">{m.user_settings()}</a>
				</DropdownMenu.Item>
				<DropdownMenu.Separator />
				<DropdownMenu.Item
					class="cursor-pointer"
					onclick={() => currentUser.set(null)}
				>
					{m.user_logout()}
				</DropdownMenu.Item>
			</DropdownMenu.Content>
		</DropdownMenu.Root>
		{/if}

		<!-- Dark Mode Toggle -->
		<Button variant="ghost" size="icon-lg" class="cursor-pointer rounded-full text-foreground hover:text-foreground/70 hover:bg-foreground/10" onclick={toggleDarkMode}>
			{#if isDark}
				<Sun class="size-5" strokeWidth={1.5} />
			{:else}
				<Moon class="size-5" strokeWidth={1.5} />
			{/if}
		</Button>

		<!-- Language Switcher -->
		<DropdownMenu.Root>
			<DropdownMenu.Trigger>
				{#snippet child({ props })}
<Button {...props} variant="ghost" size="icon-lg" class="cursor-pointer rounded-full hover:bg-foreground/10">
						{@const FlagIcon = getFlagComponent(getLocale())}
						<FlagIcon class="size-5!" />
					</Button>
				{/snippet}
			</DropdownMenu.Trigger>
			<DropdownMenu.Content align="end" class="min-w-0">
				{#each locales.filter((l) => l !== getLocale()) as locale}
					<DropdownMenu.Item
						class="cursor-pointer justify-center px-2"
						onclick={() => {
							info(`[LanguageSwitcher] Changing locale from ${getLocale()} to ${locale}`);
							setLocale(locale);
						}}
					>
						{@const FlagIcon = getFlagComponent(locale)}
						<FlagIcon size="18" />
					</DropdownMenu.Item>
				{/each}
			</DropdownMenu.Content>
		</DropdownMenu.Root>
	</div>
</nav>
