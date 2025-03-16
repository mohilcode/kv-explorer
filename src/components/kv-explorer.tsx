import { Header } from '@/components/header'
import { KeyValueTable } from '@/components/key-value-table'
import { NamespaceSidebar } from '@/components/namespace-sidebar'
import { RemoteConnectionModal } from '@/components/remote-connection-modal'
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from '@/components/ui/resizable'
import { ValueEditor } from '@/components/value-editor'
import { ValuePreview } from '@/components/value-preview'
import { useToast } from '@/hooks/use-toast'
import {
  type KVEntry,
  type KVNamespace,
  connectCloudflare,
  deleteKeys,
  deleteRemoteKeys,
  formatExpiration,
  getRemoteKeys,
  getRemoteNamespaces,
  getRemoteValue,
  selectFolder,
  updateRemoteValue,
  updateValue,
} from '@/lib/api'
import { invoke } from '@tauri-apps/api/tauri'
import { useEffect, useState } from 'react'

export function KVExplorer() {
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
  const [isRemoteModalOpen, setIsRemoteModalOpen] = useState(false)
  const [remoteConnections, setRemoteConnections] = useState<{ accountId: string }[]>([])
  const { toast } = useToast()

  useEffect(() => {
    if (remoteConnections.length > 0) {
      loadRemoteNamespaces()
    }
  }, [remoteConnections])

  const handleFolderSelect = async () => {
    try {
      setIsLoading(true)
      const { folderPath, namespaces: localNamespaces } = await selectFolder()

      if (folderPath) {
        setSelectedFolder(folderPath)
        setNamespaces(prev => {
          // Keep remote namespaces, replace local ones
          const remote = prev.filter(ns => ns.type === 'remote')
          return [...remote, ...localNamespaces]
        })

        if (localNamespaces.length > 0) {
          setSelectedNamespace(localNamespaces[0].id)
          setKeyValues(localNamespaces[0].entries)
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

  const handleRemoteConnect = () => {
    setIsRemoteModalOpen(true)
  }

  const handleRemoteConnectionSave = async (accountId: string, apiToken: string) => {
    try {
      setIsLoading(true)
      await connectCloudflare(accountId, apiToken)

      setRemoteConnections(prev => [...prev, { accountId }])

      toast({
        title: 'REMOTE CONNECTION ESTABLISHED',
        description: `Successfully connected to account ${accountId}`,
      })

      setIsRemoteModalOpen(false)
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

  const loadRemoteNamespaces = async () => {
    try {
      setIsLoading(true)
      const remoteNamespaces = await getRemoteNamespaces()

      setNamespaces(prev => {
        const local = prev.filter(ns => ns.type === 'local')
        const typed = remoteNamespaces.map(ns => ({
          ...ns,
          type: ns.type || 'remote',
        }))
        return [...local, ...typed]
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

  const handleNamespaceSelect = async (namespaceId: string) => {
    setSelectedNamespace(namespaceId)
    setSelectedKeys([])
    setViewingKeyId(null)
    setSelectedValue(null)

    const selected = namespaces.find(ns => ns.id === namespaceId)
    if (!selected) return

    setIsLoading(true)
    try {
      if (selected.type === 'local') {
        setKeyValues(selected.entries)
      } else {
        const accountId = selected.accountId || remoteConnections[0]?.accountId
        if (!accountId) throw new Error('Account ID not found')

        const entries = await getRemoteKeys(accountId, namespaceId)
        setKeyValues(entries)
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

  const handleKeyView = async (keyId: string) => {
    const keyValue = keyValues.find(kv => kv.id === keyId)
    if (!keyValue) return

    const selected = namespaces.find(ns => ns.id === selectedNamespace)
    if (!selected) return

    setViewingKeyId(keyId)
    setIsLoading(true)

    try {
      if (selected.type === 'local') {
        setSelectedValue(keyValue.value)
      } else {
        const accountId = selected.accountId || remoteConnections[0]?.accountId
        if (!accountId) throw new Error('Account ID not found')

        const value = await getRemoteValue(accountId, selected.id, keyValue.key)
        setSelectedValue(value)
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

  const handleEdit = (keyId: string) => {
    const keyValue = keyValues.find(kv => kv.id === keyId)
    if (!keyValue) return

    const selected = namespaces.find(ns => ns.id === selectedNamespace)
    if (!selected) return

    setIsLoading(true)

    const loadValue = async () => {
      try {
        let value: unknown

        if (selected.type === 'local') {
          value = keyValue.value
        } else {
          const accountId = selected.accountId || remoteConnections[0]?.accountId
          if (!accountId) throw new Error('Account ID not found')

          value = await getRemoteValue(accountId, selected.id, keyValue.key)
        }

        setEditingKey(keyValue.key)
        setEditingValue(value)
        setIsEditing(true)
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

    loadValue()
  }

  const handleDelete = async (keyIds: string[]) => {
    if (!selectedNamespace) return

    const selected = namespaces.find(ns => ns.id === selectedNamespace)
    if (!selected) return

    const keysToDelete = keyIds
      .map(id => keyValues.find(kv => kv.id === id)?.key)
      .filter((k): k is string => !!k)

    if (keysToDelete.length === 0) return

    setIsLoading(true)
    try {
      if (selected.type === 'local') {
        if (!selectedFolder) throw new Error('No folder selected')

        const updatedNamespaces = await deleteKeys(selectedNamespace, selectedFolder, keysToDelete)

        setNamespaces(prev => {
          const remote = prev.filter(ns => ns.type === 'remote')
          return [...remote, ...updatedNamespaces]
        })

        const updatedNamespace = updatedNamespaces.find(ns => ns.id === selectedNamespace)
        if (updatedNamespace) {
          setKeyValues(updatedNamespace.entries)
        }
      } else {
        const accountId = selected.accountId || remoteConnections[0]?.accountId
        if (!accountId) throw new Error('Account ID not found')

        await deleteRemoteKeys(accountId, selectedNamespace, keysToDelete)

        const entries = await getRemoteKeys(accountId, selectedNamespace)
        setKeyValues(entries)
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
    if (!editingKey || !selectedNamespace) return

    const selected = namespaces.find(ns => ns.id === selectedNamespace)
    if (!selected) return

    setIsLoading(true)
    try {
      if (selected.type === 'local') {
        if (!selectedFolder) throw new Error('No folder selected')

        const updatedNamespaces = await updateValue(
          selectedNamespace,
          selectedFolder,
          editingKey,
          editingValue
        )

        setNamespaces(prev => {
          const remote = prev.filter(ns => ns.type === 'remote')
          return [...remote, ...updatedNamespaces]
        })

        const updatedNamespace = updatedNamespaces.find(ns => ns.id === selectedNamespace)
        if (updatedNamespace) {
          setKeyValues(updatedNamespace.entries)
        }
      } else {
        const accountId = selected.accountId || remoteConnections[0]?.accountId
        if (!accountId) throw new Error('Account ID not found')

        await updateRemoteValue(accountId, selectedNamespace, editingKey, editingValue)

        if (viewingKeyId) {
          setSelectedValue(editingValue)
        }
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

  const handleRemoteDisconnect = async () => {
    try {
      setIsLoading(true)
      await invoke('disconnect_cloudflare')

      setRemoteConnections([])

      setNamespaces(prev => prev.filter(ns => ns.type === 'local'))

      if (
        selectedNamespace &&
        namespaces.find(ns => ns.id === selectedNamespace)?.type === 'remote'
      ) {
        setSelectedNamespace(null)
        setKeyValues([])
        setSelectedKeys([])
        setViewingKeyId(null)
        setSelectedValue(null)
      }

      toast({
        title: 'REMOTE DISCONNECTED',
        description: 'Successfully disconnected from remote KV',
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

  return (
    <>
      <Header
        title="WRANGLER KV EXPLORER"
        version="v1.0"
        selectedFolder={selectedFolder}
        onFolderSelect={handleFolderSelect}
        onRemoteConnect={handleRemoteConnect}
        onRemoteDisconnect={handleRemoteDisconnect}
        remoteConnections={remoteConnections.length}
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
                {selectedValue !== null && (
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

      <RemoteConnectionModal
        isOpen={isRemoteModalOpen}
        onClose={() => setIsRemoteModalOpen(false)}
        onSave={handleRemoteConnectionSave}
      />
    </>
  )
}
