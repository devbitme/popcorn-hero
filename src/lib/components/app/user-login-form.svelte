<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";
	import { toast } from "svelte-sonner";
	import * as Card from "$lib/components/ui/card";
	import * as InputOTP from "$lib/components/ui/input-otp";
	import { m } from "$lib/paraglide/messages.js";
	import { currentUser, type UserProfile } from "$lib/stores/user";

	interface Props {
		userId: string;
		username: string;
	}

	let { userId, username }: Props = $props();

	let pin = $state("");
	let pinError = $state("");
	let isSubmitting = $state(false);

	async function handleSubmit() {
		if (pin.length !== 4) {
			pinError = m.user_error_pin_incomplete();
			return;
		}

		pinError = "";
		isSubmitting = true;

		try {
			const profile = await invoke<UserProfile>("verify_pin", {
				userId,
				pin,
			});
			toast.success(m.user_success_unlocked());
			currentUser.set(profile);
		} catch (_error) {
			pinError = m.user_error_pin_invalid();
			pin = "";
		} finally {
			isSubmitting = false;
		}
	}
</script>

<div class="flex min-h-screen items-center justify-center bg-background p-4">
	<Card.Root class="w-full max-w-sm">
		<Card.Header class="text-center">
			<Card.Title class="text-2xl">{m.user_login_title({ username })}</Card.Title>
			<Card.Description>{m.user_login_description()}</Card.Description>
		</Card.Header>
		<Card.Content>
			<form
				onsubmit={(e) => {
					e.preventDefault();
					handleSubmit();
				}}
				class="space-y-6"
			>
				<!-- PIN -->
				<div class="space-y-2">
					<div class="flex justify-center">
						<InputOTP.Root
							maxlength={4}
							bind:value={pin}
							onComplete={handleSubmit}
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
						<p class="text-sm text-destructive text-center">{pinError}</p>
					{/if}
				</div>

			</form>
		</Card.Content>
	</Card.Root>
</div>
