import { defineStore } from 'pinia';
import { ref, computed, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { Theme } from '../types';
import { themes, getThemeById, getDefaultTheme } from '../themes/presets';

const STORAGE_KEY = 'primarch-settings';

export type AiProvider = 'none' | 'api' | 'claude' | 'codex';
export type MarkdownRenderingMode = 'auto' | 'always' | 'never';
export type BellStyle = 'none' | 'visual' | 'sound' | 'both';
/** xterm accepts 'normal'/'bold' or a numeric CSS weight. */
export type FontWeight = 'normal' | 'bold' | '100' | '200' | '300' | '400' | '500' | '600' | '700' | '800' | '900';

/** Contrast floor applied to ANSI colors, in WCAG ratio terms. 1 disables it. */
export const contrastRatioOptions = [
  { value: 1, label: 'Off', hint: 'Use theme colors exactly as defined' },
  { value: 3, label: '3:1', hint: 'Readable for large text (WCAG AA large)' },
  { value: 4.5, label: '4.5:1', hint: 'WCAG AA — recommended' },
  { value: 7, label: '7:1', hint: 'WCAG AAA — maximum legibility' },
] as const;

interface Settings {
  themeId: string;
  fontSize: number;
  fontFamily: string;
  lineHeight: number;
  letterSpacing: number;
  fontWeight: FontWeight;
  fontWeightBold: FontWeight;
  minimumContrastRatio: number;
  cursorBlink: boolean;
  cursorStyle: 'block' | 'underline' | 'bar';
  accentColor: string;
  anthropicApiKey: string;
  aiProvider: AiProvider;
  markdownRendering: MarkdownRenderingMode;
  bellStyle: BellStyle;
  timestampPrompt: boolean;
  dimInactivePanes: boolean;
  showPaneHeader: boolean;
  notifyCommandFinish: boolean;
  windowOpacity: number;
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
  fontFamily: "'Cascadia Code', 'Fira Code', 'JetBrains Mono', 'SF Mono', Menlo, Consolas, monospace",
  // 1.2 gives rows a little breathing room without costing a meaningful number
  // of visible lines; xterm's own default is a tighter 1.0.
  lineHeight: 1.2,
  letterSpacing: 0,
  fontWeight: 'normal',
  fontWeightBold: 'bold',
  minimumContrastRatio: 1,
  cursorBlink: true,
  cursorStyle: 'block',
  accentColor: 'cyan',
  anthropicApiKey: '',
  aiProvider: 'none',
  markdownRendering: 'auto',
  bellStyle: 'both',
  timestampPrompt: false,
  dimInactivePanes: true,
  showPaneHeader: false,
  notifyCommandFinish: true,
  windowOpacity: 100,
};

/** The stock font stack, exported so the picker can offer it as "System default". */
export const defaultFontFamily = defaultSettings.fontFamily;

export const useSettingsStore = defineStore('settings', () => {
  // Load settings from localStorage
  const savedSettings = loadSettings();

  // State
  const themeId = ref(savedSettings.themeId);
  const fontSize = ref(savedSettings.fontSize);
  const fontFamily = ref(savedSettings.fontFamily);
  const lineHeight = ref(savedSettings.lineHeight);
  const letterSpacing = ref(savedSettings.letterSpacing);
  const fontWeight = ref<FontWeight>(savedSettings.fontWeight);
  const fontWeightBold = ref<FontWeight>(savedSettings.fontWeightBold);
  const minimumContrastRatio = ref(savedSettings.minimumContrastRatio);
  const cursorBlink = ref(savedSettings.cursorBlink);
  const cursorStyle = ref(savedSettings.cursorStyle);
  const accentColor = ref(savedSettings.accentColor);
  const anthropicApiKey = ref(savedSettings.anthropicApiKey);
  const aiProvider = ref<AiProvider>(savedSettings.aiProvider);
  const markdownRendering = ref<MarkdownRenderingMode>(savedSettings.markdownRendering);
  const bellStyle = ref<BellStyle>(savedSettings.bellStyle);
  const timestampPrompt = ref(savedSettings.timestampPrompt);
  const dimInactivePanes = ref(savedSettings.dimInactivePanes);
  const showPaneHeader = ref(savedSettings.showPaneHeader);
  const notifyCommandFinish = ref(savedSettings.notifyCommandFinish);
  const windowOpacity = ref(savedSettings.windowOpacity);
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
    lineHeight: lineHeight.value,
    letterSpacing: letterSpacing.value,
    fontWeight: fontWeight.value,
    fontWeightBold: fontWeightBold.value,
    minimumContrastRatio: minimumContrastRatio.value,
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

  /** First family in the CSS stack, unquoted — what the font picker displays. */
  const fontFamilyName = computed(() => {
    const first = fontFamily.value.split(',')[0]?.trim() ?? '';
    return first.replace(/^['"]|['"]$/g, '');
  });

  /** True while the untouched stock stack is in use, rather than a chosen family. */
  const usingDefaultFont = computed(() => fontFamily.value === defaultSettings.fontFamily);

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

  /**
   * Select a single installed family by name. Stored as a CSS stack with a
   * generic fallback so a font that fails to load still renders fixed-pitch.
   */
  function setFontFamilyName(family: string) {
    if (!family) {
      fontFamily.value = defaultSettings.fontFamily;
      return;
    }
    // Family names contain spaces, so they must be quoted; pick a quote style
    // the name itself doesn't use.
    const quoted = family.includes("'") ? `"${family}"` : `'${family}'`;
    fontFamily.value = `${quoted}, monospace`;
  }

  function setLineHeight(value: number) {
    // Round to one decimal so the slider can't persist float noise.
    lineHeight.value = Math.max(1, Math.min(2, Math.round(value * 10) / 10));
  }

  function setLetterSpacing(value: number) {
    // xterm spaces characters in whole pixels; fractions are ignored.
    letterSpacing.value = Math.max(0, Math.min(4, Math.round(value)));
  }

  function setFontWeight(weight: FontWeight) {
    fontWeight.value = weight;
  }

  function setFontWeightBold(weight: FontWeight) {
    fontWeightBold.value = weight;
  }

  function setMinimumContrastRatio(ratio: number) {
    minimumContrastRatio.value = Math.max(1, Math.min(21, ratio));
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

  function setMarkdownRendering(mode: MarkdownRenderingMode) {
    markdownRendering.value = mode;
  }

  function setBellStyle(style: BellStyle) {
    bellStyle.value = style;
  }

  function setTimestampPrompt(enabled: boolean) {
    timestampPrompt.value = enabled;
  }

  function setDimInactivePanes(enabled: boolean) {
    dimInactivePanes.value = enabled;
  }

  function setShowPaneHeader(enabled: boolean) {
    showPaneHeader.value = enabled;
  }

  function setNotifyCommandFinish(enabled: boolean) {
    notifyCommandFinish.value = enabled;
  }

  function setWindowOpacity(value: number) {
    windowOpacity.value = Math.max(50, Math.min(100, Math.round(value)));
  }

  async function detectAiClis() {
    try {
      availableAiClis.value = await invoke<string[]>('detect_ai_clis');
      // Auto-select the first detected CLI if no provider has been chosen
      if (aiProvider.value === 'none' && availableAiClis.value.length > 0) {
        aiProvider.value = availableAiClis.value[0] as AiProvider;
      }
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
    lineHeight.value = defaultSettings.lineHeight;
    letterSpacing.value = defaultSettings.letterSpacing;
    fontWeight.value = defaultSettings.fontWeight;
    fontWeightBold.value = defaultSettings.fontWeightBold;
    minimumContrastRatio.value = defaultSettings.minimumContrastRatio;
    cursorBlink.value = defaultSettings.cursorBlink;
    cursorStyle.value = defaultSettings.cursorStyle;
    accentColor.value = defaultSettings.accentColor;
    applyAccentColor(defaultSettings.accentColor);
    applyColorMode(false); // Default theme (Dracula) is dark
    anthropicApiKey.value = defaultSettings.anthropicApiKey;
    aiProvider.value = defaultSettings.aiProvider;
    bellStyle.value = defaultSettings.bellStyle;
  }

  // Auto-save settings
  watch(
    [themeId, fontSize, fontFamily, lineHeight, letterSpacing, fontWeight, fontWeightBold, minimumContrastRatio, cursorBlink, cursorStyle, accentColor, anthropicApiKey, aiProvider, markdownRendering, bellStyle, timestampPrompt, dimInactivePanes, showPaneHeader, notifyCommandFinish, windowOpacity],
    () => {
      saveSettings({
        themeId: themeId.value,
        fontSize: fontSize.value,
        fontFamily: fontFamily.value,
        lineHeight: lineHeight.value,
        letterSpacing: letterSpacing.value,
        fontWeight: fontWeight.value,
        fontWeightBold: fontWeightBold.value,
        minimumContrastRatio: minimumContrastRatio.value,
        cursorBlink: cursorBlink.value,
        cursorStyle: cursorStyle.value,
        accentColor: accentColor.value,
        anthropicApiKey: anthropicApiKey.value,
        aiProvider: aiProvider.value,
        markdownRendering: markdownRendering.value,
        bellStyle: bellStyle.value,
        timestampPrompt: timestampPrompt.value,
        dimInactivePanes: dimInactivePanes.value,
        showPaneHeader: showPaneHeader.value,
        notifyCommandFinish: notifyCommandFinish.value,
        windowOpacity: windowOpacity.value,
      });
    },
    { deep: true }
  );

  return {
    // State
    themeId,
    fontSize,
    fontFamily,
    lineHeight,
    letterSpacing,
    fontWeight,
    fontWeightBold,
    minimumContrastRatio,
    cursorBlink,
    cursorStyle,
    accentColor,
    anthropicApiKey,
    aiProvider,
    markdownRendering,
    bellStyle,
    timestampPrompt,
    dimInactivePanes,
    showPaneHeader,
    notifyCommandFinish,
    windowOpacity,
    availableAiClis,

    // Computed
    currentTheme,
    availableThemes,
    terminalOptions,
    isLightTheme,
    fontFamilyName,
    usingDefaultFont,

    // Actions
    setTheme,
    setFontSize,
    setFontFamily,
    setFontFamilyName,
    setLineHeight,
    setLetterSpacing,
    setFontWeight,
    setFontWeightBold,
    setMinimumContrastRatio,
    setCursorBlink,
    setCursorStyle,
    setAccentColor,
    setAnthropicApiKey,
    setAiProvider,
    setMarkdownRendering,
    setBellStyle,
    setTimestampPrompt,
    setDimInactivePanes,
    setShowPaneHeader,
    setNotifyCommandFinish,
    setWindowOpacity,
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
