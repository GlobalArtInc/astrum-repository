<script setup lang="ts">
import { nextTick, onMounted, useTemplateRef } from 'vue';
import { choice, choose, CLIENTS, restore } from './client-choice';

const tabs = useTemplateRef<HTMLButtonElement[]>('tabs');

onMounted(restore);

async function move(event: KeyboardEvent, index: number) {
  const step = event.key === 'ArrowRight' ? 1 : event.key === 'ArrowLeft' ? -1 : 0;
  if (step === 0) {
    return;
  }
  event.preventDefault();
  const next = (index + step + CLIENTS.length) % CLIENTS.length;
  choose(CLIENTS[next].id);
  await nextTick();
  tabs.value?.[next]?.focus();
}
</script>

<template>
  <div class="client-switch">
    <div class="client-tabs" role="tablist" aria-label="Client">
      <button
        v-for="(client, index) in CLIENTS"
        ref="tabs"
        :key="client.id"
        type="button"
        role="tab"
        :aria-selected="choice === client.id"
        :tabindex="choice === client.id ? 0 : -1"
        @click="choose(client.id)"
        @keydown="move($event, index)"
      >
        {{ client.label }}
      </button>
    </div>
    <div
      v-for="client in CLIENTS"
      :key="client.id"
      role="tabpanel"
      :aria-label="client.label"
      :hidden="choice !== client.id"
    >
      <slot :name="client.id" />
    </div>
  </div>
</template>

<style scoped>
.client-switch {
  margin: 16px 0;
}

.client-tabs {
  display: flex;
  gap: 4px;
  border-bottom: 1px solid var(--vp-c-divider);
}

.client-tabs button {
  margin-bottom: -1px;
  padding: 8px 16px;
  border: 0;
  border-bottom: 2px solid transparent;
  background: none;
  color: var(--vp-c-text-2);
  font: inherit;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
}

.client-tabs button:hover {
  color: var(--vp-c-text-1);
}

.client-tabs button[aria-selected='true'] {
  border-bottom-color: var(--vp-c-brand-1);
  color: var(--vp-c-brand-1);
}

.client-tabs button:focus-visible {
  outline: 2px solid var(--vp-c-brand-1);
  outline-offset: -2px;
  border-radius: 4px;
}

[role='tabpanel'] {
  padding-top: 8px;
}

[role='tabpanel'][hidden] {
  display: none;
}
</style>
