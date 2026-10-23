// @ts-check
import { defineConfig } from 'astro/config';
import tailwind from '@astrojs/tailwind';

// https://astro.build/config
export default defineConfig({
  site: 'https://kurarensu16.github.io',
  base: '/TIDE',
  integrations: [tailwind()],
});
