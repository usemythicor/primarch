<script setup lang="ts">
import { computed } from 'vue';
import {
  XMarkIcon,
  SwatchIcon,
  ArrowPathIcon,
  CursorArrowRaysIcon,
} from '@heroicons/vue/24/outline';
import { useSettingsStore, accentPresets } from '../../stores/settings';

const emit = defineEmits<{
  (e: 'close'): void;
}>();

const settingsStore = useSettingsStore();

const themes = computed(() => settingsStore.availableThemes);
const currentThemeId = computed(() => settingsStore.themeId);
const fontSize = computed(() => settingsStore.fontSize);
const cursorStyle = computed(() => settingsStore.cursorStyle);
const cursorBlink = computed(() => settingsStore.cursorBlink);
const currentAccent = computed(() => settingsStore.accentColor);

function selectTheme(id: string) {
  settingsStore.setTheme(id);
}

function changeFontSize(delta: number) {
  settingsStore.setFontSize(fontSize.value + delta);
}

function toggleCursorBlink() {
  settingsStore.setCursorBlink(!cursorBlink.value);
}

function setCursorStyle(style: 'block' | 'underline' | 'bar') {
  settingsStore.setCursorStyle(style);
}

function resetSettings() {
  if (confirm('Reset all settings to defaults?')) {
    settingsStore.resetToDefaults();
  }
}
</script>

<template>
  <div
    class="w-[500px] max-h-[80vh] flex flex-col"
    style="background: var(--bg-secondary); border: 1px solid var(--border-default);"
  >
    <!-- Header -->
    <div
      class="flex items-center justify-between px-5 h-12"
      style="border-bottom: 1px solid var(--border-subtle);"
    >
      <div class="flex items-center gap-3">
        <div class="w-1 h-5 rounded-sm" style="background: var(--accent-cyan);"></div>
        <span class="text-header">SETTINGS</span>
      </div>
      <button
        @click="emit('close')"
        class="btn-icon btn-icon-danger"
      >
        <XMarkIcon class="w-4 h-4" />
      </button>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto p-5 space-y-6">
      <!-- Theme Selection -->
      <div>
        <div class="flex items-center gap-2 mb-4">
          <SwatchIcon class="w-4 h-4" style="color: var(--accent-cyan);" />
          <span class="text-header">THEME</span>
        </div>
        <div class="grid grid-cols-2 gap-2">
          <button
            v-for="theme in themes"
            :key="theme.id"
            @click="selectTheme(theme.id)"
            class="flex items-center gap-3 px-3 py-2.5 transition-all duration-150 text-left"
            :style="{
              background: currentThemeId === theme.id ? 'rgba(var(--accent-rgb), 0.08)' : 'var(--bg-tertiary)',
              border: currentThemeId === theme.id ? '1px solid var(--accent-cyan)' : '1px solid var(--border-subtle)',
              boxShadow: currentThemeId === theme.id ? '0 0 20px rgba(var(--accent-rgb), 0.1)' : 'none',
            }"
          >
            <div
              class="w-3 h-3 rounded-sm"
              :style="{ backgroundColor: theme.background, border: '1px solid var(--border-strong)' }"
            ></div>
            <span
              class="text-label"
              :style="{ color: currentThemeId === theme.id ? 'var(--accent-cyan)' : 'var(--text-secondary)' }"
            >
              {{ theme.name }}
            </span>
          </button>
        </div>
      </div>

      <!-- Accent Color -->
      <div>
        <div class="flex items-center gap-2 mb-4">
          <div class="w-4 h-4 rounded-full" style="background: var(--accent-cyan);"></div>
          <span class="text-header">ACCENT COLOR</span>
        </div>
        <div class="flex gap-2 flex-wrap">
          <button
            v-for="preset in accentPresets"
            :key="preset.id"
            @click="settingsStore.setAccentColor(preset.id)"
            class="w-8 h-8 transition-all duration-150 flex items-center justify-center"
            :style="{
              background: preset.color,
              boxShadow: currentAccent === preset.id ? `0 0 0 2px var(--bg-secondary), 0 0 0 3px ${preset.color}` : 'none',
              opacity: currentAccent === preset.id ? 1 : 0.6,
            }"
            :title="preset.name"
          >
            <svg v-if="currentAccent === preset.id" class="w-3.5 h-3.5" viewBox="0 0 16 16" fill="none" stroke="var(--bg-primary)" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="3,8 7,12 13,4" />
            </svg>
          </button>
        </div>
      </div>

      <!-- Font Size -->
      <div>
        <span class="text-header block mb-4">FONT SIZE</span>
        <div class="flex items-center gap-4">
          <button
            @click="changeFontSize(-1)"
            class="btn-toolbar w-10 h-10 flex items-center justify-center"
            style="background: var(--bg-tertiary);"
          >
            <span class="text-lg font-light">-</span>
          </button>
          <div
            class="w-20 h-10 flex items-center justify-center"
            style="background: var(--bg-elevated); border: 1px solid var(--border-subtle);"
          >
            <span style="color: var(--text-primary); font-size: 0.8rem; font-weight: 600;">{{ fontSize }}px</span>
          </div>
          <button
            @click="changeFontSize(1)"
            class="btn-toolbar w-10 h-10 flex items-center justify-center"
            style="background: var(--bg-tertiary);"
          >
            <span class="text-lg font-light">+</span>
          </button>
        </div>
      </div>

      <!-- Cursor Style -->
      <div>
        <div class="flex items-center gap-2 mb-4">
          <CursorArrowRaysIcon class="w-4 h-4" style="color: var(--accent-cyan);" />
          <span class="text-header">CURSOR</span>
        </div>
        <div class="flex gap-2">
          <button
            v-for="style in ['block', 'underline', 'bar'] as const"
            :key="style"
            @click="setCursorStyle(style)"
            class="flex-1 px-4 py-2.5 transition-all duration-150 uppercase"
            :style="{
              background: cursorStyle === style ? 'rgba(var(--accent-rgb), 0.08)' : 'var(--bg-tertiary)',
              border: cursorStyle === style ? '1px solid var(--accent-cyan)' : '1px solid var(--border-subtle)',
              color: cursorStyle === style ? 'var(--accent-cyan)' : 'var(--text-muted)',
              fontSize: '0.65rem',
              fontWeight: '600',
              letterSpacing: '0.1em',
            }"
          >
            {{ style }}
          </button>
        </div>
      </div>

      <!-- Cursor Blink -->
      <div class="flex items-center justify-between">
        <span class="text-header">CURSOR BLINK</span>
        <button
          @click="toggleCursorBlink"
          class="relative w-12 h-6 transition-all duration-200"
          :style="{
            background: cursorBlink ? 'rgba(var(--accent-rgb), 0.2)' : 'var(--bg-tertiary)',
            border: cursorBlink ? '1px solid var(--accent-cyan)' : '1px solid var(--border-default)',
          }"
        >
          <span
            class="absolute top-1 w-4 h-4 transition-all duration-200"
            :style="{
              left: cursorBlink ? '26px' : '2px',
              background: cursorBlink ? 'var(--accent-cyan)' : 'var(--text-muted)',
            }"
          ></span>
        </button>
      </div>
    </div>

    <!-- Footer -->
    <div
      class="flex items-center justify-between px-5 h-12"
      style="border-top: 1px solid var(--border-subtle);"
    >
      <button
        @click="resetSettings"
        class="btn-ghost flex items-center gap-2 px-3 py-1.5"
      >
        <ArrowPathIcon class="w-3.5 h-3.5" />
        <span class="text-label">Reset</span>
      </button>
      <span class="text-label" style="color: var(--text-muted);">Auto-saved</span>
    </div>
  </div>
</template>
