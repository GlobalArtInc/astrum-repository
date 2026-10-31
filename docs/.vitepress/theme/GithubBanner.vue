<script setup lang="ts">
import { dismiss, REPOSITORY_URL } from './banner';
</script>

<template>
  <div class="github-banner">
    <a class="github-banner-link" :href="REPOSITORY_URL" target="_blank" rel="noopener">Astrum is open source. Star it on <strong>GitHub</strong></a>
    <button type="button" class="github-banner-close" aria-label="Dismiss" @click="dismiss">
      <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
        <line x1="2" y1="2" x2="12" y2="12" />
        <line x1="12" y1="2" x2="2" y2="12" />
      </svg>
    </button>
  </div>
</template>
