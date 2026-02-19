<script lang="ts">
	import { Camera, KeyRound, Pencil, RefreshCw, Trash2, User } from "@lucide/svelte";
	import { convertFileSrc, invoke } from "@tauri-apps/api/core";
	import { info, warn } from "@tauri-apps/plugin-log";
	import { onMount } from "svelte";
	import { toast } from "svelte-sonner";
	import { Button } from "$lib/components/ui/button";
	import * as Card from "$lib/components/ui/card";
	import * as Dialog from "$lib/components/ui/dialog";
	import { Input } from "$lib/components/ui/input";
	import * as InputOTP from "$lib/components/ui/input-otp";
	import { Label } from "$lib/components/ui/label";
	import { Separator } from "$lib/components/ui/separator";
	import { m } from "$lib/paraglide/messages.js";
	import { getLocale } from "$lib/paraglide/runtime.js";
	import { currentUser, notifyAvatarChanged } from "$lib/stores/user";

	let username = $state("");
	let originalUsername = $state("");
	let isUpdatingUsername = $state(false);
	let showPinDialog = $state(false);
	let pendingAction = $state<"username" | null>(null);
	let confirmPin = $state("");

	// Avatar
	let avatarUrl = $state<string | null>(null);
	let isLoadingAvatar = $state(true);

	// Change PIN
	let currentPin = $state("");
	let newPin = $state("");
	let confirmNewPin = $state("");
	let isChangingPin = $state(false);
	let pinError = $state("");

	onMount(async () => {
		if ($currentUser) {
			username = $currentUser.username;
			originalUsername = $currentUser.username;
			await loadAvatar();
		}
	});

	async function loadAvatar() {
		try {
			const userId = $currentUser?.id;
			if (!userId) return;
			const path = await invoke<string | null>("get_avatar", { userId });
			if (path) {
				// Add cache-busting query param
				avatarUrl = convertFileSrc(path) + "?t=" + Date.now();
			} else {
				avatarUrl = null;
			}
		} catch (error) {
			warn("[Account] Failed to load avatar: " + String(error));
		} finally {
			isLoadingAvatar = false;
		}
	}

	async function handleChangeAvatar() {
		try {
			const userId = $currentUser?.id;
			if (!userId) return;

			const picked = await invoke<string | null>("pick_image");
			if (!picked) return;

			const savedPath = await invoke<string>("save_avatar", { userId, sourcePath: picked });
			avatarUrl = convertFileSrc(savedPath) + "?t=" + Date.now();
			notifyAvatarChanged();
			toast.success(m.account_avatar_updated());
			info("[Account] Avatar updated");
		} catch (error) {
			toast.error(m.account_error(), { description: String(error) });
			warn("[Account] Failed to change avatar: " + String(error));
		}
	}

	async function handleRemoveAvatar() {
		try {
			const userId = $currentUser?.id;
			if (!userId) return;

			await invoke("remove_avatar", { userId });
			avatarUrl = null;
			notifyAvatarChanged();
			toast.success(m.account_avatar_removed());
			info("[Account] Avatar removed");
		} catch (error) {
			toast.error(m.account_error(), { description: String(error) });
			warn("[Account] Failed to remove avatar: " + String(error));
		}
	}

	function requestUsernameUpdate() {
		if (username === originalUsername) return;
		pendingAction = "username";
		confirmPin = "";
		showPinDialog = true;
	}

	async function handleConfirmPin() {
		if (confirmPin.length !== 4) return;

		if (pendingAction === "username") {
			await updateUsername(confirmPin);
		}

		showPinDialog = false;
		confirmPin = "";
		pendingAction = null;
	}

	async function updateUsername(pin: string) {
		isUpdatingUsername = true;
		try {
			const userId = $currentUser?.id;
			if (!userId) return;

			const updated = await invoke<{ id: string; username: string; created_at: string }>(
				"update_username",
				{ userId, newUsername: username, pin }
			);

			currentUser.set({
				id: updated.id,
				username: updated.username,
				created_at: updated.created_at,
			});

			originalUsername = updated.username;
			toast.success(m.account_username_updated());
			info("[Account] Username updated to: " + updated.username);
		} catch (error) {
			toast.error(m.account_error(), { description: String(error) });
			warn("[Account] Failed to update username: " + String(error));
			username = originalUsername;
		} finally {
			isUpdatingUsername = false;
		}
	}

	async function handleChangePin() {
		pinError = "";

		if (currentPin.length !== 4 || newPin.length !== 4 || confirmNewPin.length !== 4) {
			pinError = m.user_error_pin_incomplete();
			return;
		}

		if (newPin !== confirmNewPin) {
			pinError = m.account_pin_mismatch();
			return;
		}

		isChangingPin = true;
		try {
			const userId = $currentUser?.id;
			if (!userId) return;

			await invoke("update_pin", { userId, currentPin, newPin });

			toast.success(m.account_pin_updated());
			info("[Account] PIN updated");
			currentPin = "";
			newPin = "";
			confirmNewPin = "";
		} catch (error) {
			toast.error(m.account_error(), { description: String(error) });
			warn("[Account] Failed to update PIN: " + String(error));
		} finally {
			isChangingPin = false;
		}
	}

	function formatDate(isoString: string): string {
		try {
			return new Date(isoString).toLocaleDateString(getLocale(), {
				year: "numeric",
				month: "long",
				day: "numeric",
			});
		} catch {
			return isoString;
		}
	}
</script>

<div class="mx-auto max-w-2xl p-6">
	<h1 class="mb-6 text-2xl font-bold">{m.account_title()}</h1>

	<!-- Profile Card: Avatar + Username -->
	<Card.Root>
		<Card.Header>
			<Card.Title class="flex items-center gap-2">
				<User class="size-5" />
				{m.account_profile_title()}
			</Card.Title>
			<Card.Description>
				{m.account_profile_description()}
			</Card.Description>
		</Card.Header>
		<Card.Content>
			<div class="space-y-6">
				<!-- Avatar Section -->
				<div class="space-y-2">
					<Label>{m.account_avatar()}</Label>
					<div class="flex items-center gap-4">
						<!-- Avatar preview -->
						<div class="relative size-20 shrink-0 rounded-full bg-muted flex items-center justify-center overflow-hidden border-2 border-border">
							{#if isLoadingAvatar}
								<RefreshCw class="size-5 animate-spin text-muted-foreground" />
							{:else if avatarUrl}
								<img
									src={avatarUrl}
									alt="Avatar"
									class="size-full object-cover"
								/>
							{:else}
								<User class="size-8 text-muted-foreground" />
							{/if}
						</div>

						<!-- Avatar actions -->
						<div class="flex flex-col gap-2">
							<Button variant="outline" size="sm" onclick={handleChangeAvatar}>
								<Camera class="size-4" />
								{m.account_avatar_change()}
							</Button>
							{#if avatarUrl}
								<Button
									variant="ghost"
									size="sm"
									class="text-destructive hover:bg-destructive/10 hover:text-destructive"
									onclick={handleRemoveAvatar}
								>
									<Trash2 class="size-4" />
									{m.account_avatar_remove()}
								</Button>
							{/if}
							<p class="text-xs text-muted-foreground">
								{m.account_avatar_hint()}
							</p>
						</div>
					</div>
				</div>

				<Separator />

				<!-- Username Section -->
				<div class="space-y-2">
					<Label for="username">{m.account_username()}</Label>
					<div class="flex items-center gap-2">
						<Input
							id="username"
							placeholder={m.account_username_placeholder()}
							bind:value={username}
							class="max-w-xs"
						/>
						<Button
							variant="default"
							size="sm"
							disabled={username === originalUsername || isUpdatingUsername || !username.trim()}
							onclick={requestUsernameUpdate}
						>
							{#if isUpdatingUsername}
								<RefreshCw class="size-4 animate-spin" />
							{:else}
								<Pencil class="size-4" />
							{/if}
							{m.account_username_save()}
						</Button>
					</div>
				</div>
			</div>
		</Card.Content>
	</Card.Root>

	<!-- Security Card: Change PIN -->
	<Card.Root class="mt-4">
		<Card.Header>
			<Card.Title class="flex items-center gap-2">
				<KeyRound class="size-5" />
				{m.account_security_title()}
			</Card.Title>
			<Card.Description>
				{m.account_security_description()}
			</Card.Description>
		</Card.Header>
		<Card.Content>
			<div class="space-y-4">
				<!-- Current PIN -->
				<div class="flex flex-col items-start space-y-2">
					<Label>{m.account_pin_current()}</Label>
					<InputOTP.Root
						maxlength={4}
						bind:value={currentPin}
					>
						{#snippet children({ cells })}
							<InputOTP.Group>
								{#each cells as cell (cell)}
									<InputOTP.Slot {cell} />
								{/each}
							</InputOTP.Group>
						{/snippet}
					</InputOTP.Root>
				</div>

				<Separator />

				<!-- New PIN -->
				<div class="flex flex-col items-start space-y-2">
					<Label>{m.account_pin_new()}</Label>
					<InputOTP.Root
						maxlength={4}
						bind:value={newPin}
					>
						{#snippet children({ cells })}
							<InputOTP.Group>
								{#each cells as cell (cell)}
									<InputOTP.Slot {cell} />
								{/each}
							</InputOTP.Group>
						{/snippet}
					</InputOTP.Root>
				</div>

				<!-- Confirm New PIN -->
				<div class="flex flex-col items-start space-y-2">
					<Label>{m.account_pin_confirm()}</Label>
					<InputOTP.Root
						maxlength={4}
						bind:value={confirmNewPin}
					>
						{#snippet children({ cells })}
							<InputOTP.Group>
								{#each cells as cell (cell)}
									<InputOTP.Slot {cell} />
								{/each}
							</InputOTP.Group>
						{/snippet}
					</InputOTP.Root>
				</div>

				{#if pinError}
					<p class="text-sm text-destructive">{pinError}</p>
				{/if}

				<Button
					variant="default"
					disabled={isChangingPin || currentPin.length !== 4 || newPin.length !== 4 || confirmNewPin.length !== 4}
					onclick={handleChangePin}
				>
					{#if isChangingPin}
						<RefreshCw class="size-4 animate-spin" />
					{/if}
					{m.account_pin_change()}
				</Button>
			</div>
		</Card.Content>
	</Card.Root>

	<!-- Account Info Card -->
	<Card.Root class="mt-4">
		<Card.Header>
			<Card.Title class="flex items-center gap-2">
				<User class="size-5" />
				{m.account_info_title()}
			</Card.Title>
		</Card.Header>
		<Card.Content>
			<div class="space-y-3">
				<div class="flex items-center justify-between">
					<span class="text-sm text-muted-foreground">{m.account_info_id()}</span>
					<span class="text-sm font-mono select-all">{$currentUser?.id ?? ""}</span>
				</div>
				<Separator />
				<div class="flex items-center justify-between">
					<span class="text-sm text-muted-foreground">{m.account_info_created()}</span>
					<span class="text-sm">{$currentUser?.created_at ? formatDate($currentUser.created_at) : ""}</span>
				</div>
			</div>
		</Card.Content>
	</Card.Root>
</div>

<!-- PIN confirmation dialog for username change -->
<Dialog.Root
	open={showPinDialog}
	onOpenChange={(open) => {
		if (!open) {
			showPinDialog = false;
			confirmPin = "";
			pendingAction = null;
		}
	}}
>
	<Dialog.Content class="sm:max-w-sm">
		<Dialog.Header>
			<Dialog.Title>{m.user_label_pin()}</Dialog.Title>
			<Dialog.Description>
				{m.account_pin_required()}
			</Dialog.Description>
		</Dialog.Header>
		<div class="flex flex-col items-center py-4">
			<InputOTP.Root
				maxlength={4}
				bind:value={confirmPin}
				onComplete={handleConfirmPin}
			>
				{#snippet children({ cells })}
					<InputOTP.Group>
						{#each cells as cell (cell)}
							<InputOTP.Slot {cell} />
						{/each}
					</InputOTP.Group>
				{/snippet}
			</InputOTP.Root>
		</div>
		<Dialog.Footer>
			<Button
				variant="outline"
				onclick={() => {
					showPinDialog = false;
					confirmPin = "";
					pendingAction = null;
				}}
			>
				Cancel
			</Button>
			<Button
				variant="default"
				disabled={confirmPin.length !== 4}
				onclick={handleConfirmPin}
			>
				OK
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
