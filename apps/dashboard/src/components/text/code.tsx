export function Code ({ children }: {
  children: React.ReactNode
}) {
  return (
    <code className="text-[0.9em] px-1 bg-slate-50 border border-slate-200 rounded-sm text-slate-600">
      {children}
    </code>
  )
}