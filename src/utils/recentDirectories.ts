const STORAGE_KEY = 'primarch-recent-dirs';
const MAX_RECENT = 20;

export function getRecentDirectories(): string[] {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    return stored ? JSON.parse(stored) : [];
  } catch {
    return [];
  }
}

export function addRecentDirectory(path: string): void {
  const dirs = getRecentDirectories().filter((d) => d !== path);
  dirs.unshift(path);
  if (dirs.length > MAX_RECENT) dirs.length = MAX_RECENT;
  localStorage.setItem(STORAGE_KEY, JSON.stringify(dirs));
}
