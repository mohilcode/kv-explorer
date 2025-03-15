import { useState } from "react"
import { Edit, Trash2 } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Checkbox } from "@/components/ui/checkbox"
import { ScrollArea } from "@/components/ui/scroll-area"
import { cn } from "@/lib/utils"

interface KeyValue {
  id: string
  key: string
  expiration: string
  value: any
}

interface KeyValueTableProps {
  keyValues: KeyValue[]
  selectedKeys: string[]
  onKeySelect: (keyId: string) => void
  onEdit: (keyId: string) => void
  onDelete: (keyId: string) => void
  onDeleteSelected: () => void
}

export function KeyValueTable({
  keyValues,
  selectedKeys,
  onKeySelect,
  onEdit,
  onDelete,
  onDeleteSelected,
}: KeyValueTableProps) {
  const [sortField, setSortField] = useState<"key" | "expiration">("key")
  const [sortDirection, setSortDirection] = useState<"asc" | "desc">("asc")
  const [hoveredRow, setHoveredRow] = useState<string | null>(null)

  const handleSort = (field: "key" | "expiration") => {
    if (sortField === field) {
      setSortDirection(sortDirection === "asc" ? "desc" : "asc")
    } else {
      setSortField(field)
      setSortDirection("asc")
    }
  }

  const sortedKeyValues = [...keyValues].sort((a, b) => {
    if (sortField === "key") {
      return sortDirection === "asc" ? a.key.localeCompare(b.key) : b.key.localeCompare(a.key)
    } else {
      return sortDirection === "asc"
        ? a.expiration.localeCompare(b.expiration)
        : b.expiration.localeCompare(a.expiration)
    }
  })

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between border-b border-zinc-800 p-4">
        <h2 className="text-lg font-bold tracking-wider">KEY-VALUE PAIRS</h2>
        {selectedKeys.length > 0 && (
          <Button
            variant="destructive"
            size="sm"
            onClick={onDeleteSelected}
            className="gap-2 bg-red-900 hover:bg-red-800 cursor-pointer"
          >
            <Trash2 className="h-4 w-4" />
            DELETE SELECTED ({selectedKeys.length})
          </Button>
        )}
      </div>
      <div className="grid grid-cols-[auto_1fr_auto_auto] gap-x-4 border-b border-zinc-800 px-4 py-3 font-bold text-zinc-300">
        <div className="flex items-center">
          <Checkbox
            checked={selectedKeys.length === keyValues.length && keyValues.length > 0}
            onCheckedChange={(checked) => {
              if (checked) {
                onKeySelect("all")
              } else {
                onKeySelect("none")
              }
            }}
            className="border-zinc-700 data-[state=checked]:bg-zinc-700 data-[state=checked]:text-white"
          />
        </div>
        <button className="flex items-center gap-1 text-left" onClick={() => handleSort("key")}>
          KEY
          <span className="text-xs">{sortField === "key" && (sortDirection === "asc" ? "▲" : "▼")}</span>
        </button>
        <button className="flex items-center gap-1" onClick={() => handleSort("expiration")}>
          EXPIRATION
          <span className="text-xs">{sortField === "expiration" && (sortDirection === "asc" ? "▲" : "▼")}</span>
        </button>
        <div>ACTIONS</div>
      </div>
      <ScrollArea className="flex-1">
        {sortedKeyValues.map((kv) => (
          <div
            key={kv.id}
            className={cn(
              "grid grid-cols-[auto_1fr_auto_auto] gap-x-4 border-b border-zinc-800 px-4 py-3",
              hoveredRow === kv.id && !selectedKeys.includes(kv.id) && "bg-zinc-900/30",
              selectedKeys.includes(kv.id) && "bg-zinc-900 border-l-2 border-l-zinc-500",
            )}
            onMouseEnter={() => setHoveredRow(kv.id)}
            onMouseLeave={() => setHoveredRow(null)}
          >
            <div className="flex items-center">
              <Checkbox
                checked={selectedKeys.includes(kv.id)}
                onCheckedChange={() => onKeySelect(kv.id)}
                className="border-zinc-700 data-[state=checked]:bg-zinc-700 data-[state=checked]:text-white"
              />
            </div>
            <div
              className="cursor-pointer truncate font-mono text-sm"
              onClick={() => onKeySelect(kv.id)}
            >
              {kv.key}
            </div>
            <div className="text-sm text-zinc-500">{kv.expiration}</div>
            <div className="flex gap-2">
              <Button
                variant="ghost"
                size="icon"
                onClick={() => onEdit(kv.id)}
                className="hover:bg-zinc-800 hover:text-white cursor-pointer"
              >
                <Edit className="h-4 w-4" />
                <span className="sr-only">Edit</span>
              </Button>
              <Button
                variant="ghost"
                size="icon"
                onClick={() => onDelete(kv.id)}
                className="hover:bg-zinc-800 hover:text-white cursor-pointer"
              >
                <Trash2 className="h-4 w-4" />
                <span className="sr-only">Delete</span>
              </Button>
            </div>
          </div>
        ))}
      </ScrollArea>
    </div>
  )
}