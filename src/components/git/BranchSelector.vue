<script setup lang="ts">
import { ref, computed } from 'vue';
import {
  XMarkIcon,
  PlusIcon,
  TrashIcon,
  CheckIcon,
  ArrowPathIcon,
} from '@heroicons/vue/24/outline';
import { useGitStore } from '../../stores/git';

const emit = defineEmits<{
  (e: 'close'): void;
}>();

const gitStore = useGitStore();

const searchQuery = ref('');
const newBranchName = ref('');
const showCreateForm = ref(false);

const branches = computed(() => gitStore.branches);
const branchesLoading = computed(() => gitStore.branchesLoading);
const isCheckingOut = computed(() => gitStore.isCheckingOut);
const isCreatingBranch = computed(() => gitStore.isCreatingBranch);
const currentBranch = computed(() => gitStore.branchName);

const filteredBranches = computed(() => {
  if (!searchQuery.value) return branches.value;
  const query = searchQuery.value.toLowerCase();
  return branches.value.filter(b => b.name.toLowerCase().includes(query));
});

async function handleCheckout(branchName: string) {
  if (branchName === currentBranch.value) return;
  await gitStore.checkoutBranch(branchName);
}

async function handleCreate() {
  const name = newBranchName.value.trim();
  if (!name) return;

  await gitStore.createBranch(name, true);
  newBranchName.value = '';
  showCreateForm.value = false;
}

async function handleDelete(branchName: string) {
  if (confirm(`Delete branch "${branchName}"?`)) {
    await gitStore.deleteBranch(branchName);
  }
}

function handleRefresh() {
  gitStore.loadBranches();
}

function toggleCreateForm() {
  showCreateForm.value = !showCreateForm.value;
  if (showCreateForm.value) {
    newBranchName.value = '';
  }
}
</script>

<template>
  <div
    class="branch-selector fixed inset-0 flex items-center justify-center z-50"
    @click.self="emit('close')"
  >
    <div
      class="selector-panel w-80 max-h-96 flex flex-col rounded-lg shadow-xl"
      style="background: var(--bg-secondary); border: 1px solid var(--border-subtle);"
    >
      <!-- Header -->
      <div
        class="flex items-center justify-between px-3 py-2"
        style="border-bottom: 1px solid var(--border-subtle);"
      >
        <span class="text-header">SWITCH BRANCH</span>
        <div class="flex items-center gap-1">
          <button
            @click="handleRefresh"
            class="btn-icon"
            :class="branchesLoading ? 'animate-spin' : ''"
            title="Refresh"
          >
            <ArrowPathIcon class="w-4 h-4" />
          </button>
          <button
            @click="toggleCreateForm"
            class="btn-icon"
            title="Create Branch"
          >
            <PlusIcon class="w-4 h-4" />
          </button>
          <button
            @click="emit('close')"
            class="btn-icon btn-icon-danger"
            title="Close"
          >
            <XMarkIcon class="w-4 h-4" />
          </button>
        </div>
      </div>

      <!-- Create Branch Form -->
      <div
        v-if="showCreateForm"
        class="px-3 py-2"
        style="border-bottom: 1px solid var(--border-subtle);"
      >
        <div class="flex items-center gap-2">
          <input
            v-model="newBranchName"
            type="text"
            placeholder="New branch name..."
            class="flex-1 px-2 py-1 rounded text-sm"
            style="background: var(--bg-primary); color: var(--text-primary); border: 1px solid var(--border-subtle);"
            @keyup.enter="handleCreate"
            @keyup.escape="showCreateForm = false"
          />
          <button
            @click="handleCreate"
            :disabled="!newBranchName.trim() || isCreatingBranch"
            class="p-1 rounded transition-colors"
            :style="{
              color: newBranchName.trim() ? 'var(--accent-green)' : 'var(--text-muted)',
              cursor: newBranchName.trim() ? 'pointer' : 'not-allowed',
            }"
            title="Create"
          >
            <CheckIcon class="w-4 h-4" />
          </button>
        </div>
      </div>

      <!-- Search -->
      <div class="px-3 py-2">
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Search branches..."
          class="w-full px-2 py-1 rounded text-sm"
          style="background: var(--bg-primary); color: var(--text-primary); border: 1px solid var(--border-subtle);"
        />
      </div>

      <!-- Branch List -->
      <div class="flex-1 overflow-y-auto">
        <div
          v-if="branchesLoading"
          class="flex items-center justify-center py-8"
        >
          <ArrowPathIcon class="w-5 h-5 animate-spin" style="color: var(--text-muted);" />
        </div>

        <div v-else-if="filteredBranches.length === 0" class="px-3 py-4 text-center">
          <span class="text-label" style="color: var(--text-muted);">
            {{ searchQuery ? 'No branches found' : 'No branches' }}
          </span>
        </div>

        <div v-else>
          <div
            v-for="branch in filteredBranches"
            :key="branch.name"
            class="branch-item flex items-center gap-2 px-3 py-1.5 cursor-pointer transition-colors group"
            :class="{ 'is-current': branch.isHead }"
            @click="handleCheckout(branch.name)"
          >
            <!-- Current indicator -->
            <CheckIcon
              v-if="branch.isHead"
              class="w-3.5 h-3.5 flex-shrink-0"
              style="color: var(--accent-green);"
            />
            <div v-else class="w-3.5" />

            <!-- Branch name -->
            <span
              class="flex-1 text-sm truncate"
              :style="{ color: branch.isHead ? 'var(--accent-green)' : 'var(--text-primary)' }"
            >
              {{ branch.name }}
            </span>

            <!-- Ahead/Behind badges -->
            <span
              v-if="branch.ahead > 0"
              class="text-xs px-1 rounded"
              style="background: rgba(80, 250, 123, 0.2); color: var(--accent-green);"
            >
              {{ branch.ahead }}
            </span>
            <span
              v-if="branch.behind > 0"
              class="text-xs px-1 rounded"
              style="background: rgba(255, 121, 198, 0.2); color: var(--accent-pink);"
            >
              {{ branch.behind }}
            </span>

            <!-- Delete button -->
            <button
              v-if="!branch.isHead"
              @click.stop="handleDelete(branch.name)"
              class="delete-btn p-0.5 transition-opacity opacity-0 group-hover:opacity-100"
              style="color: var(--accent-red);"
              title="Delete Branch"
            >
              <TrashIcon class="w-3.5 h-3.5" />
            </button>
          </div>
        </div>
      </div>

      <!-- Loading overlay -->
      <div
        v-if="isCheckingOut"
        class="absolute inset-0 flex items-center justify-center rounded-lg"
        style="background: rgba(0, 0, 0, 0.5);"
      >
        <div class="flex items-center gap-2">
          <ArrowPathIcon class="w-5 h-5 animate-spin" style="color: var(--accent-cyan);" />
          <span class="text-sm" style="color: var(--text-primary);">Switching branch...</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.branch-selector {
  background: rgba(0, 0, 0, 0.5);
}

.selector-panel {
  position: relative;
}

.branch-item:hover {
  background: var(--bg-hover);
}

.branch-item.is-current {
  background: rgba(80, 250, 123, 0.05);
}

.delete-btn:hover {
  background: var(--bg-elevated);
  border-radius: 2px;
}
</style>
