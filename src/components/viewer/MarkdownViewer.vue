<script setup lang="ts">
import { ref, watch, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { marked } from 'marked';
import {
  XMarkIcon,
  DocumentTextIcon,
  GlobeAltIcon,
  ArrowTopRightOnSquareIcon,
} from '@heroicons/vue/24/outline';
import { openUrl } from '@tauri-apps/plugin-opener';

const props = defineProps<{
  source?: string; // file path or URL
}>();

defineEmits<{
  (e: 'close'): void;
}>();

const content = ref('');
const renderedHtml = ref('');
const isLoading = ref(false);
const error = ref<string | null>(null);
const title = ref('');
const viewerWidth = ref(380);

// Resize drag handling
let isDragging = false;
let startX = 0;
let startWidth = 0;

function startResize(e: MouseEvent) {
  isDragging = true;
  startX = e.clientX;
  startWidth = viewerWidth.value;
  document.addEventListener('mousemove', onResize);
  document.addEventListener('mouseup', stopResize);
  document.body.style.cursor = 'col-resize';
  document.body.style.userSelect = 'none';
}

function onResize(e: MouseEvent) {
  if (!isDragging) return;
  const delta = startX - e.clientX; // dragging left = wider
  viewerWidth.value = Math.max(250, Math.min(window.innerWidth * 0.6, startWidth + delta));
}

function stopResize() {
  isDragging = false;
  document.removeEventListener('mousemove', onResize);
  document.removeEventListener('mouseup', stopResize);
  document.body.style.cursor = '';
  document.body.style.userSelect = '';
}

const isUrl = computed(() => {
  if (!props.source) return false;
  return props.source.startsWith('http://') || props.source.startsWith('https://');
});

const sourceIcon = computed(() => isUrl.value ? GlobeAltIcon : DocumentTextIcon);

async function loadContent(source: string) {
  isLoading.value = true;
  error.value = null;
  content.value = '';
  renderedHtml.value = '';

  // Set title from filename or URL
  if (isUrl.value) {
    try {
      const url = new URL(source);
      title.value = url.pathname.split('/').pop() || url.hostname;
    } catch {
      title.value = source;
    }
  } else {
    title.value = source.split(/[/\\]/).pop() || source;
  }

  try {
    if (isUrl.value) {
      content.value = await invoke<string>('fetch_url', { url: source });
    } else {
      content.value = await invoke<string>('read_text_file', { path: source });
    }

    renderedHtml.value = await marked(content.value, {
      breaks: true,
      gfm: true,
    });
  } catch (e) {
    error.value = `${e}`;
  } finally {
    isLoading.value = false;
  }
}

function handleLinkClick(e: MouseEvent) {
  const target = e.target as HTMLElement;
  const anchor = target.closest('a');
  if (anchor?.href) {
    e.preventDefault();
    const href = anchor.getAttribute('href') || '';
    // If it's a relative link and we loaded from a URL, resolve it
    if (href.startsWith('http://') || href.startsWith('https://')) {
      openUrl(href).catch(() => {});
    }
  }
}

function openExternal() {
  if (!props.source) return;
  if (isUrl.value) {
    openUrl(props.source).catch(() => {});
  }
}

watch(() => props.source, (source) => {
  if (source) {
    loadContent(source);
  }
}, { immediate: true });
</script>

<template>
  <div class="md-viewer" :style="{ width: viewerWidth + 'px' }">
    <!-- Resize handle -->
    <div class="md-resize-handle" @mousedown="startResize"></div>
    <!-- Header -->
    <div class="md-header">
      <div class="md-title">
        <component :is="sourceIcon" class="w-3.5 h-3.5" style="color: var(--accent-cyan);" />
        <span class="md-title-text">{{ title }}</span>
      </div>
      <div class="md-actions">
        <button
          v-if="isUrl"
          class="md-btn"
          @click="openExternal"
          title="Open in browser"
        >
          <ArrowTopRightOnSquareIcon class="w-3.5 h-3.5" />
        </button>
        <button class="md-btn md-btn-close" @click="$emit('close')" title="Close">
          <XMarkIcon class="w-3.5 h-3.5" />
        </button>
      </div>
    </div>

    <!-- Content -->
    <div class="md-body">
      <div v-if="isLoading" class="md-loading">
        <span class="text-label">Loading...</span>
      </div>
      <div v-else-if="error" class="md-error">
        <span>{{ error }}</span>
      </div>
      <div
        v-else
        class="md-content"
        v-html="renderedHtml"
        @click="handleLinkClick"
      ></div>
    </div>
  </div>
</template>

<style scoped>
.md-viewer {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--bg-secondary);
  border-left: 1px solid var(--border-default);
  position: relative;
  flex-shrink: 0;
}

.md-resize-handle {
  position: absolute;
  left: -3px;
  top: 0;
  bottom: 0;
  width: 6px;
  cursor: col-resize;
  z-index: 20;
}

.md-resize-handle:hover,
.md-resize-handle:active {
  background: var(--accent-cyan);
  opacity: 0.3;
}

.md-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  border-bottom: 1px solid var(--border-subtle);
  flex-shrink: 0;
}

.md-title {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.md-title-text {
  font-size: 0.7rem;
  font-weight: 600;
  letter-spacing: 0.05em;
  text-transform: uppercase;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.md-actions {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
}

.md-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  background: transparent;
  border: 1px solid transparent;
  color: var(--text-muted);
  cursor: pointer;
  transition: all 0.1s ease;
}

.md-btn:hover {
  color: var(--text-secondary);
  border-color: var(--border-default);
}

.md-btn-close:hover {
  color: var(--accent-red);
  border-color: var(--accent-red);
}

.md-body {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
  min-height: 0;
}

.md-loading,
.md-error {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--text-muted);
  font-size: 0.75rem;
}

.md-error {
  color: var(--accent-red);
}

/* Markdown content styling */
.md-content {
  font-family: var(--font-sans), sans-serif;
  font-size: 0.8rem;
  line-height: 1.6;
  color: var(--text-primary);
}

.md-content :deep(h1) {
  font-size: 1.3rem;
  font-weight: 700;
  margin: 0 0 12px 0;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--border-subtle);
  color: var(--text-primary);
}

.md-content :deep(h2) {
  font-size: 1.05rem;
  font-weight: 700;
  margin: 20px 0 8px 0;
  padding-bottom: 6px;
  border-bottom: 1px solid var(--border-subtle);
  color: var(--text-primary);
}

.md-content :deep(h3) {
  font-size: 0.9rem;
  font-weight: 600;
  margin: 16px 0 6px 0;
  color: var(--text-primary);
}

.md-content :deep(h4),
.md-content :deep(h5),
.md-content :deep(h6) {
  font-size: 0.8rem;
  font-weight: 600;
  margin: 12px 0 4px 0;
  color: var(--text-secondary);
}

.md-content :deep(p) {
  margin: 0 0 10px 0;
}

.md-content :deep(ul),
.md-content :deep(ol) {
  margin: 0 0 10px 0;
  padding-left: 20px;
}

.md-content :deep(li) {
  margin: 2px 0;
}

.md-content :deep(li input[type="checkbox"]) {
  margin-right: 6px;
}

.md-content :deep(a) {
  color: var(--accent-cyan);
  text-decoration: none;
}

.md-content :deep(a:hover) {
  text-decoration: underline;
}

.md-content :deep(code) {
  font-family: var(--font-mono);
  font-size: 0.75rem;
  background: var(--bg-tertiary);
  padding: 2px 5px;
  border-radius: 2px;
  color: var(--accent-cyan);
}

.md-content :deep(pre) {
  background: var(--bg-primary);
  border: 1px solid var(--border-subtle);
  padding: 10px 12px;
  margin: 0 0 10px 0;
  overflow-x: auto;
}

.md-content :deep(pre code) {
  background: none;
  padding: 0;
  color: var(--text-primary);
}

.md-content :deep(blockquote) {
  margin: 0 0 10px 0;
  padding: 4px 12px;
  border-left: 3px solid var(--accent-cyan);
  color: var(--text-secondary);
  background: var(--bg-tertiary);
}

.md-content :deep(table) {
  width: 100%;
  border-collapse: collapse;
  margin: 0 0 10px 0;
  font-size: 0.75rem;
}

.md-content :deep(th),
.md-content :deep(td) {
  padding: 6px 10px;
  border: 1px solid var(--border-default);
  text-align: left;
}

.md-content :deep(th) {
  background: var(--bg-tertiary);
  font-weight: 600;
  color: var(--text-primary);
}

.md-content :deep(hr) {
  border: none;
  border-top: 1px solid var(--border-subtle);
  margin: 16px 0;
}

.md-content :deep(img) {
  max-width: 100%;
  border-radius: 4px;
}

.md-content :deep(strong) {
  font-weight: 600;
  color: var(--text-primary);
}
</style>
