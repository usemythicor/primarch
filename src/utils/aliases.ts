const STORAGE_KEY = 'primarch-aliases';

export interface CommandAlias {
  id: string;
  name: string;
  command: string;
}

export function getAliases(): CommandAlias[] {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    return stored ? JSON.parse(stored) : [];
  } catch {
    return [];
  }
}

export function saveAlias(alias: CommandAlias): void {
  const aliases = getAliases().filter((a) => a.id !== alias.id);
  aliases.push(alias);
  localStorage.setItem(STORAGE_KEY, JSON.stringify(aliases));
}

export function deleteAlias(id: string): void {
  const aliases = getAliases().filter((a) => a.id !== id);
  localStorage.setItem(STORAGE_KEY, JSON.stringify(aliases));
}
