import type React from "react"
import { useState } from "react"
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Cloud, Key, User } from "lucide-react"

interface RemoteConnectionModalProps {
  isOpen: boolean
  onClose: () => void
  onSave: (accountId: string, apiToken: string) => void
}

export function RemoteConnectionModal({ isOpen, onClose, onSave }: RemoteConnectionModalProps) {
  const [accountId, setAccountId] = useState("")
  const [apiToken, setApiToken] = useState("")
  const [isTokenVisible, setIsTokenVisible] = useState(false)

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (accountId.trim() && apiToken.trim()) {
      onSave(accountId, apiToken)
      setAccountId("")
      setApiToken("")
      setIsTokenVisible(false)
    }
  }

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent className="border-zinc-800 bg-black font-mono text-white sm:max-w-md">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2 text-lg font-bold tracking-wider">
            <Cloud className="h-5 w-5 text-cyan-500" />
            CONNECT TO REMOTE KV
          </DialogTitle>
        </DialogHeader>
        <form onSubmit={handleSubmit}>
          <div className="grid gap-6 py-4">
            <div className="grid gap-2">
              <Label
                htmlFor="accountId"
                className="flex items-center gap-2 text-xs font-bold tracking-wider text-zinc-400"
              >
                <User className="h-3.5 w-3.5" />
                CLOUDFLARE ACCOUNT ID
              </Label>
              <Input
                id="accountId"
                value={accountId}
                onChange={(e) => setAccountId(e.target.value)}
                className="border-zinc-800 bg-zinc-900 font-mono text-sm focus-visible:ring-zinc-700"
                placeholder="Enter your Cloudflare account ID"
                required
              />
            </div>
            <div className="grid gap-2">
              <Label
                htmlFor="apiToken"
                className="flex items-center gap-2 text-xs font-bold tracking-wider text-zinc-400"
              >
                <Key className="h-3.5 w-3.5" />
                API TOKEN
              </Label>
              <div className="relative">
                <Input
                  id="apiToken"
                  type={isTokenVisible ? "text" : "password"}
                  value={apiToken}
                  onChange={(e) => setApiToken(e.target.value)}
                  className="border-zinc-800 bg-zinc-900 font-mono text-sm focus-visible:ring-zinc-700"
                  placeholder="Enter your API token"
                  required
                />
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  className="absolute right-0 top-0 h-full px-3 hover:bg-transparent hover:text-white cursor-pointer"
                  onClick={() => setIsTokenVisible(!isTokenVisible)}
                >
                  {isTokenVisible ? "HIDE" : "SHOW"}
                </Button>
              </div>
              <p className="text-xs text-zinc-500">Requires Workers KV access permissions</p>
            </div>
          </div>
          <DialogFooter className="border-t border-zinc-800 pt-4">
            <Button
              type="button"
              variant="outline"
              onClick={onClose}
              className="border-zinc-700 bg-transparent hover:bg-zinc-900 cursor-pointer"
            >
              CANCEL
            </Button>
            <Button type="submit" className="bg-cyan-700 hover:bg-cyan-600 cursor-pointer">
              CONNECT
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

