import { Folder, RefreshCw } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Separator } from "@/components/ui/separator"

interface HeaderProps {
  title: string
  version: string
  selectedFolder: string | null
  onFolderSelect: () => void
  isLoading: boolean
}

export function Header({ title, version, selectedFolder, onFolderSelect, isLoading }: HeaderProps) {
  return (
    <header className="flex flex-col border-b border-zinc-800">
      <div className="flex h-14 items-center px-4 lg:px-6">
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
          <span>SELECT WRANGLER FOLDER</span>
        </Button>
        <div className="ml-4 flex-1 truncate text-sm text-zinc-400">{selectedFolder || "NO FOLDER SELECTED"}</div>
        {isLoading && (
          <div className="ml-auto flex items-center gap-2">
            <RefreshCw className="h-4 w-4 animate-spin" />
            <span className="text-sm">LOADING...</span>
          </div>
        )}
      </div>
      <div className="flex px-4 text-xs text-zinc-500">
        <div className="flex-1">SYSTEM</div>
        <div className="flex items-center gap-2">
          <span>Last updated: Mar 16, 2025, 06:15 AM</span>
          <Button variant="ghost" size="icon" className="h-6 w-6 text-zinc-500 hover:text-white cursor-pointer">
            <RefreshCw className="h-3 w-3" />
          </Button>
        </div>
      </div>
    </header>
  )
}

