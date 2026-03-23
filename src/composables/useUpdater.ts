import { ref } from 'vue';
import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { openUrl } from '@tauri-apps/plugin-opener';

export interface UpdateInfo {
  version: string;
  date?: string;
  body?: string;
}

const updateAvailable = ref(false);
const updateInfo = ref<UpdateInfo | null>(null);
const isChecking = ref(false);
const isDownloading = ref(false);
const downloadProgress = ref(0);
const error = ref<string | null>(null);
const requiresManualUpdate = ref(false);

const RELEASES_URL = 'https://github.com/usemythicor/primarch/releases/latest';

let updateInstance: Awaited<ReturnType<typeof check>> | null = null;

export function useUpdater() {
  async function checkForUpdates(silent = false): Promise<boolean> {
    if (isChecking.value) return false;

    isChecking.value = true;
    error.value = null;

    try {
      const update = await check();

      if (update) {
        updateAvailable.value = true;
        updateInfo.value = {
          version: update.version,
          date: update.date,
          body: update.body,
        };
        updateInstance = update;
        return true;
      }

      updateAvailable.value = false;
      updateInfo.value = null;
      return false;
    } catch (e) {
      if (!silent) {
        error.value = e instanceof Error ? e.message : 'Failed to check for updates';
      }
      return false;
    } finally {
      isChecking.value = false;
    }
  }

  async function downloadAndInstall(): Promise<void> {
    if (!updateInstance) {
      error.value = 'No update available';
      return;
    }

    // On macOS, unsigned apps can't auto-update - direct to manual download
    const isMac = navigator.platform.toLowerCase().includes('mac');
    if (isMac) {
      requiresManualUpdate.value = true;
      return;
    }

    isDownloading.value = true;
    downloadProgress.value = 0;
    error.value = null;

    try {
      await updateInstance.downloadAndInstall((event) => {
        if (event.event === 'Started' && event.data.contentLength) {
          downloadProgress.value = 0;
        } else if (event.event === 'Progress') {
          downloadProgress.value = event.data.chunkLength;
        } else if (event.event === 'Finished') {
          downloadProgress.value = 100;
        }
      });

      // Relaunch the app after install
      await relaunch();
    } catch (e) {
      // If auto-update fails, fallback to manual
      requiresManualUpdate.value = true;
      error.value = e instanceof Error ? e.message : 'Failed to install update';
      isDownloading.value = false;
    }
  }

  async function openReleasesPage(): Promise<void> {
    await openUrl(RELEASES_URL);
  }

  function dismissUpdate() {
    updateAvailable.value = false;
    updateInfo.value = null;
    updateInstance = null;
  }

  return {
    // State
    updateAvailable,
    updateInfo,
    isChecking,
    isDownloading,
    downloadProgress,
    error,
    requiresManualUpdate,

    // Actions
    checkForUpdates,
    downloadAndInstall,
    dismissUpdate,
    openReleasesPage,
  };
}
