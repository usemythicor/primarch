import { defineStore } from 'pinia';
import { ref, computed, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { Theme } from '../types';
import { themes, getThemeById, getDefaultTheme } from '../themes/presets';

const STORAGE_KEY = 'primarch-settings';

export type AiProvider = 'none' | 'api' | 'claude' | 'codex';

interface Settings {
  themeId: string;
  fontSize: number;
  fontFamily: string;
  cursorBlink: boolean;
  cursorStyle: 'block' | 'underline' | 'bar';
  accentColor: string;
  anthropicApiKey: string;
  aiProvider: AiProvider;
}

export interface AccentPreset {
  id: string;
  name: string;
  color: string;
  dim: string;
}

export const accentPresets: AccentPreset[] = [
  { id: 'cyan', name: 'Cyan', color: '#00d4ff', dim: '#00a8cc' },
  { id: 'blue', name: 'Blue', color: '#3b82f6', dim: '#2e6ac4' },
  { id: 'purple', name: 'Purple', color: '#a855f7', dim: '#8b3fd4' },
  { id: 'pink', name: 'Pink', color: '#ec4899', dim: '#c9367f' },
  { id: 'red', name: 'Red', color: '#ef4444', dim: '#c93636' },
  { id: 'orange', name: 'Orange', color: '#f97316', dim: '#d4610f' },
  { id: 'yellow', name: 'Gold', color: '#eab308', dim: '#c49607' },
  { id: 'green', name: 'Green', color: '#22c55e', dim: '#1a9e4b' },
];

const defaultSettings: Settings = {
  themeId: 'dracula',
  fontSize: 14,
  fontFamily: "'Cascadia Code', 'Fira Code', 'JetBrains Mono', monospace",
  cursorBlink: true,
  cursorStyle: 'block',
  accentColor: 'cyan',
  anthropicApiKey: '',
  aiProvider: 'none',
};

export const useSettingsStore = defineStore('settings', () => {
  // Load settings from localStorage
  const savedSettings = loadSettings();

  // State
  const themeId = ref(savedSettings.themeId);
  const fontSize = ref(savedSettings.fontSize);
  const fontFamily = ref(savedSettings.fontFamily);
  const cursorBlink = ref(savedSettings.cursorBlink);
  const cursorStyle = ref(savedSettings.cursorStyle);
  const accentColor = ref(savedSettings.accentColor);
  const anthropicApiKey = ref(savedSettings.anthropicApiKey);
  const aiProvider = ref<AiProvider>(savedSettings.aiProvider);
  const availableAiClis = ref<string[]>([]);

  // Apply accent color to CSS variables
  function applyAccentColor(id: string) {
    const preset = accentPresets.find((p) => p.id === id) || accentPresets[0];
    const root = document.documentElement;
    root.style.setProperty('--accent-cyan', preset.color);
    root.style.setProperty('--accent-cyan-dim', preset.dim);
    root.style.setProperty('--text-accent', preset.color);
    // Convert hex to rgb for rgba() usages
    const r = parseInt(preset.color.slice(1, 3), 16);
    const g = parseInt(preset.color.slice(3, 5), 16);
    const b = parseInt(preset.color.slice(5, 7), 16);
    root.style.setProperty('--accent-rgb', `${r}, ${g}, ${b}`);
  }

  // Apply light/dark mode CSS variables
  function applyColorMode(isLight: boolean) {
    const root = document.documentElement;
    if (isLight) {
      // Light mode
      root.style.setProperty('--bg-primary', '#ffffff');
      root.style.setProperty('--bg-secondary', '#f8f9fa');
      root.style.setProperty('--bg-tertiary', '#f0f1f3');
      root.style.setProperty('--bg-elevated', '#ffffff');
      root.style.setProperty('--bg-hover', '#e9ecef');
      root.style.setProperty('--border-subtle', '#e0e3e7');
      root.style.setProperty('--border-default', '#ced4da');
      root.style.setProperty('--border-strong', '#adb5bd');
      root.style.setProperty('--text-primary', '#1a1d21');
      root.style.setProperty('--text-secondary', '#495057');
      root.style.setProperty('--text-muted', '#868e96');
    } else {
      // Dark mode (defaults)
      root.style.setProperty('--bg-primary', '#06080c');
      root.style.setProperty('--bg-secondary', '#0a0d12');
      root.style.setProperty('--bg-tertiary', '#0f1318');
      root.style.setProperty('--bg-elevated', '#141820');
      root.style.setProperty('--bg-hover', '#1a1f28');
      root.style.setProperty('--border-subtle', '#1a1f28');
      root.style.setProperty('--border-default', '#252a35');
      root.style.setProperty('--border-strong', '#3a4150');
      root.style.setProperty('--text-primary', '#e8eaed');
      root.style.setProperty('--text-secondary', '#8b9099');
      root.style.setProperty('--text-muted', '#5a6270');
    }
  }

  // Apply on init
  applyAccentColor(accentColor.value);
  const initialTheme = getThemeById(themeId.value);
  applyColorMode(initialTheme?.light ?? false);

  // Computed
  const currentTheme = computed((): Theme => {
    return getThemeById(themeId.value) || getDefaultTheme();
  });

  const availableThemes = computed(() => themes);

  const terminalOptions = computed(() => ({
    fontSize: fontSize.value,
    fontFamily: fontFamily.value,
    cursorBlink: cursorBlink.value,
    cursorStyle: cursorStyle.value,
    theme: {
      background: currentTheme.value.background,
      foreground: currentTheme.value.foreground,
      cursor: currentTheme.value.cursor,
      selectionBackground: currentTheme.value.selection,
      black: currentTheme.value.black,
      red: currentTheme.value.red,
      green: currentTheme.value.green,
      yellow: currentTheme.value.yellow,
      blue: currentTheme.value.blue,
      magenta: currentTheme.value.magenta,
      cyan: currentTheme.value.cyan,
      white: currentTheme.value.white,
      brightBlack: currentTheme.value.brightBlack,
      brightRed: currentTheme.value.brightRed,
      brightGreen: currentTheme.value.brightGreen,
      brightYellow: currentTheme.value.brightYellow,
      brightBlue: currentTheme.value.brightBlue,
      brightMagenta: currentTheme.value.brightMagenta,
      brightCyan: currentTheme.value.brightCyan,
      brightWhite: currentTheme.value.brightWhite,
    },
  }));

  // Computed
  const isLightTheme = computed(() => currentTheme.value.light ?? false);

  // Actions
  function setTheme(id: string) {
    const theme = getThemeById(id);
    if (theme) {
      themeId.value = id;
      applyColorMode(theme.light ?? false);
    }
  }

  function setFontSize(size: number) {
    fontSize.value = Math.max(8, Math.min(32, size));
  }

  function setFontFamily(family: string) {
    fontFamily.value = family;
  }

  function setCursorBlink(blink: boolean) {
    cursorBlink.value = blink;
  }

  function setCursorStyle(style: 'block' | 'underline' | 'bar') {
    cursorStyle.value = style;
  }

  function setAnthropicApiKey(key: string) {
    anthropicApiKey.value = key;
  }

  function setAiProvider(provider: AiProvider) {
    aiProvider.value = provider;
  }

  async function detectAiClis() {
    try {
      availableAiClis.value = await invoke<string[]>('detect_ai_clis');
    } catch {
      availableAiClis.value = [];
    }
  }

  // Detect available CLIs on init
  detectAiClis();

  function setAccentColor(id: string) {
    if (accentPresets.find((p) => p.id === id)) {
      accentColor.value = id;
      applyAccentColor(id);
    }
  }

  function resetToDefaults() {
    themeId.value = defaultSettings.themeId;
    fontSize.value = defaultSettings.fontSize;
    fontFamily.value = defaultSettings.fontFamily;
    cursorBlink.value = defaultSettings.cursorBlink;
    cursorStyle.value = defaultSettings.cursorStyle;
    accentColor.value = defaultSettings.accentColor;
    applyAccentColor(defaultSettings.accentColor);
    applyColorMode(false); // Default theme (Dracula) is dark
    anthropicApiKey.value = defaultSettings.anthropicApiKey;
    aiProvider.value = defaultSettings.aiProvider;
  }

  // Auto-save settings
  watch(
    [themeId, fontSize, fontFamily, cursorBlink, cursorStyle, accentColor, anthropicApiKey, aiProvider],
    () => {
      saveSettings({
        themeId: themeId.value,
        fontSize: fontSize.value,
        fontFamily: fontFamily.value,
        cursorBlink: cursorBlink.value,
        cursorStyle: cursorStyle.value,
        accentColor: accentColor.value,
        anthropicApiKey: anthropicApiKey.value,
        aiProvider: aiProvider.value,
      });
    },
    { deep: true }
  );

  return {
    // State
    themeId,
    fontSize,
    fontFamily,
    cursorBlink,
    cursorStyle,
    accentColor,
    anthropicApiKey,
    aiProvider,
    availableAiClis,

    // Computed
    currentTheme,
    availableThemes,
    terminalOptions,
    isLightTheme,

    // Actions
    setTheme,
    setFontSize,
    setFontFamily,
    setCursorBlink,
    setCursorStyle,
    setAccentColor,
    setAnthropicApiKey,
    setAiProvider,
    detectAiClis,
    resetToDefaults,
  };
});

function loadSettings(): Settings {
  try {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved) {
      return { ...defaultSettings, ...JSON.parse(saved) };
    }
  } catch {
    // Return defaults if settings are corrupted
  }
  return defaultSettings;
}

function saveSettings(settings: Settings) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
  } catch {
    // localStorage may be full or unavailable
  }
}
