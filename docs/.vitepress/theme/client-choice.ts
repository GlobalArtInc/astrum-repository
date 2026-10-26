import { ref } from 'vue';

export type ClientId = 'npm' | 'docker';

export interface Client {
  id: ClientId;
  label: string;
}

export const CLIENTS: readonly Client[] = [
  { id: 'npm', label: 'npm' },
  { id: 'docker', label: 'Docker' },
];

const STORAGE_KEY = 'astrum-docs-client';

export const choice = ref<ClientId>('npm');

function known(value: string | null): value is ClientId {
  return CLIENTS.some((client) => client.id === value);
}

export function choose(id: ClientId): void {
  choice.value = id;
  try {
    window.localStorage.setItem(STORAGE_KEY, id);
  } catch {
    return;
  }
}

export function restore(): void {
  const asked = new URLSearchParams(window.location.search).get('client');
  if (known(asked)) {
    choose(asked);
    return;
  }
  try {
    const saved = window.localStorage.getItem(STORAGE_KEY);
    if (known(saved)) {
      choice.value = saved;
    }
  } catch {
    return;
  }
}
