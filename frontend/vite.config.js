import { sveltekit } from '@sveltejs/kit/vite';

/** @type {import('vite').UserConfig} */
const config = {
	plugins: [sveltekit()],
    kit: {
        vite: {
            build: {
                sourcemap: true,
            }
        }
    }
};

export default config;
