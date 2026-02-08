<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";
	import { toast } from "svelte-sonner";
	import { Button } from "$lib/components/ui/button";
	import * as Card from "$lib/components/ui/card";
	import { Input } from "$lib/components/ui/input";
	import * as InputOTP from "$lib/components/ui/input-otp";
	import { Label } from "$lib/components/ui/label";
	import { m } from "$lib/paraglide/messages.js";
	import { currentUser, type UserProfile } from "$lib/stores/user";

	let username = $state("");
	let pin = $state("");
	let usernameError = $state("");
	let pinError = $state("");
	let isSubmitting = $state(false);

	const USERNAME_REGEX = /^[a-zA-Z0-9\-_]+$/;

	function validateUsername(value: string): boolean {
		if (!value.trim()) {
			usernameError = m.user_error_username_required();
			return false;
		}
		if (!USERNAME_REGEX.test(value)) {
			usernameError = m.user_error_username_invalid();
			return false;
		}
		usernameError = "";
		return true;
	}

	function validatePin(value: string): boolean {
		if (value.length !== 4) {
			pinError = m.user_error_pin_incomplete();
			return false;
		}
		pinError = "";
		return true;
	}

	async function handleSubmit() {
		const isUsernameValid = validateUsername(username);
		const isPinValid = validatePin(pin);

		if (!isUsernameValid || !isPinValid) return;

		isSubmitting = true;

		try {
			const profile = await invoke<UserProfile>("create_user", {
				username: username.trim(),
				pin,
			});
			toast.success(m.user_success_created());
			currentUser.set(profile);
		} catch (error) {
			toast.error(m.user_error_creation_failed(), {
				description: String(error),
			});
		} finally {
			isSubmitting = false;
		}
	}
</script>

<div class="flex min-h-screen items-center justify-center bg-background p-4">
	<div class="flex w-full max-w-sm flex-col items-center gap-6">
		<img src="/logo-circle.svg" alt="Popcorn Hero" class="size-24" />
		<Card.Root class="w-full">
		<Card.Header class="text-center">
			<Card.Title class="text-2xl">{m.user_create_title()}</Card.Title>
			<Card.Description>{m.user_create_description()}</Card.Description>
		</Card.Header>
		<Card.Content>
			<form
				onsubmit={(e) => {
					e.preventDefault();
					handleSubmit();
				}}
				class="space-y-6"
			>
				<!-- Username -->
				<div class="space-y-2">
					<Label for="username">{m.user_label_username()}</Label>
					<Input
						id="username"
						type="text"
						placeholder={m.user_placeholder_username()}
						bind:value={username}
						oninput={() => {
							if (usernameError) validateUsername(username);
						}}
						aria-invalid={usernameError ? true : undefined}
					/>
					{#if usernameError}
						<p class="text-sm text-destructive">{usernameError}</p>
					{/if}
				</div>

				<!-- PIN -->
				<div class="flex flex-col items-center space-y-2">
					<Label>{m.user_label_pin()}</Label>
					<InputOTP.Root
						maxlength={4}
						bind:value={pin}
						onComplete={() => {
							if (pinError) validatePin(pin);
						}}
					>
						{#snippet children({ cells })}
							<InputOTP.Group>
								{#each cells as cell (cell)}
									<InputOTP.Slot {cell} />
								{/each}
							</InputOTP.Group>
						{/snippet}
					</InputOTP.Root>
					{#if pinError}
						<p class="text-sm text-destructive">{pinError}</p>
					{/if}
				</div>

				<Button type="submit" class="w-full" disabled={isSubmitting}>
					{m.user_btn_create()}
				</Button>
			</form>
		</Card.Content>
	</Card.Root>
	</div>
</div>
