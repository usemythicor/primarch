import { ref } from 'vue';
import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

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
      error.value = e instanceof Error ? e.message : 'Failed to install update';
      isDownloading.value = false;
    }
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

    // Actions
    checkForUpdates,
    downloadAndInstall,
    dismissUpdate,
  };
}
