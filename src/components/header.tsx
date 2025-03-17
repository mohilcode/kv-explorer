import { Button } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'
import { Cloud, Folder, RefreshCw } from 'lucide-react'

interface HeaderProps {
  title: string
  version: string
  localFolders: { id: number; name: string; path: string }[]
  onFolderSelect: () => void
  onRemoteConnect: () => void
  onRemoteDisconnect: () => void
  remoteConnections: number
  isLoading: boolean
}

export function Header({
  title,
  version,
  localFolders,
  onFolderSelect,
  onRemoteConnect,
  onRemoteDisconnect,
  remoteConnections,
  isLoading,
}: HeaderProps) {
  return (
    <header className="flex flex-col border-b border-zinc-800">
      <div className="flex h-14 items-center px-4 lg:px-6">
        <img
          src="/kv-icon.png"
          alt="KV Explorer"
          className="h-8 w-8 mr-2"
        />
        <h1 className="text-xl font-bold tracking-wider">{title}</h1>
        <span className="ml-2 text-xs text-zinc-500">{version}</span>
        <Separator orientation="vertical" className="mx-4 h-6" />

        <Button
          onClick={onFolderSelect}
          variant="outline"
          className="gap-2 border-zinc-700 bg-transparent hover:bg-zinc-900 cursor-pointer"
          disabled={isLoading}
        >
          <Folder className="h-4 w-4" />
          <span>ADD WRANGLER FOLDER</span>
        </Button>

        <Separator orientation="vertical" className="mx-4 h-6" />

        {remoteConnections > 0 ? (
          <Button
            onClick={onRemoteDisconnect}
            variant="outline"
            className="gap-2 border-red-700 bg-transparent hover:bg-red-900/30 cursor-pointer"
            disabled={isLoading}
          >
            <Cloud className="h-4 w-4 text-red-500" />
            <span>REMOVE CONNECTION</span>
          </Button>
        ) : (
          <Button
            onClick={onRemoteConnect}
            variant="outline"
            className="gap-2 border-zinc-700 bg-transparent hover:bg-zinc-900 cursor-pointer"
            disabled={isLoading}
          >
            <Cloud className="h-4 w-4 text-cyan-500" />
            <span>CONNECT REMOTE KV</span>
          </Button>
        )}

        <div className="ml-4 flex-1 truncate text-sm">
          {localFolders.length > 0 && (
            <div className="truncate text-zinc-400">
              <span className="font-bold mr-1">LOCAL:</span> {localFolders.length} folder
              {localFolders.length !== 1 ? 's' : ''}
            </div>
          )}
          {remoteConnections > 0 && (
            <div className="truncate text-cyan-400">
              <span className="font-bold mr-1">REMOTE:</span> {remoteConnections} connection
              {remoteConnections !== 1 ? 's' : ''}
            </div>
          )}
          {localFolders.length === 0 && remoteConnections === 0 && (
            <div className="text-zinc-500">NO CONNECTION</div>
          )}
        </div>

        {isLoading && (
          <div className="ml-auto flex items-center gap-2">
            <RefreshCw className="h-4 w-4 animate-spin" />
            <span className="text-sm">LOADING...</span>
          </div>
        )}
      </div>
      <div className="flex px-4 text-xs text-zinc-500">
        <div className="flex-1">WRANGLER KV EXPLORER</div>
        <div className="flex items-center gap-2">
          <span>
            Last updated: {new Date().toLocaleDateString()}, {new Date().toLocaleTimeString()}
          </span>
        </div>
      </div>
    </header>
  )
}