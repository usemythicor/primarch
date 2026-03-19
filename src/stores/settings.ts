import { defineStore } from 'pinia';
import { ref, computed, watch } from 'vue';
import type { Theme } from '../types';
import { themes, getThemeById, getDefaultTheme } from '../themes/presets';

const STORAGE_KEY = 'mythicor-terminal-settings';

interface Settings {
  themeId: string;
  fontSize: number;
  fontFamily: string;
  cursorBlink: boolean;
  cursorStyle: 'block' | 'underline' | 'bar';
}

const defaultSettings: Settings = {
  themeId: 'dracula',
  fontSize: 14,
  fontFamily: "'Cascadia Code', 'Fira Code', 'JetBrains Mono', monospace",
  cursorBlink: true,
  cursorStyle: 'block',
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

  // Actions
  function setTheme(id: string) {
    if (getThemeById(id)) {
      themeId.value = id;
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

  function resetToDefaults() {
    themeId.value = defaultSettings.themeId;
    fontSize.value = defaultSettings.fontSize;
    fontFamily.value = defaultSettings.fontFamily;
    cursorBlink.value = defaultSettings.cursorBlink;
    cursorStyle.value = defaultSettings.cursorStyle;
  }

  // Auto-save settings
  watch(
    [themeId, fontSize, fontFamily, cursorBlink, cursorStyle],
    () => {
      saveSettings({
        themeId: themeId.value,
        fontSize: fontSize.value,
        fontFamily: fontFamily.value,
        cursorBlink: cursorBlink.value,
        cursorStyle: cursorStyle.value,
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

    // Computed
    currentTheme,
    availableThemes,
    terminalOptions,

    // Actions
    setTheme,
    setFontSize,
    setFontFamily,
    setCursorBlink,
    setCursorStyle,
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
