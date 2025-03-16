import { useState } from 'react'
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from '@/components/ui/resizable'
import { Header } from '@/components/header'
import { KeyValueTable } from '@/components/key-value-table'
import { NamespaceSidebar } from '@/components/namespace-sidebar'
import { ValueEditor } from '@/components/value-editor'
import { ValuePreview } from '@/components/value-preview'
import { useToast } from '@/hooks/use-toast'
import { type KVEntry, type KVNamespace, selectFolder, deleteKeys, updateValue, formatExpiration, hasValue } from '@/lib/api'

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
  const { toast } = useToast()

  const handleFolderSelect = async () => {
    try {
      setIsLoading(true)
      const { folderPath, namespaces: loadedNamespaces } = await selectFolder()

      if (folderPath) {
        setSelectedFolder(folderPath)
        setNamespaces(loadedNamespaces)

        if (loadedNamespaces.length > 0) {
          setSelectedNamespace(loadedNamespaces[0].id)
          setKeyValues(loadedNamespaces[0].entries)
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
    if (!selectedNamespace || !selectedFolder || keyIds.length === 0) {
      return
    }

    setIsLoading(true)
    try {
      const keysToDelete = keyIds
        .map(id => keyValues.find(kv => kv.id === id)?.key || '')
        .filter(k => k !== '')

      const updatedNamespaces = await deleteKeys(selectedNamespace, selectedFolder, keysToDelete)

      setNamespaces(updatedNamespaces)

      const updatedNamespace = updatedNamespaces.find(ns => ns.id === selectedNamespace)
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
    if (!editingKey || !selectedNamespace || !selectedFolder) {
      return
    }

    setIsLoading(true)
    try {
      const updatedNamespaces = await updateValue(selectedNamespace, selectedFolder, editingKey, editingValue)

      setNamespaces(updatedNamespaces)

      const updatedNamespace = updatedNamespaces.find(ns => ns.id === selectedNamespace)
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

  return (
    <>
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
    </>
  )
}