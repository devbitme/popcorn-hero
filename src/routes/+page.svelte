<script lang="ts">
	import { m } from "$lib/paraglide/messages.js";
</script>

<main class="h-screen flex items-center justify-center bg-carbon-50">
	<div class="text-center">
		<h1 class="sr-only">Popcorn Hero</h1>
		<img src="/logo.png" alt="Popcorn Hero Logo" class="w-32 h-32" />
		<br />
		{m.hello_world({ name: "Silbad" })}
	</div>
</main>
