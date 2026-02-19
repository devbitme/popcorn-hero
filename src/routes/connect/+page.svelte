<script lang="ts">
	import { Check, ClipboardCopy, Loader2, Share2, Trash2, Wifi, WifiOff } from "@lucide/svelte";
	import { invoke } from "@tauri-apps/api/core";
	import { listen } from "@tauri-apps/api/event";
	import { info, warn } from "@tauri-apps/plugin-log";
	import { onDestroy, onMount } from "svelte";
	import { toast } from "svelte-sonner";
	import { Badge } from "$lib/components/ui/badge";
	import { Button } from "$lib/components/ui/button";
	import * as Card from "$lib/components/ui/card";
	import { Input } from "$lib/components/ui/input";
	import { Label } from "$lib/components/ui/label";
	import { Separator } from "$lib/components/ui/separator";
	import { Textarea } from "$lib/components/ui/textarea";
	import { m } from "$lib/paraglide/messages.js";
	import type { PeerInfo } from "$lib/stores/peers";
	import { isPeerEndpointRunning, localNodeId, peers } from "$lib/stores/peers";
	import { currentUser } from "$lib/stores/user";

	let generatedTicket = $state("");
	let remoteTicket = $state("");
	let remoteUsername = $state("");
	let isGenerating = $state(false);
	let isConnecting = $state(false);
	let isStarting = $state(true);
	let justCopied = $state(false);

	let unlistenPeerStatus: (() => void) | null = null;

	onMount(async () => {
		// Listen for peer status events from the backend
		unlistenPeerStatus = await listen<{ peers: PeerInfo[] }>("peer-status", (event) => {
			info(`[Connect] Peer status update: ${event.payload.peers.length} peers`);
			peers.set(event.payload.peers);
		});

		// Start the iroh endpoint if not already running
		if (!$isPeerEndpointRunning) {
			try {
				info("[Connect] Starting iroh endpoint...");
				const nodeId = await invoke<string>("peer_start");
				localNodeId.set(nodeId);
				isPeerEndpointRunning.set(true);
				info(`[Connect] Endpoint started, node_id=${nodeId}`);

				// Load existing peer list
				const existingPeers = await invoke<PeerInfo[]>("peer_list");
				peers.set(existingPeers);
			} catch (error) {
				warn("[Connect] Failed to start endpoint: " + String(error));
				toast.error(m.connect_endpoint_error());
			} finally {
				isStarting = false;
			}
		} else {
			isStarting = false;
			// Refresh peer list
			try {
				const existingPeers = await invoke<PeerInfo[]>("peer_list");
				peers.set(existingPeers);
			} catch {
				// Endpoint might have stopped
			}
		}
	});

	onDestroy(() => {
		if (unlistenPeerStatus) {
			unlistenPeerStatus();
		}
	});

	async function generateTicket() {
		isGenerating = true;
		try {
			const ticket = await invoke<string>("peer_generate_ticket");
			generatedTicket = ticket;
			info(`[Connect] Ticket generated (${ticket.length} chars)`);
		} catch (error) {
			warn("[Connect] Failed to generate ticket: " + String(error));
			toast.error(String(error));
		} finally {
			isGenerating = false;
		}
	}

	async function copyTicket() {
		if (!generatedTicket) return;
		await navigator.clipboard.writeText(generatedTicket);
		justCopied = true;
		toast.success(m.connect_generate_copied());
		info("[Connect] Ticket copied to clipboard");
		setTimeout(() => {
			justCopied = false;
		}, 2000);
	}

	async function connectToPeer() {
		if (!remoteTicket.trim()) return;
		isConnecting = true;
		try {
			const username = remoteUsername.trim() || "Unknown";
			const peer = await invoke<PeerInfo>("peer_connect", {
				ticketStr: remoteTicket.trim(),
				username,
			});
			info(`[Connect] Connected to peer ${peer.node_id}`);
			toast.success(m.connect_join_success({ username: peer.username }));
			remoteTicket = "";
			remoteUsername = "";
		} catch (error) {
			warn("[Connect] Connection failed: " + String(error));
			toast.error(m.connect_join_error());
		} finally {
			isConnecting = false;
		}
	}

	async function disconnectPeer(nodeId: string) {
		try {
			await invoke("peer_disconnect", { nodeId });
			info(`[Connect] Disconnected peer ${nodeId}`);
		} catch (error) {
			warn("[Connect] Failed to disconnect peer: " + String(error));
		}
	}

	function formatLastSeen(timestamp: number | null): string {
		if (!timestamp) return "-";
		const date = new Date(timestamp);
		return date.toLocaleTimeString();
	}
</script>

<div class="max-w-4xl mx-auto p-8 space-y-8">
	<!-- Page Header -->
	<div>
		<h1 class="text-3xl font-bold tracking-tight">{m.connect_title()}</h1>
		<p class="text-muted-foreground mt-2">{m.connect_description()}</p>
	</div>

	{#if isStarting}
		<div class="flex items-center justify-center gap-3 py-16 text-muted-foreground">
			<Loader2 class="size-5 animate-spin" />
			<span>{m.connect_endpoint_starting()}</span>
		</div>
	{:else}

	<!-- Generate & Join side by side -->
	<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
		<!-- Generate Ticket Card -->
		<Card.Root class="flex flex-col">
			<Card.Header>
				<Card.Title>{m.connect_generate_title()}</Card.Title>
				<Card.Description>{m.connect_generate_description()}</Card.Description>
			</Card.Header>
			<Card.Content class="flex flex-col flex-1 gap-4">
				<Button
					class="w-full cursor-pointer"
					onclick={generateTicket}
					disabled={isGenerating}
				>
					{#if isGenerating}
						<Loader2 class="size-4 animate-spin mr-2" />
					{:else}
						<Share2 class="size-4 mr-2" />
					{/if}
					{m.connect_generate_btn()}
				</Button>

				{#if generatedTicket}
					<div class="relative flex-1 flex flex-col">
						<Textarea
							value={generatedTicket}
							readonly
							class="font-mono text-xs resize-none min-h-28 flex-1 pr-10"
						/>
						<Button
							variant="ghost"
							size="icon"
							class="absolute top-2 right-2 cursor-pointer"
							onclick={copyTicket}
						>
							{#if justCopied}
								<Check class="size-4 text-green-500" />
							{:else}
								<ClipboardCopy class="size-4" />
							{/if}
						</Button>
					</div>
				{:else}
					<div class="flex-1 flex items-center justify-center">
						<p class="text-xs text-muted-foreground italic text-center">
							{m.connect_generate_placeholder()}
						</p>
					</div>
				{/if}
			</Card.Content>
		</Card.Root>

		<!-- Join Peer Card -->
		<Card.Root>
			<Card.Header>
				<Card.Title>{m.connect_join_title()}</Card.Title>
				<Card.Description>{m.connect_join_description()}</Card.Description>
			</Card.Header>
			<Card.Content class="space-y-4">
				<div class="space-y-2">
					<Label for="remote-username">Username</Label>
					<Input
						id="remote-username"
						bind:value={remoteUsername}
						placeholder="popcorn-friend"
					/>
				</div>
				<div class="space-y-2">
					<Label for="remote-ticket">Code</Label>
					<Textarea
						id="remote-ticket"
						bind:value={remoteTicket}
						placeholder={m.connect_join_placeholder()}
						class="font-mono text-xs resize-none h-20"
					/>
				</div>
				<Button
					class="w-full cursor-pointer"
					onclick={connectToPeer}
					disabled={isConnecting || !remoteTicket.trim()}
				>
					{#if isConnecting}
						<Loader2 class="size-4 animate-spin mr-2" />
						{m.connect_join_connecting()}
					{:else}
						{m.connect_join_btn()}
					{/if}
				</Button>
			</Card.Content>
		</Card.Root>
	</div>

	<Separator />

	<!-- Connected Peers List -->
	<div class="space-y-4">
		<h2 class="text-xl font-semibold">{m.connect_peers_title()}</h2>

		{#if $peers.length === 0}
			<Card.Root>
				<Card.Content class="flex flex-col items-center justify-center py-12 text-center">
					<WifiOff class="size-10 text-muted-foreground/50 mb-3" />
					<p class="text-muted-foreground text-sm">{m.connect_peers_empty()}</p>
				</Card.Content>
			</Card.Root>
		{:else}
			<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
				{#each $peers as peer (peer.node_id)}
					<Card.Root class="relative">
						<Card.Header class="pb-3">
							<div class="flex items-center justify-between">
								<div class="flex items-center gap-3">
									<!-- Status indicator dot -->
									<span
										class="inline-block size-3 rounded-full shrink-0 {peer.online ? 'bg-green-500 shadow-green-500/50 shadow-sm' : 'bg-gray-400'}"
									></span>
									<Card.Title class="text-base">{peer.username}</Card.Title>
								</div>
								<Badge variant={peer.online ? "default" : "secondary"}>
									{#if peer.online}
										<Wifi class="size-3 mr-1" />
										{m.connect_peers_online()}
									{:else}
										<WifiOff class="size-3 mr-1" />
										{m.connect_peers_offline()}
									{/if}
								</Badge>
							</div>
						</Card.Header>
						<Card.Content class="space-y-1 text-sm text-muted-foreground">
							<p class="font-mono text-xs truncate" title={peer.node_id}>
								{peer.node_id.slice(0, 16)}…
							</p>
							{#if peer.rtt_ms != null}
								<p>{m.connect_peers_rtt({ rtt: peer.rtt_ms.toFixed(1) })}</p>
							{/if}
							{#if peer.last_seen}
								<p>{m.connect_peers_last_seen({ time: formatLastSeen(peer.last_seen) })}</p>
							{/if}
						</Card.Content>
						<Card.Footer>
							<Button
								variant="destructive"
								size="sm"
								class="w-full cursor-pointer"
								onclick={() => disconnectPeer(peer.node_id)}
							>
								<Trash2 class="size-3.5 mr-1.5" />
								{m.connect_peers_disconnect()}
							</Button>
						</Card.Footer>
					</Card.Root>
				{/each}
			</div>
		{/if}
	</div>

	{/if}
</div>
