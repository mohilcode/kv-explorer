import { ScrollArea } from '@/components/ui/scroll-area'
import { cn } from '@/lib/utils'

interface Namespace {
  id: string
  name: string
  count?: number
  entries: unknown[]
  type: string
  accountId?: string
}

interface NamespaceSidebarProps {
  namespaces: Namespace[]
  selectedNamespace: string | null
  onNamespaceSelect: (namespaceId: string) => void
}

export function NamespaceSidebar({
  namespaces,
  selectedNamespace,
  onNamespaceSelect,
}: NamespaceSidebarProps) {
  const localNamespaces = namespaces.filter(ns => ns.type === 'local')
  const remoteNamespaces = namespaces.filter(ns => ns.type === 'remote')

  const getNamespaceCount = (namespace: Namespace) => {
    if (namespace.count !== undefined) {
      return namespace.count
    }
    return namespace.entries.length
  }

  return (
    <div className="h-full flex flex-col border-r border-zinc-800 bg-black">
      <div className="border-b border-zinc-800 p-4 font-bold tracking-wider text-zinc-300">
        KV NAMESPACES
      </div>
      <ScrollArea className="h-full">
        <div className="p-2">
          {localNamespaces.length > 0 && (
            <>
              <div className="px-3 py-2 text-xs font-bold text-zinc-500">LOCAL</div>
              {localNamespaces.map(namespace => (
                <button
                  type="button"
                  key={namespace.id}
                  className={cn(
                    'flex w-full items-center justify-between rounded-none border border-transparent px-3 py-2 text-sm transition-colors hover:border-zinc-700 hover:bg-zinc-900',
                    selectedNamespace === namespace.id && 'border-zinc-700 bg-zinc-900 font-medium'
                  )}
                  onClick={() => onNamespaceSelect(namespace.id)}
                >
                  <span className="break-words hyphens-auto mr-2">{namespace.name}</span>
                  <span className="flex-shrink-0 text-xs text-zinc-500">
                    ({getNamespaceCount(namespace)})
                  </span>
                </button>
              ))}
            </>
          )}

          {remoteNamespaces.length > 0 && (
            <>
              <div className="mt-4 px-3 py-2 text-xs font-bold text-cyan-500">REMOTE</div>
              {remoteNamespaces.map(namespace => (
                <button
                  type="button"
                  key={namespace.id}
                  className={cn(
                    'flex w-full items-center justify-between rounded-none border border-transparent px-3 py-2 text-sm transition-colors hover:border-cyan-800 hover:bg-cyan-900/30',
                    selectedNamespace === namespace.id &&
                      'border-cyan-700 bg-cyan-900/30 font-medium'
                  )}
                  onClick={() => onNamespaceSelect(namespace.id)}
                >
                  <span className="break-words hyphens-auto mr-2 text-cyan-300">
                    {namespace.name}
                  </span>
                  {namespace.count !== undefined ? (
                    <span className="flex-shrink-0 text-xs text-cyan-500">({namespace.count})</span>
                  ) : (
                    <span className="flex-shrink-0 text-xs text-cyan-500">(?)</span>
                  )}
                </button>
              ))}
            </>
          )}
        </div>
      </ScrollArea>
    </div>
  )
}
