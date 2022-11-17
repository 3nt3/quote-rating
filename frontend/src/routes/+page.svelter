<script lang="ts">
	import Button, { Label, Icon } from '@smui/button';

	let clicked = 0;

	function handleClick(event: CustomEvent | MouseEvent) {
		event = event as MouseEvent;
		if (event.button === 0) {
			clicked++;
		}
	}

	function reset() {
		clicked = 0;
	}
</script>

<h1 class="mdc-typography--headline3"><a href="/vote">Vote</a></h1>
<h1 class="mdc-typography--headline3"><a href="/leaderboard">Leaderboard</a></h1>
<p class="mdc-typography--body1">I'm sorry everything's ugly but I am way too lazy to fix it</p>

<style>
	a {
		text-decoration: none;
	}
</style>
