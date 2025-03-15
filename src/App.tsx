import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { open } from "@tauri-apps/api/dialog";
import { Header } from "./components/header";
import { NamespaceSidebar } from "./components/namespace-sidebar";
import { KeyValueTable } from "./components/key-value-table";
import { ValuePreview } from "./components/value-preview";
import { ValueEditor } from "./components/value-editor";
import { Toaster } from "./components/ui/toaster";
import { useToast } from "./hooks/use-toast";
import { GridBackground } from "./components/grid-background";
import { ResizablePanelGroup, ResizablePanel, ResizableHandle } from "@/components/ui/resizable";

// Define interfaces for our data
interface KVEntry {
  id: string;
  key: string;
  blob_id: string;
  expiration: number | null;
  metadata: string | null;
  value: any;
}

interface KVNamespace {
  id: string;
  name: string;
  count: number;
  entries: KVEntry[];
}

export default function App() {
  const [selectedFolder, setSelectedFolder] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [namespaces, setNamespaces] = useState<KVNamespace[]>([]);
  const [selectedNamespace, setSelectedNamespace] = useState<string | null>(null);
  const [selectedKeys, setSelectedKeys] = useState<string[]>([]);
  const [selectedValue, setSelectedValue] = useState<any>(null);
  const [isEditing, setIsEditing] = useState(false);
  const [editingKey, setEditingKey] = useState<string | null>(null);
  const [editingValue, setEditingValue] = useState<any>(null);
  const [keyValues, setKeyValues] = useState<KVEntry[]>([]);
  const { toast } = useToast();

  // Force dark mode
  useEffect(() => {
    document.documentElement.classList.add("dark");
  }, []);

  const handleFolderSelect = async () => {
    try {
      setIsLoading(true);
      // Open folder dialog and get path
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select Wrangler Project Folder",
      });

      if (selected) {
        setSelectedFolder(selected as string);

        // Call Rust backend to load namespaces
        const result = await invoke<KVNamespace[]>("select_folder", { path: selected });

        // Transform the data to match our UI components
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
            value: entry.value
          }))
        }));

        setNamespaces(transformedNamespaces);

        if (transformedNamespaces.length > 0) {
          setSelectedNamespace(transformedNamespaces[0].id);
          setKeyValues(transformedNamespaces[0].entries);
        }

        toast({
          title: "FOLDER SELECTED",
          description: "Successfully loaded KV namespaces",
        });
      }
    } catch (error) {
      toast({
        title: "ERROR",
        description: String(error),
        variant: "destructive",
      });
    } finally {
      setIsLoading(false);
    }
  };

  const handleNamespaceSelect = (namespaceId: string) => {
    setSelectedNamespace(namespaceId);
    setSelectedKeys([]);
    setSelectedValue(null);

    // Find the selected namespace and set its key-values
    const selected = namespaces.find(ns => ns.id === namespaceId);
    if (selected) {
      setKeyValues(selected.entries);
    }
  };

  const handleKeySelect = (keyId: string) => {
    // Handle special cases for select all/none
    if (keyId === "all") {
      setSelectedKeys(keyValues.map(kv => kv.id));
      return;
    }

    if (keyId === "none") {
      setSelectedKeys([]);
      return;
    }

    // Normal key selection
    const keyValue = keyValues.find((kv) => kv.id === keyId);
    if (keyValue) {
      setSelectedValue(keyValue.value);
    }

    // Toggle selection
    if (selectedKeys.includes(keyId)) {
      setSelectedKeys(selectedKeys.filter((k) => k !== keyId));
    } else {
      setSelectedKeys([...selectedKeys, keyId]);
    }
  };

  const handleEdit = (keyId: string) => {
    const keyValue = keyValues.find((kv) => kv.id === keyId);
    if (keyValue) {
      setEditingKey(keyValue.key);
      setEditingValue(keyValue.value);
      setIsEditing(true);
    }
  };

  const handleDelete = async (keyIds: string[]) => {
    if (!selectedNamespace || keyIds.length === 0) return;

    setIsLoading(true);
    try {
      // Get the keys to delete
      const keysToDelete = keyIds.map(id =>
        keyValues.find(kv => kv.id === id)?.key || ""
      ).filter(k => k !== "");

      // Call Rust backend to delete keys
      await invoke("delete_kv", {
        namespaceId: selectedNamespace,
        keys: keysToDelete
      });

      // Refresh data
      const result = await invoke<KVNamespace[]>("select_folder", { path: selectedFolder });

      // Transform and update UI
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
          value: entry.value
        }))
      }));

      setNamespaces(transformedNamespaces);

      // Update selected namespace data
      const updatedNamespace = transformedNamespaces.find(ns => ns.id === selectedNamespace);
      if (updatedNamespace) {
        setKeyValues(updatedNamespace.entries);
      }

      setSelectedKeys([]);
      setSelectedValue(null);

      toast({
        title: "KEYS DELETED",
        description: `Successfully deleted ${keyIds.length} key(s)`,
      });
    } catch (error) {
      toast({
        title: "ERROR",
        description: String(error),
        variant: "destructive",
      });
    } finally {
      setIsLoading(false);
    }
  };

  const handleSaveEdit = async () => {
    if (!editingKey || !selectedNamespace) return;

    setIsLoading(true);
    try {
      // Call Rust backend to update key value
      await invoke("update_kv", {
        namespaceId: selectedNamespace,
        key: editingKey,
        valueStr: JSON.stringify(editingValue)
      });

      // Refresh data
      const result = await invoke<KVNamespace[]>("select_folder", { path: selectedFolder });

      // Transform and update UI
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
          value: entry.value
        }))
      }));

      setNamespaces(transformedNamespaces);

      // Update selected namespace data
      const updatedNamespace = transformedNamespaces.find(ns => ns.id === selectedNamespace);
      if (updatedNamespace) {
        setKeyValues(updatedNamespace.entries);
      }

      setIsEditing(false);
      setEditingKey(null);
      setEditingValue(null);

      toast({
        title: "VALUE UPDATED",
        description: "Successfully saved changes",
      });
    } catch (error) {
      toast({
        title: "ERROR",
        description: String(error),
        variant: "destructive",
      });
    } finally {
      setIsLoading(false);
    }
  };

  const handleCancelEdit = () => {
    setIsEditing(false);
    setEditingKey(null);
    setEditingValue(null);
  };

  // Helper function to format expiration times
  const formatExpiration = (timestamp: number | null): string => {
    if (!timestamp) return "No expiration";
    return new Date(timestamp).toLocaleDateString();
  };

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
        <ResizableHandle withHandle className="bg-zinc-800 w-1 hover:w-1 hover:bg-zinc-600 transition-colors" />
        <ResizablePanel defaultSize={80}>
          <main className="flex flex-1 flex-col overflow-hidden h-full">
            {isEditing ? (
              <ValueEditor
                keyName={editingKey!}
                value={editingValue}
                onChange={setEditingValue}
                onSave={handleSaveEdit}
                onCancel={handleCancelEdit}
              />
            ) : (
              <>
                <KeyValueTable
                  keyValues={keyValues.map(kv => ({
                    ...kv,
                    expiration: formatExpiration(kv.expiration)
                  }))}
                  selectedKeys={selectedKeys}
                  onKeySelect={handleKeySelect}
                  onEdit={handleEdit}
                  onDelete={(keyId) => handleDelete([keyId])}
                  onDeleteSelected={() => handleDelete(selectedKeys)}
                />
                {selectedValue && <ValuePreview value={selectedValue} />}
              </>
            )}
          </main>
        </ResizablePanel>
      </ResizablePanelGroup>
      <Toaster />
    </div>
  );
}