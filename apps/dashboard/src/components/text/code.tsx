import { cn } from "@/lib/utils"

export function Code ({ children, className }: {
  children: React.ReactNode,
  className?: string
}) {
  return (
    <code className={cn("text-[0.9em] px-1 bg-slate-50 border border-slate-200 rounded-sm text-slate-600", className)}>
      {children}
    </code>
  )
}