import { redirect } from '@sveltejs/kit';
import { loadedSaveFile } from '$lib/store.svelte';
import type { LayoutLoad } from './$types';

export const ssr = false;

export const load: LayoutLoad = ({ route }) => {
  if (route.id !== '/select' && !loadedSaveFile.current) {
    redirect(307, '/select');
  }
};
