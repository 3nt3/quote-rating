<script lang="ts">
	import Button from '@smui/button';
	import TopAppBar, { Row, Section, Title, AutoAdjust } from '@smui/top-app-bar';
	import IconButton from '@smui/icon-button';
	import { Label, Icon } from '@smui/common';
	import { Svg } from '@smui/common';
	import { mdiGithub, mdiWeb } from '@mdi/js';

	let topAppBar: any;

	let lightTheme =
		typeof window === 'undefined' || window.matchMedia('(prefers-color-scheme: light)').matches;
	function switchTheme() {
		lightTheme = !lightTheme;
		let themeLink = document.head.querySelector<HTMLLinkElement>('#theme');
		if (!themeLink) {
			themeLink = document.createElement('link');
			themeLink.rel = 'stylesheet';
			themeLink.id = 'theme';
		}
		themeLink.href = `/smui${lightTheme ? '' : '-dark'}.css`;
		document.head
			.querySelector<HTMLLinkElement>('link[href$="/smui-dark.css"]')
			?.insertAdjacentElement('afterend', themeLink);
	}
</script>

<TopAppBar bind:this={topAppBar} variant="standard">
	<Row>
		<Section>
			<Title
				><a href="/" class="mdc-typography--headline5 link"
					>#krasse-zitate-von-menschen geile app dazu</a
				></Title
			>
		</Section>
		<Section align="end" toolbar>
			<Button on:click={switchTheme}>
				<Label>{lightTheme ? 'Lights off' : 'Lights on'}</Label>
			</Button>
			<IconButton aria-label="GitHub" href="https://github.com/3nt3/quote-rating">
				<Icon component={Svg} viewBox="0 0 24 24">
					<path fill="currentColor" d={mdiGithub} />
				</Icon>
			</IconButton>
		</Section>
	</Row>
</TopAppBar>

<AutoAdjust {topAppBar} style="display: flex; justify-content: space-between;">
	<div class="container"><slot /></div>
</AutoAdjust>

<style>
	.link {
		color: var(--mdc-theme-on-primary);
	}

	.container {
		width: 100vw;
	}
</style>
