import { ScrollArea } from "@/components/ui/scroll-area"
import { cn } from "@/lib/utils"

interface Namespace {
  id: string
  name: string
  count: number
}

interface NamespaceSidebarProps {
  namespaces: Namespace[]
  selectedNamespace: string | null
  onNamespaceSelect: (namespaceId: string) => void
}

export function NamespaceSidebar({ namespaces, selectedNamespace, onNamespaceSelect }: NamespaceSidebarProps) {
  return (
    <div className="h-full flex flex-col border-r border-zinc-800 bg-black">
      <div className="border-b border-zinc-800 p-4 font-bold tracking-wider text-zinc-300">KV NAMESPACES</div>
      <ScrollArea className="h-full">
        <div className="p-2">
          {namespaces.map((namespace) => (
            <button
              key={namespace.id}
              className={cn(
                "flex w-full items-center justify-between rounded-none border border-transparent px-3 py-2 text-sm transition-colors hover:border-zinc-700 hover:bg-zinc-900",
                selectedNamespace === namespace.id && "border-zinc-700 bg-zinc-900 font-medium",
              )}
              onClick={() => onNamespaceSelect(namespace.id)}
            >
              <span className="break-words hyphens-auto mr-2">{namespace.name}</span>
              <span className="flex-shrink-0 text-xs text-zinc-500">({namespace.count})</span>
            </button>
          ))}
        </div>
      </ScrollArea>
    </div>
  )
}