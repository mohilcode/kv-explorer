import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from '@/components/ui/resizable'
import { open } from '@tauri-apps/api/dialog'
import { invoke } from '@tauri-apps/api/tauri'
import { useEffect, useState } from 'react'
import { GridBackground } from './components/grid-background'
import { Header } from './components/header'
import { KeyValueTable } from './components/key-value-table'
import { NamespaceSidebar } from './components/namespace-sidebar'
import { Toaster } from './components/ui/toaster'
import { ValueEditor } from './components/value-editor'
import { ValuePreview } from './components/value-preview'
import { useToast } from './hooks/use-toast'

interface KVEntry {
  id: string
  key: string
  blob_id: string
  expiration: number | null
  metadata: string | null
  value: unknown
}

interface KVNamespace {
  id: string
  name: string
  count: number
  entries: KVEntry[]
}

export default function App() {
  const [selectedFolder, setSelectedFolder] = useState<string | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [namespaces, setNamespaces] = useState<KVNamespace[]>([])
  const [selectedNamespace, setSelectedNamespace] = useState<string | null>(null)
  const [selectedKeys, setSelectedKeys] = useState<string[]>([])
  const [viewingKeyId, setViewingKeyId] = useState<string | null>(null)
  const [selectedValue, setSelectedValue] = useState<unknown | null>(null)
  const [isEditing, setIsEditing] = useState(false)
  const [editingKey, setEditingKey] = useState<string | null>(null)
  const [editingValue, setEditingValue] = useState<unknown | null>(null)
  const [keyValues, setKeyValues] = useState<KVEntry[]>([])
  const { toast } = useToast()

  useEffect(() => {
    document.documentElement.classList.add('dark')
  }, [])

  const handleFolderSelect = async () => {
    try {
      setIsLoading(true)
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Select Wrangler Project Folder',
      })

      if (selected) {
        setSelectedFolder(selected as string)

        const result = await invoke<KVNamespace[]>('select_folder', { path: selected })

        const transformedNamespaces = result.map(ns => ({
          id: ns.id,
          name: ns.id.toUpperCase(),
          count: ns.entries.length,
          entries: ns.entries.map((entry, index) => ({
            id: index.toString(),
            key: entry.key,
            blob_id: entry.blob_id,
            expiration: entry.expiration,
            metadata: entry.metadata,
            value: entry.value,
          })),
        }))

        setNamespaces(transformedNamespaces)

        if (transformedNamespaces.length > 0) {
          setSelectedNamespace(transformedNamespaces[0].id)
          setKeyValues(transformedNamespaces[0].entries)
        }

        toast({
          title: 'FOLDER SELECTED',
          description: 'Successfully loaded KV namespaces',
        })
      }
    } catch (error) {
      toast({
        title: 'ERROR',
        description: String(error),
        variant: 'destructive',
      })
    } finally {
      setIsLoading(false)
    }
  }

  const handleNamespaceSelect = (namespaceId: string) => {
    setSelectedNamespace(namespaceId)
    setSelectedKeys([])
    setViewingKeyId(null)
    setSelectedValue(null)

    const selected = namespaces.find(ns => ns.id === namespaceId)
    if (selected) {
      setKeyValues(selected.entries)
    }
  }

  const handleKeySelect = (keyId: string) => {
    if (keyId === 'all') {
      setSelectedKeys(keyValues.map(kv => kv.id))
      return
    }

    if (keyId === 'none') {
      setSelectedKeys([])
      return
    }

    if (selectedKeys.includes(keyId)) {
      setSelectedKeys(selectedKeys.filter(k => k !== keyId))
    } else {
      setSelectedKeys([...selectedKeys, keyId])
    }
  }

  const handleKeyView = (keyId: string) => {
    const keyValue = keyValues.find(kv => kv.id === keyId)
    if (keyValue) {
      setViewingKeyId(keyId)
      setSelectedValue(keyValue.value)
    }
  }

  const handleEdit = (keyId: string) => {
    const keyValue = keyValues.find(kv => kv.id === keyId)
    if (keyValue) {
      setEditingKey(keyValue.key)
      setEditingValue(keyValue.value)
      setIsEditing(true)
    }
  }

  const handleDelete = async (keyIds: string[]) => {
    if (!selectedNamespace || keyIds.length === 0) {
      return
    }

    setIsLoading(true)
    try {
      const keysToDelete = keyIds
        .map(id => keyValues.find(kv => kv.id === id)?.key || '')
        .filter(k => k !== '')

      await invoke('delete_kv', {
        namespaceId: selectedNamespace,
        keys: keysToDelete,
      })

      const result = await invoke<KVNamespace[]>('select_folder', { path: selectedFolder })

      const transformedNamespaces = result.map(ns => ({
        id: ns.id,
        name: ns.id.toUpperCase(),
        count: ns.entries.length,
        entries: ns.entries.map((entry, index) => ({
          id: index.toString(),
          key: entry.key,
          blob_id: entry.blob_id,
          expiration: entry.expiration,
          metadata: entry.metadata,
          value: entry.value,
        })),
      }))

      setNamespaces(transformedNamespaces)

      const updatedNamespace = transformedNamespaces.find(ns => ns.id === selectedNamespace)
      if (updatedNamespace) {
        setKeyValues(updatedNamespace.entries)
      }

      setSelectedKeys([])
      setViewingKeyId(null)
      setSelectedValue(null)

      toast({
        title: 'KEYS DELETED',
        description: `Successfully deleted ${keyIds.length} key(s)`,
      })
    } catch (error) {
      toast({
        title: 'ERROR',
        description: String(error),
        variant: 'destructive',
      })
    } finally {
      setIsLoading(false)
    }
  }

  const handleSaveEdit = async () => {
    if (!editingKey || !selectedNamespace) {
      return
    }

    setIsLoading(true)
    try {
      await invoke('update_kv', {
        namespaceId: selectedNamespace,
        key: editingKey,
        valueStr: JSON.stringify(editingValue),
      })

      const result = await invoke<KVNamespace[]>('select_folder', { path: selectedFolder })

      const transformedNamespaces = result.map(ns => ({
        id: ns.id,
        name: ns.id.toUpperCase(),
        count: ns.entries.length,
        entries: ns.entries.map((entry, index) => ({
          id: index.toString(),
          key: entry.key,
          blob_id: entry.blob_id,
          expiration: entry.expiration,
          metadata: entry.metadata,
          value: entry.value,
        })),
      }))

      setNamespaces(transformedNamespaces)

      const updatedNamespace = transformedNamespaces.find(ns => ns.id === selectedNamespace)
      if (updatedNamespace) {
        setKeyValues(updatedNamespace.entries)
      }

      setIsEditing(false)
      setEditingKey(null)
      setEditingValue(null)

      toast({
        title: 'VALUE UPDATED',
        description: 'Successfully saved changes',
      })
    } catch (error) {
      toast({
        title: 'ERROR',
        description: String(error),
        variant: 'destructive',
      })
    } finally {
      setIsLoading(false)
    }
  }

  const handleCancelEdit = () => {
    setIsEditing(false)
    setEditingKey(null)
    setEditingValue(null)
  }

  const formatExpiration = (timestamp: number | null): string => {
    if (!timestamp) {
      return 'No expiration'
    }
    return new Date(timestamp).toLocaleDateString()
  }

  const hasValue = (value: unknown): boolean => {
    return value !== null && value !== undefined
  }

  return (
    <div className="relative flex h-screen flex-col text-white font-mono">
      <GridBackground />
      <Header
        title="WRANGLER KV EXPLORER"
        version="v1.0"
        selectedFolder={selectedFolder}
        onFolderSelect={handleFolderSelect}
        isLoading={isLoading}
      />
      <ResizablePanelGroup direction="horizontal" className="flex-1 overflow-hidden">
        <ResizablePanel defaultSize={20} minSize={15} maxSize={40}>
          <NamespaceSidebar
            namespaces={namespaces}
            selectedNamespace={selectedNamespace}
            onNamespaceSelect={handleNamespaceSelect}
          />
        </ResizablePanel>
        <ResizableHandle
          withHandle
          className="bg-zinc-800 w-1 hover:w-1 hover:bg-zinc-600 transition-colors"
        />
        <ResizablePanel defaultSize={80}>
          <main className="flex flex-1 flex-col overflow-hidden h-full">
            {isEditing && editingKey ? (
              <ValueEditor
                keyName={editingKey}
                value={editingValue}
                onChange={setEditingValue}
                onSave={handleSaveEdit}
                onCancel={handleCancelEdit}
              />
            ) : (
              <ResizablePanelGroup direction="vertical" className="flex-1 overflow-hidden">
                <ResizablePanel defaultSize={60} minSize={30}>
                  <KeyValueTable
                    keyValues={keyValues.map(kv => ({
                      ...kv,
                      expiration: formatExpiration(kv.expiration),
                    }))}
                    selectedKeys={selectedKeys}
                    viewingKeyId={viewingKeyId}
                    onKeySelect={handleKeySelect}
                    onKeyView={handleKeyView}
                    onEdit={handleEdit}
                    onDelete={keyId => handleDelete([keyId])}
                    onDeleteSelected={() => handleDelete(selectedKeys)}
                  />
                </ResizablePanel>
                {hasValue(selectedValue) && (
                  <>
                    <ResizableHandle
                      withHandle
                      className="bg-zinc-800 h-1 hover:h-1 hover:bg-zinc-600 transition-colors"
                    />
                    <ResizablePanel defaultSize={40} minSize={20}>
                      <ValuePreview value={selectedValue} />
                    </ResizablePanel>
                  </>
                )}
              </ResizablePanelGroup>
            )}
          </main>
        </ResizablePanel>
      </ResizablePanelGroup>
      <Toaster />
    </div>
  )
}
