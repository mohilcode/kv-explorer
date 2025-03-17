import { open } from '@tauri-apps/api/dialog'
// src/lib/api.ts
import { invoke } from '@tauri-apps/api/tauri'

export interface KVEntry {
  id: string
  key: string
  blob_id: string
  expiration: number | null
  metadata: string | null
  value: unknown
}

export interface KVNamespace {
  id: string
  name: string
  count?: number
  entries: KVEntry[]
  type: string
  accountId?: string
  folderId?: number
}

export interface LocalFolder {
  id: number
  path: string
  name: string
}

export async function getFolders(): Promise<LocalFolder[]> {
  return invoke<LocalFolder[]>('get_folders')
}

export async function addFolder(): Promise<{ folderId: number; namespaces: KVNamespace[] }> {
  const selected = await open({
    directory: true,
    multiple: false,
    title: 'Select Wrangler Project Folder',
  })

  if (!selected) {
    return { folderId: 0, namespaces: [] }
  }

  try {
    const result = await invoke<KVNamespace[]>('add_folder', { path: selected })

    const folderId = result.length > 0 ? result[0].folderId || 0 : 0

    return {
      folderId,
      namespaces: result,
    }
  } catch (error) {
    console.error('Error in add_folder:', error)
    throw error
  }
}

export async function removeFolder(folderId: number): Promise<void> {
  await invoke('remove_folder', { folderId })
}

export async function loadFolder(folderId: number): Promise<KVNamespace[]> {
  return invoke<KVNamespace[]>('load_folder', { folderId })
}

export async function deleteKeys(
  folderId: number,
  namespaceId: string,
  keysToDelete: string[]
): Promise<void> {
  await invoke('delete_kv', {
    folderId,
    namespaceId,
    keys: keysToDelete,
  })
}

export async function updateValue(
  folderId: number,
  namespaceId: string,
  key: string,
  value: unknown
): Promise<void> {
  await invoke('update_kv', {
    folderId,
    namespaceId,
    key,
    valueStr: JSON.stringify(value),
  })
}

export async function connectCloudflare(accountId: string, apiToken: string): Promise<void> {
  await invoke('connect_cloudflare', { accountId, apiToken })
}

export async function getRemoteNamespaces(): Promise<KVNamespace[]> {
  return invoke<KVNamespace[]>('get_remote_namespaces')
}

export async function getRemoteKeys(accountId: string, namespaceId: string): Promise<KVEntry[]> {
  const entries = await invoke<KVEntry[]>('get_remote_keys', {
    accountId,
    namespaceId,
  })

  return entries.map((entry, index) => ({
    ...entry,
    id: index.toString(),
  }))
}

export async function getRemoteValue(
  accountId: string,
  namespaceId: string,
  keyName: string
): Promise<unknown> {
  return invoke('get_remote_value', {
    accountId,
    namespaceId,
    keyName,
  })
}

export async function updateRemoteValue(
  accountId: string,
  namespaceId: string,
  keyName: string,
  value: unknown
): Promise<void> {
  const valueStr = JSON.stringify(value)

  await invoke('update_remote_kv', {
    accountId,
    namespaceId,
    keyName,
    value: valueStr,
  })
}

export async function deleteRemoteKeys(
  accountId: string,
  namespaceId: string,
  keys: string[]
): Promise<void> {
  await invoke('delete_remote_kv', {
    accountId,
    namespaceId,
    keys,
  })
}

export function formatExpiration(timestamp: number | null): string {
  if (!timestamp) {
    return 'No expiration'
  }
  return new Date(timestamp).toLocaleDateString()
}

export function hasValue(value: unknown): boolean {
  return value !== null && value !== undefined
}
