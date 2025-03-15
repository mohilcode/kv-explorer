import type React from "react"

import { useState } from "react"
import { Button } from "@/components/ui/button"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Textarea } from "@/components/ui/textarea"
import { useToast } from "@/hooks/use-toast"

interface ValueEditorProps {
  keyName: string
  value: any
  onChange: (value: any) => void
  onSave: () => void
  onCancel: () => void
}

export function ValueEditor({ keyName, value, onChange, onSave, onCancel }: ValueEditorProps) {
  const [error, setError] = useState<string | null>(null)
  const { toast } = useToast()
  const formattedValue = JSON.stringify(value, null, 2)

  const handleChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    try {
      const parsed = JSON.parse(e.target.value)
      onChange(parsed)
      setError(null)
    } catch (err) {
      setError("Invalid JSON format")
    }
  }

  const handleSave = () => {
    if (error) {
      toast({
        title: "ERROR",
        description: "Please fix the JSON errors before saving",
        variant: "destructive",
      })
      return
    }
    onSave()
  }

  return (
    <div className="flex flex-1 flex-col">
      <div className="flex items-center justify-between border-b border-zinc-800 p-4">
        <h2 className="text-lg font-bold tracking-wider">
          EDITING: <span className="font-mono text-green-400">{keyName}</span>
        </h2>
        <div className="flex gap-2">
          <Button variant="outline" onClick={onCancel} className="border-zinc-700 bg-transparent hover:bg-zinc-900 cursor-pointer">
            CANCEL
          </Button>
          <Button onClick={handleSave} className="bg-white text-black hover:bg-zinc-400 cursor-pointer">
            SAVE
          </Button>
        </div>
      </div>
      <div className="flex flex-1 flex-col p-4">
        {error && (
          <div className="mb-2 rounded-none border border-red-900 bg-red-900/20 p-2 text-sm text-red-500">{error}</div>
        )}
        <ScrollArea className="flex-1">
          <Textarea
            className="min-h-[60vh] resize-none border-zinc-700 bg-black font-mono text-sm text-green-400 focus-visible:ring-zinc-700"
            value={formattedValue}
            onChange={handleChange}
          />
        </ScrollArea>
      </div>
    </div>
  )
}

