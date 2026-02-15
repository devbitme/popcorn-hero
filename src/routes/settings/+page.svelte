<script lang="ts">
	import { CircleHelp, Download, ExternalLink, FolderOpen, GripVertical, Plus, RefreshCw, Trash2 } from "@lucide/svelte";
	import { invoke } from "@tauri-apps/api/core";
	import { info, warn } from "@tauri-apps/plugin-log";
	import { onMount } from "svelte";
	import { toast } from "svelte-sonner";
	import { Button } from "$lib/components/ui/button";
	import * as Card from "$lib/components/ui/card";
	import * as Dialog from "$lib/components/ui/dialog";
	import { Input } from "$lib/components/ui/input";
	import { Label } from "$lib/components/ui/label";
	import * as Popover from "$lib/components/ui/popover";
	import { ScrollArea } from "$lib/components/ui/scroll-area";
	import { Separator } from "$lib/components/ui/separator";
	import { Switch } from "$lib/components/ui/switch";
	import { m } from "$lib/paraglide/messages.js";
	import { currentUser } from "$lib/stores/user";

	interface MediaConfig {
		folders: string[];
	}

	interface MetadataProviderConfig {
		id: string;
		name: string;
		enabled: boolean;
		api_key: string;
	}

	interface AppSettings {
		media: MediaConfig;
		scan_on_startup: boolean;
		live_scan: boolean;
		metadata_providers: MetadataProviderConfig[];
	}

	interface ScanResult {
		total_found: number;
		media_entries: { id: string; path: string; filename: string; extension: string; size_bytes: number }[];
	}

	const PROVIDER_LINKS: Record<string, string> = {
		tmdb: "https://www.themoviedb.org/settings/api",
		omdb: "https://www.omdbapi.com/apikey.aspx"
	};

	const PROVIDER_HELP: Record<string, () => { steps: string[]; link: string; linkLabel: string }> = {
		tmdb: () => ({
			steps: [
				m.settings_metadata_help_tmdb_step1(),
				m.settings_metadata_help_tmdb_step2(),
				m.settings_metadata_help_tmdb_step3(),
				m.settings_metadata_help_tmdb_step4()
			],
			link: "https://www.themoviedb.org/signup",
			linkLabel: "themoviedb.org"
		}),
		omdb: () => ({
			steps: [
				m.settings_metadata_help_omdb_step1(),
				m.settings_metadata_help_omdb_step2(),
				m.settings_metadata_help_omdb_step3()
			],
			link: "https://www.omdbapi.com/apikey.aspx",
			linkLabel: "omdbapi.com"
		})
	};

	let folders = $state<string[]>([]);
	let scanOnStartup = $state(true);
	let liveScan = $state(true);
	let metadataProviders = $state<MetadataProviderConfig[]>([]);
	let isLoading = $state(true);
	let isScanning = $state(false);
	let isFetchingMetadata = $state(false);
	let confirmRemoveFolder = $state<string | null>(null);
	let dragIndex = $state<number | null>(null);

	onMount(async () => {
		await loadConfig();
	});

	async function loadConfig() {
		try {
			const userId = $currentUser?.id;
			if (!userId) return;
			const settings = await invoke<AppSettings>("get_settings", { userId });
			folders = settings.media.folders;
			scanOnStartup = settings.scan_on_startup;
			liveScan = settings.live_scan;
			metadataProviders = settings.metadata_providers ?? [];
			info("[Settings] Loaded settings: " + folders.length + " folder(s), scanOnStartup=" + scanOnStartup + ", liveScan=" + liveScan + ", providers=" + metadataProviders.length);
		} catch (error) {
			warn("[Settings] Failed to load settings: " + String(error));
			toast.error(m.settings_media_error(), { description: String(error) });
		} finally {
			isLoading = false;
		}
	}

	async function handleToggleScanOnStartup(checked: boolean) {
		scanOnStartup = checked;
		await saveSettingsToggle();
	}

	async function handleToggleLiveScan(checked: boolean) {
		liveScan = checked;
		await saveSettingsToggle();
		// Start or stop the watcher based on the new setting
		const userId = $currentUser?.id;
		if (!userId) return;
		try {
			if (checked) {
				await invoke("start_media_watcher", { userId });
				info("[Settings] Live scan enabled, watcher started");
			} else {
				await invoke("stop_media_watcher", { userId });
				info("[Settings] Live scan disabled, watcher stopped");
			}
		} catch (error) {
			warn("[Settings] Failed to toggle watcher: " + String(error));
		}
	}

	async function saveSettingsToggle() {
		try {
			const userId = $currentUser?.id;
			if (!userId) return;
			const settings: AppSettings = {
				media: { folders },
				scan_on_startup: scanOnStartup,
				live_scan: liveScan,
				metadata_providers: metadataProviders
			};
			await invoke("update_settings", { userId, settings });
			info("[Settings] Settings saved: scanOnStartup=" + scanOnStartup + ", liveScan=" + liveScan);
		} catch (error) {
			warn("[Settings] Failed to save settings: " + String(error));
			toast.error(m.settings_media_error(), { description: String(error) });
		}
	}

	async function handleAddFolder() {
		try {
			const userId = $currentUser?.id;
			if (!userId) return;
			const picked = await invoke<string | null>("pick_folder");
			if (!picked) return;

			const config = await invoke<MediaConfig>("add_media_folder", { userId, folder: picked });
			folders = config.folders;
			toast.success(m.settings_media_folder_added());
			info("[Settings] Folder added: " + picked);
		} catch (error) {
			const errMsg = String(error);
			if (errMsg.includes("already")) {
				toast.error(m.settings_media_folder_exists());
			} else if (errMsg.includes("not exist")) {
				toast.error(m.settings_media_folder_invalid());
			} else {
				toast.error(m.settings_media_error(), { description: errMsg });
			}
			warn("[Settings] Failed to add folder: " + errMsg);
		}
	}

	async function handleRemoveFolder(folder: string) {
		try {
			const userId = $currentUser?.id;
			if (!userId) return;
			const config = await invoke<MediaConfig>("remove_media_folder", { userId, folder });
			folders = config.folders;
			toast.success(m.settings_media_folder_removed());
			info("[Settings] Folder removed: " + folder);
		} catch (error) {
			toast.error(m.settings_media_error(), { description: String(error) });
			warn("[Settings] Failed to remove folder: " + String(error));
		} finally {
			confirmRemoveFolder = null;
		}
	}

	async function handleScan() {
		isScanning = true;
		try {
			const userId = $currentUser?.id;
			if (!userId) return;
			const result = await invoke<ScanResult>("scan_media_folders", { userId });
			toast.success(m.settings_media_scan_result({ count: String(result.total_found) }));
			info("[Settings] Scan complete: " + result.total_found + " media file(s) found");
		} catch (error) {
			toast.error(m.settings_media_error(), { description: String(error) });
			warn("[Settings] Scan failed: " + String(error));
		} finally {
			isScanning = false;
		}
	}

	function formatPath(path: string): string {
		const parts = path.split("/");
		if (parts.length > 3) {
			return "…/" + parts.slice(-3).join("/");
		}
		return path;
	}

	async function handleToggleProvider(index: number, checked: boolean) {
		metadataProviders[index].enabled = checked;
		metadataProviders = [...metadataProviders];
		await saveSettingsToggle();
		info("[Settings] Provider " + metadataProviders[index].id + " " + (checked ? "enabled" : "disabled"));
	}

	async function handleApiKeyChange(index: number, value: string) {
		metadataProviders[index].api_key = value;
		metadataProviders = [...metadataProviders];
	}

	async function handleApiKeyBlur() {
		await saveSettingsToggle();
	}

	function handleDragStart(index: number) {
		dragIndex = index;
	}

	function handleDragOver(e: DragEvent, index: number) {
		e.preventDefault();
		if (dragIndex === null || dragIndex === index) return;
		const items = [...metadataProviders];
		const [moved] = items.splice(dragIndex, 1);
		items.splice(index, 0, moved);
		metadataProviders = items;
		dragIndex = index;
	}

	function handleDragEnd() {
		dragIndex = null;
		saveSettingsToggle();
	}

	async function handleFetchMetadata() {
		isFetchingMetadata = true;
		try {
			const userId = $currentUser?.id;
			if (!userId) return;
			const result = await invoke<string>("fetch_all_metadata", { userId });
			toast.success(m.settings_metadata_fetch_result(), { description: result });
			info("[Settings] Metadata fetch complete: " + result);
		} catch (error) {
			toast.error(m.settings_media_error(), { description: String(error) });
			warn("[Settings] Metadata fetch failed: " + String(error));
		} finally {
			isFetchingMetadata = false;
		}
	}
</script>

<div class="mx-auto max-w-2xl p-6">
	<h1 class="mb-6 text-2xl font-bold">{m.settings_title()}</h1>

	<Card.Root>
		<Card.Header>
			<Card.Title class="flex items-center gap-2">
				<FolderOpen class="size-5" />
				{m.settings_media_title()}
			</Card.Title>
			<Card.Description>
				{m.settings_media_description()}
			</Card.Description>
		</Card.Header>
		<Card.Content>
			{#if isLoading}
				<div class="flex items-center justify-center py-8">
					<RefreshCw class="size-5 animate-spin text-muted-foreground" />
				</div>
			{:else}
				<div class="space-y-4">
					<!-- Folder list -->
					{#if folders.length === 0}
						<p class="py-4 text-center text-sm text-muted-foreground">
							{m.settings_media_no_folders()}
						</p>
					{:else}
						<ScrollArea class="max-h-64">
							<div class="space-y-2">
								{#each folders as folder (folder)}
									<div
										class="flex items-center justify-between rounded-md border px-3 py-2"
									>
										<div class="flex min-w-0 items-center gap-2">
											<FolderOpen class="size-4 shrink-0 text-muted-foreground" />
											<span class="truncate text-sm" title={folder}>
												{formatPath(folder)}
											</span>
										</div>
										<Button
											variant="ghost"
											size="sm"
											class="shrink-0 text-destructive hover:bg-destructive/10 hover:text-destructive"
											onclick={() => (confirmRemoveFolder = folder)}
										>
											<Trash2 class="size-4" />
											<span class="sr-only">{m.settings_media_remove()}</span>
										</Button>
									</div>
								{/each}
							</div>
						</ScrollArea>
					{/if}

					<Separator />

					<!-- Actions -->
					<div class="flex flex-wrap items-center gap-2">
						<Button variant="outline" onclick={handleAddFolder}>
							<Plus class="size-4" />
							{m.settings_media_add_folder()}
						</Button>

						{#if folders.length > 0}
							<Button
								variant="default"
								onclick={handleScan}
								disabled={isScanning}
							>
								<RefreshCw class="size-4 {isScanning ? 'animate-spin' : ''}" />
								{isScanning ? m.settings_media_scanning() : m.settings_media_scan()}
							</Button>
						{/if}
					</div>
					<Separator />

					<div class="flex items-center justify-between gap-4">
						<div class="space-y-0.5">
							<Label for="scan-on-startup">{m.settings_scan_on_startup()}</Label>
							<p class="text-sm text-muted-foreground">
								{m.settings_scan_on_startup_description()}
							</p>
						</div>
						<Switch
							id="scan-on-startup"
							checked={scanOnStartup}
							onCheckedChange={handleToggleScanOnStartup}
						/>
					</div>

					<Separator />

					<div class="flex items-center justify-between gap-4">
						<div class="space-y-0.5">
							<Label for="live-scan">{m.settings_live_scan()}</Label>
							<p class="text-sm text-muted-foreground">
								{m.settings_live_scan_description()}
							</p>
						</div>
						<Switch
							id="live-scan"
							checked={liveScan}
							onCheckedChange={handleToggleLiveScan}
						/>
					</div>
				</div>
			{/if}
		</Card.Content>
	</Card.Root>

	<!-- Metadata Providers -->
	<Card.Root class="mt-4">
		<Card.Header>
			<Card.Title class="flex items-center gap-2">
				<Download class="size-5" />
				{m.settings_metadata_title()}
			</Card.Title>
			<Card.Description>
				{m.settings_metadata_description()}
			</Card.Description>
		</Card.Header>
		<Card.Content>
			{#if isLoading}
				<div class="flex items-center justify-center py-8">
					<RefreshCw class="size-5 animate-spin text-muted-foreground" />
				</div>
			{:else}
				<div class="space-y-4">
					{#if metadataProviders.length === 0}
						<p class="py-4 text-center text-sm text-muted-foreground">
							{m.settings_metadata_no_providers()}
						</p>
					{:else}
						<div class="space-y-3">
							{#each metadataProviders as provider, index (provider.id)}
								<div
									class="rounded-md border p-4 transition-colors {dragIndex === index ? 'border-primary bg-muted/50' : ''}"
									draggable="true"
									ondragstart={() => handleDragStart(index)}
									ondragover={(e) => handleDragOver(e, index)}
									ondragend={handleDragEnd}
									role="listitem"
								>
									<div class="flex items-center justify-between gap-4">
										<div class="flex items-center gap-3">
											<button
												class="cursor-grab text-muted-foreground hover:text-foreground active:cursor-grabbing"
												aria-label="Drag to reorder"
											>
												<GripVertical class="size-4" />
											</button>
											<div class="space-y-0.5">
												<div class="flex items-center gap-2">
													<span class="text-sm font-medium">{provider.name}</span>
													{#if PROVIDER_HELP[provider.id]}
														{@const help = PROVIDER_HELP[provider.id]()}
														<Popover.Root>
															<Popover.Trigger class="cursor-pointer text-muted-foreground hover:text-foreground transition-colors">
																<CircleHelp class="size-4" />
																<span class="sr-only">{m.settings_metadata_help()}</span>
															</Popover.Trigger>
															<Popover.Content class="w-80" side="top">
																<div class="space-y-3">
																	<p class="text-sm font-medium">{m.settings_metadata_help_title()}</p>
																	<ol class="list-decimal space-y-1 pl-4 text-sm text-muted-foreground">
																		{#each help.steps as step}
																			<li>{step}</li>
																		{/each}
																	</ol>
																	<a
																		href={help.link}
																		target="_blank"
																		rel="noopener noreferrer"
																		class="inline-flex items-center gap-1.5 text-sm font-medium text-primary hover:underline"
																	>
																		<ExternalLink class="size-3.5" />
																		{help.linkLabel}
																	</a>
																</div>
															</Popover.Content>
														</Popover.Root>
													{/if}
												</div>
											</div>
										</div>
										<Switch
											checked={provider.enabled}
											onCheckedChange={(checked) => handleToggleProvider(index, checked)}
										/>
									</div>
									{#if provider.enabled}
										<div class="mt-3 ml-7">
											<Label class="text-xs text-muted-foreground">{m.settings_metadata_api_key()}</Label>
											<Input
												type="password"
												class="mt-1"
												placeholder={m.settings_metadata_api_key_placeholder()}
												value={provider.api_key}
												oninput={(e) => handleApiKeyChange(index, e.currentTarget.value)}
												onblur={handleApiKeyBlur}
											/>
										</div>
									{/if}
								</div>
							{/each}
						</div>
					{/if}

					<Separator />

					<div class="flex flex-wrap items-center gap-2">
						<Button
							variant="default"
							onclick={handleFetchMetadata}
							disabled={isFetchingMetadata || !metadataProviders.some((p) => p.enabled && p.api_key)}
						>
							<Download class="size-4 {isFetchingMetadata ? 'animate-bounce' : ''}" />
							{isFetchingMetadata ? m.settings_metadata_fetching() : m.settings_metadata_fetch()}
						</Button>
					</div>
				</div>
			{/if}
		</Card.Content>
	</Card.Root>
</div>

<!-- Confirm remove dialog -->
<Dialog.Root
	open={confirmRemoveFolder !== null}
	onOpenChange={(open) => {
		if (!open) confirmRemoveFolder = null;
	}}
>
	<Dialog.Content class="sm:max-w-md">
		<Dialog.Header>
			<Dialog.Title>{m.settings_media_remove()}</Dialog.Title>
			<Dialog.Description>
				{confirmRemoveFolder ? formatPath(confirmRemoveFolder) : ""}
			</Dialog.Description>
		</Dialog.Header>
		<Dialog.Footer>
			<Button variant="outline" onclick={() => (confirmRemoveFolder = null)}>
				Cancel
			</Button>
			<Button
				variant="destructive"
				onclick={() => {
					if (confirmRemoveFolder) handleRemoveFolder(confirmRemoveFolder);
				}}
			>
				{m.settings_media_remove()}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
