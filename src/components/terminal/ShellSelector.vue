<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import {
  CommandLineIcon,
  ChevronDownIcon,
  ComputerDesktopIcon,
  PlusIcon,
} from '@heroicons/vue/24/outline';

interface ShellInfo {
  id: string;
  name: string;
  command: string;
  args: string[];
  shell_type: 'powershell' | 'cmd' | 'wsl' | 'git' | 'other';
}

const emit = defineEmits<{
  (e: 'select', shell: ShellInfo): void;
}>();

const shells = ref<ShellInfo[]>([]);
const isOpen = ref(false);
const isLoading = ref(true);

onMounted(async () => {
  try {
    shells.value = await invoke<ShellInfo[]>('get_available_shells');
  } catch {
    // Fallback to basic shells if detection fails
    shells.value = [
      {
        id: 'powershell',
        name: 'Windows PowerShell',
        command: 'powershell.exe',
        args: [],
        shell_type: 'powershell',
      },
      {
        id: 'cmd',
        name: 'Command Prompt',
        command: 'cmd.exe',
        args: [],
        shell_type: 'cmd',
      },
    ];
  } finally {
    isLoading.value = false;
  }
});

function selectShell(shell: ShellInfo) {
  emit('select', shell);
  isOpen.value = false;
}

function getShellIcon(shellType: string) {
  switch (shellType) {
    case 'wsl':
      return ComputerDesktopIcon;
    default:
      return CommandLineIcon;
  }
}

function toggleDropdown() {
  isOpen.value = !isOpen.value;
}

function closeDropdown() {
  isOpen.value = false;
}
</script>

<template>
  <div class="relative" @blur="closeDropdown">
    <button
      @click="toggleDropdown"
      class="btn-toolbar px-3 py-1.5"
      :class="{ 'btn-toolbar-active': isOpen }"
      :disabled="isLoading"
    >
      <PlusIcon class="w-3.5 h-3.5" />
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
        class="absolute top-full left-0 mt-1 w-72 z-50 overflow-hidden"
        style="background: var(--bg-secondary); border: 1px solid var(--border-default);"
      >
        <!-- Header -->
        <div
          class="flex items-center gap-2 px-4 py-2.5"
          style="border-bottom: 1px solid var(--border-subtle);"
        >
          <div class="w-0.5 h-3 rounded-sm" style="background: var(--accent-cyan);"></div>
          <span class="text-label" style="color: var(--text-muted);">Available Shells</span>
        </div>

        <!-- Shell list -->
        <div class="py-1">
          <button
            v-for="shell in shells"
            :key="shell.id"
            @click="selectShell(shell)"
            class="w-full flex items-center gap-3 px-4 py-2.5 transition-all duration-150 text-left group"
            style="background: transparent;"
            @mouseenter="($event.currentTarget as HTMLElement).style.background = 'var(--bg-elevated)'"
            @mouseleave="($event.currentTarget as HTMLElement).style.background = 'transparent'"
          >
            <div
              class="w-7 h-7 flex items-center justify-center transition-all duration-150"
              :style="{
                background: 'var(--bg-tertiary)',
                border: '1px solid var(--border-subtle)',
              }"
            >
              <component
                :is="getShellIcon(shell.shell_type)"
                class="w-3.5 h-3.5"
                style="color: var(--text-muted);"
              />
            </div>
            <div class="flex-1 min-w-0">
              <div class="text-label truncate" style="color: var(--text-primary);">
                {{ shell.name }}
              </div>
              <div
                class="truncate"
                style="font-size: 0.6rem; color: var(--text-muted);"
              >
                {{ shell.command }}
              </div>
            </div>
            <ChevronDownIcon
              class="w-3 h-3 -rotate-90 opacity-0 group-hover:opacity-100 transition-opacity duration-150"
              style="color: var(--accent-cyan);"
            />
          </button>

          <div
            v-if="shells.length === 0 && !isLoading"
            class="px-4 py-6 text-center"
          >
            <span class="text-label" style="color: var(--text-muted);">No shells detected</span>
          </div>

          <div
            v-if="isLoading"
            class="px-4 py-6 text-center"
          >
            <span class="text-label" style="color: var(--text-muted);">Detecting...</span>
          </div>
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
