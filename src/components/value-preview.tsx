import { ScrollArea } from "@/components/ui/scroll-area"

interface ValuePreviewProps {
  value: any
}

export function ValuePreview({ value }: ValuePreviewProps) {
  const formattedValue = JSON.stringify(value, null, 2)

  return (
    <div className="flex flex-col h-full border-t border-zinc-800">
      <div className="flex-none flex items-center justify-between border-b border-zinc-800 p-4">
        <h2 className="text-lg font-bold tracking-wider">VALUE PREVIEW</h2>
      </div>
      <ScrollArea className="flex-1 h-[calc(100%-4rem)]">
        <pre className="p-4 font-mono text-sm text-green-400">
          <code className="language-json">{formattedValue}</code>
        </pre>
      </ScrollArea>
    </div>
  )
}