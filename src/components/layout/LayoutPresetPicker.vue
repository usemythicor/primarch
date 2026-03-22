<script setup lang="ts">
import { ref } from 'vue';
import { ChevronDownIcon } from '@heroicons/vue/24/outline';
import { useLayoutStore } from '../../stores/layout';
import { layoutPresets, type LayoutPreset } from './presets';

const layoutStore = useLayoutStore();
const isOpen = ref(false);

function selectPreset(preset: LayoutPreset) {
  layoutStore.applyPreset(preset);
  isOpen.value = false;
}

function toggleDropdown() {
  isOpen.value = !isOpen.value;
}

function closeDropdown() {
  isOpen.value = false;
}
</script>

<template>
  <div class="relative">
    <button
      @click="toggleDropdown"
      class="btn-toolbar px-3 py-1.5"
      :class="{ 'btn-toolbar-active': isOpen }"
    >
      <svg class="w-3.5 h-3.5" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
        <rect x="1.5" y="1.5" width="5.5" height="5.5" rx="0.5" />
        <rect x="9" y="1.5" width="5.5" height="5.5" rx="0.5" />
        <rect x="1.5" y="9" width="5.5" height="5.5" rx="0.5" />
        <rect x="9" y="9" width="5.5" height="5.5" rx="0.5" />
      </svg>
      <span class="text-label">Layouts</span>
      <ChevronDownIcon
        class="w-3 h-3 transition-transform duration-150"
        :style="{ transform: isOpen ? 'rotate(180deg)' : 'rotate(0deg)' }"
      />
    </button>

    <!-- Dropdown menu -->
    <Transition
      enter-active-class="transition duration-100 ease-out"
      enter-from-class="opacity-0 -translate-y-1"
      enter-to-class="opacity-100 translate-y-0"
      leave-active-class="transition duration-75 ease-in"
      leave-from-class="opacity-100 translate-y-0"
      leave-to-class="opacity-0 -translate-y-1"
    >
      <div
        v-if="isOpen"
        class="absolute top-full right-0 mt-1 w-56 z-50 overflow-hidden"
        style="background: var(--bg-secondary); border: 1px solid var(--border-default);"
      >
        <!-- Header -->
        <div
          class="flex items-center gap-2 px-4 py-2.5"
          style="border-bottom: 1px solid var(--border-subtle);"
        >
          <div class="w-0.5 h-3 rounded-sm" style="background: var(--accent-cyan);"></div>
          <span class="text-label" style="color: var(--text-muted);">Layout Presets</span>
        </div>

        <!-- Preset list -->
        <div class="py-1">
          <button
            v-for="preset in layoutPresets"
            :key="preset.id"
            @click="selectPreset(preset)"
            class="w-full flex items-center gap-3 px-4 py-2.5 transition-all duration-150 text-left group"
            style="background: transparent;"
            @mouseenter="($event.currentTarget as HTMLElement).style.background = 'var(--bg-elevated)'"
            @mouseleave="($event.currentTarget as HTMLElement).style.background = 'transparent'"
          >
            <!-- Thumbnail -->
            <div
              class="w-8 h-6 flex items-center justify-center flex-shrink-0"
              style="border: 1px solid var(--border-subtle); background: var(--bg-tertiary);"
            >
              <!-- 2 Columns -->
              <svg v-if="preset.id === '2-col'" viewBox="0 0 28 20" class="w-full h-full p-0.5">
                <rect x="1" y="1" width="12" height="18" rx="0.5" fill="none" stroke="var(--accent-cyan)" stroke-width="1" opacity="0.6" />
                <rect x="15" y="1" width="12" height="18" rx="0.5" fill="none" stroke="var(--accent-cyan)" stroke-width="1" opacity="0.6" />
              </svg>
              <!-- 3 Columns -->
              <svg v-else-if="preset.id === '3-col'" viewBox="0 0 28 20" class="w-full h-full p-0.5">
                <rect x="1" y="1" width="7.5" height="18" rx="0.5" fill="none" stroke="var(--accent-cyan)" stroke-width="1" opacity="0.6" />
                <rect x="10.25" y="1" width="7.5" height="18" rx="0.5" fill="none" stroke="var(--accent-cyan)" stroke-width="1" opacity="0.6" />
                <rect x="19.5" y="1" width="7.5" height="18" rx="0.5" fill="none" stroke="var(--accent-cyan)" stroke-width="1" opacity="0.6" />
              </svg>
              <!-- 2x2 Grid -->
              <svg v-else-if="preset.id === '2x2'" viewBox="0 0 28 20" class="w-full h-full p-0.5">
                <rect x="1" y="1" width="12" height="8" rx="0.5" fill="none" stroke="var(--accent-cyan)" stroke-width="1" opacity="0.6" />
                <rect x="15" y="1" width="12" height="8" rx="0.5" fill="none" stroke="var(--accent-cyan)" stroke-width="1" opacity="0.6" />
                <rect x="1" y="11" width="12" height="8" rx="0.5" fill="none" stroke="var(--accent-cyan)" stroke-width="1" opacity="0.6" />
                <rect x="15" y="11" width="12" height="8" rx="0.5" fill="none" stroke="var(--accent-cyan)" stroke-width="1" opacity="0.6" />
              </svg>
              <!-- 2x2 + Bottom -->
              <svg v-else-if="preset.id === '2x2-bottom'" viewBox="0 0 28 20" class="w-full h-full p-0.5">
                <rect x="1" y="1" width="12" height="6" rx="0.5" fill="none" stroke="var(--accent-cyan)" stroke-width="1" opacity="0.6" />
                <rect x="15" y="1" width="12" height="6" rx="0.5" fill="none" stroke="var(--accent-cyan)" stroke-width="1" opacity="0.6" />
                <rect x="1" y="9" width="12" height="6" rx="0.5" fill="none" stroke="var(--accent-cyan)" stroke-width="1" opacity="0.6" />
                <rect x="15" y="9" width="12" height="6" rx="0.5" fill="none" stroke="var(--accent-cyan)" stroke-width="1" opacity="0.6" />
                <rect x="1" y="17" width="26" height="2" rx="0.5" fill="none" stroke="var(--accent-cyan)" stroke-width="1" opacity="0.6" />
              </svg>
              <!-- 3x2 Grid -->
              <svg v-else-if="preset.id === '3x2'" viewBox="0 0 28 20" class="w-full h-full p-0.5">
                <rect x="1" y="1" width="7.5" height="8" rx="0.5" fill="none" stroke="var(--accent-cyan)" stroke-width="1" opacity="0.6" />
                <rect x="10.25" y="1" width="7.5" height="8" rx="0.5" fill="none" stroke="var(--accent-cyan)" stroke-width="1" opacity="0.6" />
                <rect x="19.5" y="1" width="7.5" height="8" rx="0.5" fill="none" stroke="var(--accent-cyan)" stroke-width="1" opacity="0.6" />
                <rect x="1" y="11" width="7.5" height="8" rx="0.5" fill="none" stroke="var(--accent-cyan)" stroke-width="1" opacity="0.6" />
                <rect x="10.25" y="11" width="7.5" height="8" rx="0.5" fill="none" stroke="var(--accent-cyan)" stroke-width="1" opacity="0.6" />
                <rect x="19.5" y="11" width="7.5" height="8" rx="0.5" fill="none" stroke="var(--accent-cyan)" stroke-width="1" opacity="0.6" />
              </svg>
            </div>

            <div class="flex-1 min-w-0">
              <div class="text-label" style="color: var(--text-primary);">
                {{ preset.name }}
              </div>
              <div style="font-size: 0.6rem; color: var(--text-muted);">
                {{ preset.terminalCount }} terminals
              </div>
            </div>

            <ChevronDownIcon
              class="w-3 h-3 -rotate-90 opacity-0 group-hover:opacity-100 transition-opacity duration-150"
              style="color: var(--accent-cyan);"
            />
          </button>
        </div>
      </div>
    </Transition>

    <!-- Backdrop to close dropdown -->
    <div
      v-if="isOpen"
      class="fixed inset-0 z-40"
      @click="closeDropdown"
    ></div>
  </div>
</template>
