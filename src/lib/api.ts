import { open } from '@tauri-apps/api/dialog'
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
  type: 'local' | 'remote'
  accountId?: string
}

export async function selectFolder() {
  const selected = await open({
    directory: true,
    multiple: false,
    title: 'Select Wrangler Project Folder',
  })

  if (!selected) {
    return { folderPath: null, namespaces: [] }
  }

  const result = await invoke<KVNamespace[]>('select_folder', { path: selected })

  const transformedNamespaces = result.map(ns => ({
    id: ns.id,
    name: ns.id.toUpperCase(),
    count: ns.entries.length,
    type: 'local' as const,
    entries: ns.entries.map((entry, index) => ({
      id: index.toString(),
      key: entry.key,
      blob_id: entry.blob_id,
      expiration: entry.expiration,
      metadata: entry.metadata,
      value: entry.value,
    })),
  }))

  return {
    folderPath: selected as string,
    namespaces: transformedNamespaces,
  }
}

export async function deleteKeys(namespaceId: string, folderPath: string, keysToDelete: string[]) {
  await invoke('delete_kv', {
    namespaceId,
    keys: keysToDelete,
  })

  return await refreshNamespaces(folderPath)
}

export async function updateValue(
  namespaceId: string,
  folderPath: string,
  key: string,
  value: unknown
) {
  await invoke('update_kv', {
    namespaceId,
    key,
    valueStr: JSON.stringify(value),
  })

  return await refreshNamespaces(folderPath)
}

async function refreshNamespaces(folderPath: string) {
  const result = await invoke<KVNamespace[]>('select_folder', { path: folderPath })

  return result.map(ns => ({
    id: ns.id,
    name: ns.id.toUpperCase(),
    count: ns.entries.length,
    type: 'local' as const,
    entries: ns.entries.map((entry, index) => ({
      id: index.toString(),
      key: entry.key,
      blob_id: entry.blob_id,
      expiration: entry.expiration,
      metadata: entry.metadata,
      value: entry.value,
    })),
  }))
}

export async function connectCloudflare(accountId: string, apiToken: string): Promise<void> {
  await invoke('connect_cloudflare', { accountId, apiToken })
}

export async function getRemoteNamespaces(): Promise<KVNamespace[]> {
  const remoteNamespaces = await invoke<KVNamespace[]>('get_remote_namespaces')
  return remoteNamespaces
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
