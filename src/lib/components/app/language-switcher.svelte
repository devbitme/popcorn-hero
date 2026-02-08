<script lang="ts">
	import CircleUserIcon from "@lucide/svelte/icons/circle-user";
	import { info } from "@tauri-apps/plugin-log";
	import * as Icon from "svelte-flag-icons";
	import { Button } from "$lib/components/ui/button/index.js";
	import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
	import { m } from "$lib/paraglide/messages.js";
	import { getLocale, locales, setLocale } from "$lib/paraglide/runtime.js";

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

<div class="fixed top-4 right-8 z-50 flex items-center gap-2">
	<!-- Avatar / Account Menu -->
	<DropdownMenu.Root>
		<DropdownMenu.Trigger>
			{#snippet child({ props })}
				<Button {...props} variant="ghost" size="icon-sm" class="cursor-pointer rounded-full">
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
		</DropdownMenu.Content>
	</DropdownMenu.Root>

	<!-- Separator -->
	<div class="mx-1 h-5 w-px bg-border"></div>

	<!-- Language Switcher -->
	<DropdownMenu.Root>
		<DropdownMenu.Trigger>
			{#snippet child({ props })}
				<Button {...props} variant="ghost" size="icon-lg" class="cursor-pointer rounded-full">
					<svelte:component
						this={getFlagComponent(getLocale())}
						class="size-5!"
					/>
				</Button>
			{/snippet}
		</DropdownMenu.Trigger>
		<DropdownMenu.Content align="end" class="min-w-0 bg-primary">
			{#each locales.filter((l) => l !== getLocale()) as locale}
				<DropdownMenu.Item
					class="cursor-pointer justify-center px-2"
					onclick={() => {
						info(`[LanguageSwitcher] Changing locale from ${getLocale()} to ${locale}`);
						setLocale(locale);
					}}
				>
					<svelte:component
						this={getFlagComponent(locale)}
						size="18"
					/>
				</DropdownMenu.Item>
			{/each}
		</DropdownMenu.Content>
	</DropdownMenu.Root>
</div>
