<script setup lang="ts">
import { ref, onMounted } from 'vue';
import {
  FolderIcon,
  PlusIcon,
  TrashIcon,
  XMarkIcon,
  ClockIcon,
} from '@heroicons/vue/24/outline';
import { useWorkspaceStore } from '../../stores/workspace';

const emit = defineEmits<{
  (e: 'close'): void;
}>();

const workspaceStore = useWorkspaceStore();
const newWorkspaceName = ref('');
const isSaving = ref(false);
const showSaveForm = ref(false);

onMounted(() => {
  workspaceStore.loadWorkspaces();
});

async function saveWorkspace() {
  if (!newWorkspaceName.value.trim()) return;

  isSaving.value = true;
  try {
    await workspaceStore.saveCurrentLayout(newWorkspaceName.value.trim());
    newWorkspaceName.value = '';
    showSaveForm.value = false;
  } catch {
    // Error is displayed via workspaceStore.error
  } finally {
    isSaving.value = false;
  }
}

async function loadWorkspace(id: string) {
  try {
    await workspaceStore.loadWorkspace(id);
    emit('close');
  } catch {
    // Error is displayed via workspaceStore.error
  }
}

async function deleteWorkspace(id: string, event: Event) {
  event.stopPropagation();
  try {
    await workspaceStore.deleteWorkspace(id);
  } catch {
    // Error is displayed via workspaceStore.error
  }
}

function formatDate(dateStr: string) {
  try {
    let date: Date;
    // Handle Rust's unix-seconds format (e.g. "1710812345Z")
    const unixMatch = dateStr.match(/^(\d+)Z$/);
    if (unixMatch) {
      date = new Date(parseInt(unixMatch[1]) * 1000);
    } else {
      date = new Date(dateStr);
    }

    if (isNaN(date.getTime())) return dateStr;

    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const days = Math.floor(diff / (1000 * 60 * 60 * 24));

    if (days === 0) return 'Today';
    if (days === 1) return 'Yesterday';
    if (days < 7) return `${days} days ago`;
    return date.toLocaleDateString();
  } catch {
    return dateStr;
  }
}
</script>

<template>
  <div
    class="w-[420px] max-h-[80vh] flex flex-col"
    style="background: var(--bg-secondary); border: 1px solid var(--border-default);"
  >
    <!-- Header -->
    <div
      class="flex items-center justify-between px-5 h-12"
      style="border-bottom: 1px solid var(--border-subtle);"
    >
      <div class="flex items-center gap-3">
        <div class="w-1 h-5 rounded-sm" style="background: var(--accent-cyan);"></div>
        <span class="text-header">WORKSPACES</span>
        <span
          v-if="workspaceStore.workspaces.length > 0"
          class="text-label px-2 py-0.5"
          style="background: var(--bg-tertiary); color: var(--text-muted);"
        >
          {{ workspaceStore.workspaces.length }}
        </span>
      </div>
      <button
        @click="emit('close')"
        class="btn-icon btn-icon-danger"
      >
        <XMarkIcon class="w-4 h-4" />
      </button>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto p-4">
      <!-- Save current layout -->
      <div class="mb-4">
        <button
          v-if="!showSaveForm"
          @click="showSaveForm = true"
          class="w-full flex items-center justify-center gap-2 py-3 transition-all duration-150"
          style="background: rgba(var(--accent-rgb), 0.08); border: 1px dashed var(--accent-cyan); color: var(--accent-cyan);"
          @mouseenter="($event.target as HTMLElement).style.background = 'rgba(var(--accent-rgb), 0.15)'"
          @mouseleave="($event.target as HTMLElement).style.background = 'rgba(var(--accent-rgb), 0.08)'"
        >
          <PlusIcon class="w-4 h-4" />
          <span class="text-label">Save Current Layout</span>
        </button>

        <div v-else class="space-y-3">
          <input
            v-model="newWorkspaceName"
            type="text"
            placeholder="WORKSPACE NAME..."
            class="w-full px-4 py-2.5 transition-all duration-150 placeholder:uppercase"
            style="
              background: var(--bg-tertiary);
              border: 1px solid var(--border-default);
              color: var(--text-primary);
              font-size: 0.75rem;
              letter-spacing: 0.05em;
            "
            @keyup.enter="saveWorkspace"
            @focus="($event.target as HTMLElement).style.borderColor = 'var(--accent-cyan)'"
            @blur="($event.target as HTMLElement).style.borderColor = 'var(--border-default)'"
            autofocus
          />
          <div class="flex gap-2">
            <button
              @click="saveWorkspace"
              :disabled="!newWorkspaceName.trim() || isSaving"
              class="flex-1 py-2 transition-all duration-150"
              :style="{
                background: !newWorkspaceName.trim() || isSaving ? 'var(--bg-tertiary)' : 'rgba(var(--accent-rgb), 0.1)',
                border: !newWorkspaceName.trim() || isSaving ? '1px solid var(--border-subtle)' : '1px solid var(--accent-cyan)',
                color: !newWorkspaceName.trim() || isSaving ? 'var(--text-muted)' : 'var(--accent-cyan)',
                cursor: !newWorkspaceName.trim() || isSaving ? 'not-allowed' : 'pointer',
              }"
            >
              <span class="text-label">{{ isSaving ? 'Saving...' : 'Save' }}</span>
            </button>
            <button
              @click="showSaveForm = false; newWorkspaceName = ''"
              class="btn-ghost px-4 py-2"
            >
              <span class="text-label">Cancel</span>
            </button>
          </div>
        </div>
      </div>

      <!-- Workspace list -->
      <div class="space-y-1">
        <div
          v-if="workspaceStore.isLoading"
          class="text-center py-12"
          style="color: var(--text-muted);"
        >
          <span class="text-label">Loading...</span>
        </div>

        <div
          v-else-if="workspaceStore.workspaces.length === 0"
          class="text-center py-12"
          style="color: var(--text-muted);"
        >
          <FolderIcon class="w-10 h-10 mx-auto mb-3 opacity-30" />
          <p class="text-label mb-1">No Saved Workspaces</p>
          <p style="font-size: 0.65rem; color: var(--text-muted);">Save your current layout to get started</p>
        </div>

        <div
          v-else
          v-for="workspace in workspaceStore.workspaces"
          :key="workspace.id"
          @click="loadWorkspace(workspace.id)"
          class="card-interactive group flex items-center gap-4 px-4 py-3"
        >
          <div
            class="w-8 h-8 flex items-center justify-center"
            style="background: rgba(var(--accent-rgb), 0.1); border: 1px solid var(--accent-cyan-dim);"
          >
            <FolderIcon class="w-4 h-4" style="color: var(--accent-cyan);" />
          </div>
          <div class="flex-1 min-w-0">
            <div class="text-label truncate" style="color: var(--text-primary);">
              {{ workspace.name }}
            </div>
            <div class="flex items-center gap-1 mt-0.5">
              <ClockIcon class="w-3 h-3" style="color: var(--text-muted);" />
              <span style="font-size: 0.6rem; color: var(--text-muted);">
                {{ formatDate(workspace.createdAt) }}
              </span>
            </div>
          </div>
          <button
            @click="deleteWorkspace(workspace.id, $event)"
            class="btn-icon btn-icon-danger opacity-0 group-hover:opacity-100"
            title="Delete workspace"
          >
            <TrashIcon class="w-4 h-4" />
          </button>
        </div>
      </div>
    </div>

    <!-- Footer -->
    <div
      class="flex items-center justify-center px-5 h-10"
      style="border-top: 1px solid var(--border-subtle);"
    >
      <span class="text-label" style="color: var(--text-muted);">Click to load workspace</span>
    </div>
  </div>
</template>
