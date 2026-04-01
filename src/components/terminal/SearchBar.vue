<script setup lang="ts">
import { ref, onMounted, nextTick, watch } from 'vue';
import {
  ChevronUpIcon,
  ChevronDownIcon,
  XMarkIcon,
} from '@heroicons/vue/24/outline';

const props = defineProps<{
  visible: boolean;
  resultCount?: number;
  resultIndex?: number;
}>();

const emit = defineEmits<{
  (e: 'search', query: string, options: { caseSensitive: boolean; regex: boolean }): void;
  (e: 'next'): void;
  (e: 'previous'): void;
  (e: 'close'): void;
}>();

const searchInput = ref<HTMLInputElement>();
const query = ref('');
const caseSensitive = ref(false);
const regex = ref(false);

function doSearch() {
  emit('search', query.value, {
    caseSensitive: caseSensitive.value,
    regex: regex.value,
  });
}

function toggleCaseSensitive() {
  caseSensitive.value = !caseSensitive.value;
  doSearch();
}

function toggleRegex() {
  regex.value = !regex.value;
  doSearch();
}

function handleKeydown(e: KeyboardEvent) {
  if (e.code === 'Escape') {
    e.preventDefault();
    e.stopPropagation();
    emit('close');
  } else if (e.code === 'Enter') {
    e.preventDefault();
    if (e.shiftKey) {
      emit('previous');
    } else {
      emit('next');
    }
  }
}

watch(query, () => doSearch());

watch(() => props.visible, (visible) => {
  if (visible) {
    nextTick(() => {
      searchInput.value?.focus();
      searchInput.value?.select();
    });
  }
});

onMounted(() => {
  if (props.visible) {
    nextTick(() => {
      searchInput.value?.focus();
    });
  }
});
</script>

<template>
  <Transition
    enter-active-class="transition duration-100 ease-out"
    enter-from-class="opacity-0 -translate-y-1"
    enter-to-class="opacity-100 translate-y-0"
    leave-active-class="transition duration-75 ease-in"
    leave-from-class="opacity-100 translate-y-0"
    leave-to-class="opacity-0 -translate-y-1"
  >
    <div v-if="visible" class="search-bar" @keydown="handleKeydown">
      <input
        ref="searchInput"
        v-model="query"
        class="search-input"
        placeholder="Find..."
        spellcheck="false"
      />
      <button
        class="search-toggle"
        :class="{ 'search-toggle-active': caseSensitive }"
        @click="toggleCaseSensitive"
        title="Match Case"
      >
        Aa
      </button>
      <button
        class="search-toggle"
        :class="{ 'search-toggle-active': regex }"
        @click="toggleRegex"
        title="Use Regular Expression"
      >
        .*
      </button>
      <span v-if="query && resultCount !== undefined" class="search-count">
        <template v-if="resultCount > 0">
          {{ (resultIndex ?? 0) + 1 }} of {{ resultCount }}
        </template>
        <template v-else>
          No results
        </template>
      </span>
      <div class="search-divider"></div>
      <button class="search-btn" @click="$emit('previous')" title="Previous Match (Shift+Enter)">
        <ChevronUpIcon class="w-3.5 h-3.5" />
      </button>
      <button class="search-btn" @click="$emit('next')" title="Next Match (Enter)">
        <ChevronDownIcon class="w-3.5 h-3.5" />
      </button>
      <button class="search-btn search-btn-close" @click="$emit('close')" title="Close (Escape)">
        <XMarkIcon class="w-3.5 h-3.5" />
      </button>
    </div>
  </Transition>
</template>

<style scoped>
.search-bar {
  display: flex;
  align-items: center;
  gap: 2px;
  padding: 4px 8px;
  background: var(--bg-elevated);
  border-bottom: 1px solid var(--border-default);
}

.search-input {
  flex: 1;
  min-width: 120px;
  max-width: 240px;
  height: 24px;
  padding: 0 8px;
  font-size: 0.75rem;
  font-family: inherit;
  color: var(--text-primary);
  background: var(--bg-secondary);
  border: 1px solid var(--border-default);
  outline: none;
}

.search-input:focus {
  border-color: var(--accent-cyan);
}

.search-input::placeholder {
  color: var(--text-muted);
}

.search-toggle {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  font-size: 0.65rem;
  font-weight: 700;
  font-family: inherit;
  color: var(--text-muted);
  background: transparent;
  border: 1px solid transparent;
  cursor: pointer;
  transition: all 0.1s ease;
}

.search-toggle:hover {
  color: var(--text-secondary);
  border-color: var(--border-default);
}

.search-toggle-active {
  color: var(--accent-cyan);
  border-color: var(--accent-cyan);
  background: rgba(0, 212, 255, 0.1);
}

.search-count {
  font-size: 0.65rem;
  color: var(--text-muted);
  white-space: nowrap;
  padding: 0 4px;
}

.search-divider {
  width: 1px;
  height: 16px;
  background: var(--border-default);
  margin: 0 2px;
}

.search-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  color: var(--text-muted);
  background: transparent;
  border: 1px solid transparent;
  cursor: pointer;
  transition: all 0.1s ease;
}

.search-btn:hover {
  color: var(--text-secondary);
  border-color: var(--border-default);
}

.search-btn-close:hover {
  color: var(--accent-red);
  border-color: var(--accent-red);
}
</style>
