import { useEffect } from 'react'
import { Toaster } from '@/components/ui/toaster'
import { GridBackground } from '@/components/grid-background'
import { KVExplorer } from '@/components/kv-explorer'

export default function App() {
  useEffect(() => {
    document.documentElement.classList.add('dark')
  }, [])

  return (
    <div className="relative flex h-screen flex-col text-white font-mono">
      <GridBackground />
      <KVExplorer />
      <Toaster />
    </div>
  )
}