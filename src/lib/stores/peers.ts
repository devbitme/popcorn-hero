import { writable } from "svelte/store";

export interface PeerInfo {
	node_id: string;
	username: string;
	online: boolean;
	rtt_ms: number | null;
	last_seen: number | null;
}

export const peers = writable<PeerInfo[]>([]);
export const localNodeId = writable<string | null>(null);
export const isPeerEndpointRunning = writable(false);
