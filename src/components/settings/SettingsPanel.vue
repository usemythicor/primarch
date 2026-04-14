<script setup lang="ts">
import { computed, ref } from 'vue';
import {
  XMarkIcon,
  SwatchIcon,
  ArrowPathIcon,
  CursorArrowRaysIcon,
  SparklesIcon,
  PaintBrushIcon,
  ChevronRightIcon,
  BellAlertIcon,
  ComputerDesktopIcon,
} from '@heroicons/vue/24/outline';
import { invoke } from '@tauri-apps/api/core';
import { useSettingsStore, accentPresets } from '../../stores/settings';

const emit = defineEmits<{
  (e: 'close'): void;
}>();

const settingsStore = useSettingsStore();

// Shell integration state
const shellIntegrationInstalled = ref(false);
const shellIntegrationLoading = ref(false);
const isMac = navigator.platform.startsWith('Mac');

async function checkShellIntegration() {
  try {
    shellIntegrationInstalled.value = await invoke<boolean>('is_shell_integration_installed');
  } catch {
    shellIntegrationInstalled.value = false;
  }
}

async function toggleShellIntegration() {
  shellIntegrationLoading.value = true;
  try {
    if (shellIntegrationInstalled.value) {
      await invoke('uninstall_shell_integration');
    } else {
      await invoke('install_shell_integration');
    }
    await checkShellIntegration();
  } catch (e) {
    console.error('Shell integration error:', e);
  } finally {
    shellIntegrationLoading.value = false;
  }
}

// Check on mount
checkShellIntegration();

const themes = computed(() => settingsStore.availableThemes);
const darkThemes = computed(() => themes.value.filter(t => !t.light));
const lightThemes = computed(() => themes.value.filter(t => t.light));
const currentThemeId = computed(() => settingsStore.themeId);
const currentTheme = computed(() => settingsStore.currentTheme);
const fontSize = computed(() => settingsStore.fontSize);
const cursorStyle = computed(() => settingsStore.cursorStyle);
const cursorBlink = computed(() => settingsStore.cursorBlink);
const currentAccent = computed(() => settingsStore.accentColor);

const bellStyle = computed(() => settingsStore.bellStyle);
const hasKey = computed(() => !!settingsStore.anthropicApiKey);
const aiProvider = computed(() => settingsStore.aiProvider);
const availableAiClis = computed(() => settingsStore.availableAiClis);
const showThemePicker = ref(false);

function selectTheme(id: string) {
  settingsStore.setTheme(id);
  showThemePicker.value = false;
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
        <div class="flex items-center gap-2 mb-3">
          <SwatchIcon class="w-4 h-4" style="color: var(--accent-cyan);" />
          <span class="text-header">THEME</span>
        </div>

        <!-- Current theme display with picker button -->
        <button
          @click="showThemePicker = !showThemePicker"
          class="w-full flex items-center justify-between px-3 py-2.5 transition-all duration-150"
          :style="{
            background: 'var(--bg-tertiary)',
            border: showThemePicker ? '1px solid var(--accent-cyan)' : '1px solid var(--border-subtle)',
          }"
        >
          <div class="flex items-center gap-3">
            <div
              class="w-4 h-4 rounded-sm"
              :style="{ backgroundColor: currentTheme.background, border: '1px solid var(--border-strong)' }"
            ></div>
            <span class="text-label" style="color: var(--text-secondary);">
              {{ currentTheme.name }}
            </span>
            <span
              v-if="currentTheme.light"
              class="text-label px-1.5 py-0.5"
              style="background: var(--bg-hover); color: var(--text-muted); font-size: 0.55rem;"
            >
              LIGHT
            </span>
          </div>
          <div class="flex items-center gap-2">
            <PaintBrushIcon class="w-4 h-4" style="color: var(--text-muted);" />
            <ChevronRightIcon
              class="w-3 h-3 transition-transform duration-150"
              :style="{ color: 'var(--text-muted)', transform: showThemePicker ? 'rotate(90deg)' : 'rotate(0)' }"
            />
          </div>
        </button>

        <!-- Theme picker dropdown -->
        <Transition
          enter-active-class="transition-all duration-150 ease-out"
          enter-from-class="opacity-0 max-h-0"
          enter-to-class="opacity-100 max-h-[400px]"
          leave-active-class="transition-all duration-100 ease-in"
          leave-from-class="opacity-100 max-h-[400px]"
          leave-to-class="opacity-0 max-h-0"
        >
          <div
            v-if="showThemePicker"
            class="mt-2 overflow-hidden"
            style="border: 1px solid var(--border-subtle);"
          >
            <div class="max-h-[300px] overflow-y-auto">
              <!-- Dark themes -->
              <div class="px-3 py-2" style="background: var(--bg-primary); border-bottom: 1px solid var(--border-subtle);">
                <span class="text-label" style="color: var(--text-muted);">DARK</span>
              </div>
              <div class="grid grid-cols-2">
                <button
                  v-for="theme in darkThemes"
                  :key="theme.id"
                  @click="selectTheme(theme.id)"
                  class="flex items-center gap-2 px-3 py-2 transition-all duration-150 text-left"
                  :style="{
                    background: currentThemeId === theme.id ? 'rgba(var(--accent-rgb), 0.08)' : 'transparent',
                    borderRight: '1px solid var(--border-subtle)',
                    borderBottom: '1px solid var(--border-subtle)',
                  }"
                >
                  <div
                    class="w-3 h-3 rounded-sm flex-shrink-0"
                    :style="{ backgroundColor: theme.background, border: '1px solid var(--border-strong)' }"
                  ></div>
                  <span
                    class="text-label truncate"
                    :style="{ color: currentThemeId === theme.id ? 'var(--accent-cyan)' : 'var(--text-secondary)' }"
                  >
                    {{ theme.name }}
                  </span>
                </button>
              </div>

              <!-- Light themes -->
              <div class="px-3 py-2" style="background: var(--bg-primary); border-bottom: 1px solid var(--border-subtle);">
                <span class="text-label" style="color: var(--text-muted);">LIGHT</span>
              </div>
              <div class="grid grid-cols-2">
                <button
                  v-for="theme in lightThemes"
                  :key="theme.id"
                  @click="selectTheme(theme.id)"
                  class="flex items-center gap-2 px-3 py-2 transition-all duration-150 text-left"
                  :style="{
                    background: currentThemeId === theme.id ? 'rgba(var(--accent-rgb), 0.08)' : 'transparent',
                    borderRight: '1px solid var(--border-subtle)',
                    borderBottom: '1px solid var(--border-subtle)',
                  }"
                >
                  <div
                    class="w-3 h-3 rounded-sm flex-shrink-0"
                    :style="{ backgroundColor: theme.background, border: '1px solid var(--border-strong)' }"
                  ></div>
                  <span
                    class="text-label truncate"
                    :style="{ color: currentThemeId === theme.id ? 'var(--accent-cyan)' : 'var(--text-secondary)' }"
                  >
                    {{ theme.name }}
                  </span>
                </button>
              </div>
            </div>
          </div>
        </Transition>
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

      <!-- Bell / Alert Style -->
      <div>
        <div class="flex items-center gap-2 mb-4">
          <BellAlertIcon class="w-4 h-4" style="color: var(--accent-cyan);" />
          <span class="text-header">BELL / ALERTS</span>
        </div>
        <div class="flex gap-2">
          <button
            v-for="style in ['none', 'visual', 'sound', 'both'] as const"
            :key="style"
            @click="settingsStore.setBellStyle(style)"
            class="flex-1 px-4 py-2.5 transition-all duration-150 uppercase"
            :style="{
              background: bellStyle === style ? 'rgba(var(--accent-rgb), 0.08)' : 'var(--bg-tertiary)',
              border: bellStyle === style ? '1px solid var(--accent-cyan)' : '1px solid var(--border-subtle)',
              color: bellStyle === style ? 'var(--accent-cyan)' : 'var(--text-muted)',
              fontSize: '0.65rem',
              fontWeight: '600',
              letterSpacing: '0.1em',
            }"
          >
            {{ style }}
          </button>
        </div>
        <div class="mt-2">
          <span style="font-size: 0.6rem; color: var(--text-muted);">
            Notifies when a program sends a bell signal (e.g. input needed, task complete).
          </span>
        </div>
      </div>

      <!-- Shell Integration -->
      <div>
        <div class="flex items-center gap-2 mb-3">
          <ComputerDesktopIcon class="w-4 h-4" style="color: var(--accent-cyan);" />
          <span class="text-header">SHELL INTEGRATION</span>
        </div>

        <div
          class="flex items-center justify-between px-3 py-3"
          style="background: var(--bg-tertiary); border: 1px solid var(--border-subtle);"
        >
          <div class="flex-1 min-w-0 mr-3">
            <span class="text-label block" style="color: var(--text-secondary);">
              "Open in Primarch" context menu
            </span>
            <span style="font-size: 0.6rem; color: var(--text-muted);">
              {{ isMac
                ? 'Adds a Finder Quick Action for folders'
                : 'Adds a right-click option for folders in Explorer'
              }}
            </span>
          </div>
          <button
            @click="toggleShellIntegration"
            :disabled="shellIntegrationLoading"
            class="px-3 py-1.5 transition-all duration-150 flex-shrink-0"
            :style="{
              background: shellIntegrationInstalled
                ? 'transparent'
                : 'rgba(var(--accent-rgb), 0.15)',
              border: shellIntegrationInstalled
                ? '1px solid var(--border-default)'
                : '1px solid var(--accent-cyan)',
              color: shellIntegrationInstalled
                ? 'var(--text-muted)'
                : 'var(--accent-cyan)',
              fontSize: '0.65rem',
              fontWeight: '600',
              letterSpacing: '0.1em',
              opacity: shellIntegrationLoading ? 0.5 : 1,
            }"
          >
            {{ shellIntegrationLoading
              ? 'WORKING...'
              : shellIntegrationInstalled
                ? 'UNINSTALL'
                : 'INSTALL'
            }}
          </button>
        </div>

        <div v-if="shellIntegrationInstalled" class="mt-2">
          <span style="font-size: 0.6rem; color: var(--text-muted);">
            {{ isMac
              ? 'Right-click a folder in Finder → Quick Actions → Open in Primarch.'
              : 'Right-click a folder in Explorer → Open in Primarch. On Windows 11, this appears under "Show more options".'
            }}
          </span>
        </div>
      </div>

      <!-- AI / API Key -->
      <div>
        <div class="flex items-center gap-2 mb-4">
          <SparklesIcon class="w-4 h-4" style="color: var(--accent-cyan);" />
          <span class="text-header">AI COMMIT MESSAGES</span>
        </div>

        <!-- Provider Selection -->
        <div class="space-y-2 mb-4">
          <button
            @click="settingsStore.setAiProvider('none')"
            class="w-full flex items-center gap-3 px-3 py-2.5 transition-all duration-150 text-left"
            :style="{
              background: aiProvider === 'none' ? 'rgba(var(--accent-rgb), 0.08)' : 'var(--bg-tertiary)',
              border: aiProvider === 'none' ? '1px solid var(--accent-cyan)' : '1px solid var(--border-subtle)',
            }"
          >
            <div
              class="w-3 h-3 rounded-full border-2 flex items-center justify-center"
              :style="{
                borderColor: aiProvider === 'none' ? 'var(--accent-cyan)' : 'var(--text-muted)',
              }"
            >
              <div
                v-if="aiProvider === 'none'"
                class="w-1.5 h-1.5 rounded-full"
                style="background: var(--accent-cyan);"
              ></div>
            </div>
            <div class="flex-1">
              <span class="text-label" :style="{ color: aiProvider === 'none' ? 'var(--accent-cyan)' : 'var(--text-secondary)' }">
                Disabled
              </span>
            </div>
          </button>

          <button
            v-if="availableAiClis.includes('claude')"
            @click="settingsStore.setAiProvider('claude')"
            class="w-full flex items-center gap-3 px-3 py-2.5 transition-all duration-150 text-left"
            :style="{
              background: aiProvider === 'claude' ? 'rgba(var(--accent-rgb), 0.08)' : 'var(--bg-tertiary)',
              border: aiProvider === 'claude' ? '1px solid var(--accent-cyan)' : '1px solid var(--border-subtle)',
            }"
          >
            <div
              class="w-3 h-3 rounded-full border-2 flex items-center justify-center"
              :style="{
                borderColor: aiProvider === 'claude' ? 'var(--accent-cyan)' : 'var(--text-muted)',
              }"
            >
              <div
                v-if="aiProvider === 'claude'"
                class="w-1.5 h-1.5 rounded-full"
                style="background: var(--accent-cyan);"
              ></div>
            </div>
            <div class="flex-1">
              <span class="text-label" :style="{ color: aiProvider === 'claude' ? 'var(--accent-cyan)' : 'var(--text-secondary)' }">
                Claude Code CLI
              </span>
              <span class="text-label ml-2" style="color: var(--accent-green); font-size: 0.55rem;">FREE</span>
            </div>
          </button>

          <button
            v-if="availableAiClis.includes('codex')"
            @click="settingsStore.setAiProvider('codex')"
            class="w-full flex items-center gap-3 px-3 py-2.5 transition-all duration-150 text-left"
            :style="{
              background: aiProvider === 'codex' ? 'rgba(var(--accent-rgb), 0.08)' : 'var(--bg-tertiary)',
              border: aiProvider === 'codex' ? '1px solid var(--accent-cyan)' : '1px solid var(--border-subtle)',
            }"
          >
            <div
              class="w-3 h-3 rounded-full border-2 flex items-center justify-center"
              :style="{
                borderColor: aiProvider === 'codex' ? 'var(--accent-cyan)' : 'var(--text-muted)',
              }"
            >
              <div
                v-if="aiProvider === 'codex'"
                class="w-1.5 h-1.5 rounded-full"
                style="background: var(--accent-cyan);"
              ></div>
            </div>
            <div class="flex-1">
              <span class="text-label" :style="{ color: aiProvider === 'codex' ? 'var(--accent-cyan)' : 'var(--text-secondary)' }">
                Codex CLI
              </span>
              <span class="text-label ml-2" style="color: var(--accent-green); font-size: 0.55rem;">FREE</span>
            </div>
          </button>

          <button
            @click="settingsStore.setAiProvider('api')"
            class="w-full flex items-center gap-3 px-3 py-2.5 transition-all duration-150 text-left"
            :style="{
              background: aiProvider === 'api' ? 'rgba(var(--accent-rgb), 0.08)' : 'var(--bg-tertiary)',
              border: aiProvider === 'api' ? '1px solid var(--accent-cyan)' : '1px solid var(--border-subtle)',
            }"
          >
            <div
              class="w-3 h-3 rounded-full border-2 flex items-center justify-center"
              :style="{
                borderColor: aiProvider === 'api' ? 'var(--accent-cyan)' : 'var(--text-muted)',
              }"
            >
              <div
                v-if="aiProvider === 'api'"
                class="w-1.5 h-1.5 rounded-full"
                style="background: var(--accent-cyan);"
              ></div>
            </div>
            <div class="flex-1">
              <span class="text-label" :style="{ color: aiProvider === 'api' ? 'var(--accent-cyan)' : 'var(--text-secondary)' }">
                Anthropic API
              </span>
              <span class="text-label ml-2" style="color: var(--text-muted); font-size: 0.55rem;">REQUIRES KEY</span>
            </div>
          </button>
        </div>

        <!-- API Key (only show when API is selected) -->
        <div v-if="aiProvider === 'api'">
          <div class="flex items-center gap-2">
            <input
              v-model="settingsStore.anthropicApiKey"
              :type="hasKey ? 'password' : 'text'"
              :placeholder="hasKey ? '' : 'sk-ant-...'"
              class="flex-1 px-3 py-2 transition-all duration-150"
              style="
                background: var(--bg-primary);
                border: 1px solid var(--border-default);
                color: var(--text-primary);
                font-family: var(--font-mono);
                font-size: 0.7rem;
              "
            />
            <button
              v-if="hasKey"
              @click="settingsStore.setAnthropicApiKey('')"
              class="btn-icon btn-icon-danger"
              title="Remove key"
            >
              <XMarkIcon class="w-3.5 h-3.5" />
            </button>
          </div>

          <div class="flex items-center justify-between mt-2">
            <span style="font-size: 0.6rem; color: var(--text-muted);">Stored locally.</span>
            <a
              href="https://console.anthropic.com/settings/keys"
              target="_blank"
              class="text-label transition-colors"
              style="color: var(--accent-cyan); cursor: pointer; text-decoration: none;"
            >
              Get key
            </a>
          </div>
        </div>

        <!-- Info text for CLI options -->
        <div v-if="aiProvider === 'claude' || aiProvider === 'codex'" class="mt-2">
          <span style="font-size: 0.6rem; color: var(--text-muted);">
            Uses your existing {{ aiProvider === 'claude' ? 'Claude Max' : 'OpenAI' }} subscription.
          </span>
        </div>

        <!-- No CLIs detected message -->
        <div v-if="availableAiClis.length === 0" class="mt-3 px-3 py-2" style="background: var(--bg-tertiary); border: 1px solid var(--border-subtle);">
          <span style="font-size: 0.6rem; color: var(--text-muted);">
            No AI CLIs detected. Install <a href="https://claude.ai/code" target="_blank" style="color: var(--accent-cyan);">Claude Code</a> for free commit messages with your Max subscription.
          </span>
        </div>
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
        style="border: none;"
      >
        <ArrowPathIcon class="w-3.5 h-3.5" />
        <span class="text-label">Reset</span>
      </button>
      <span class="text-label" style="color: var(--text-muted);">Auto-saved</span>
    </div>
  </div>
</template>
