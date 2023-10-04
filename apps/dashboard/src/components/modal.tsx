export default function Modal ({children}: {children: React.ReactNode}) {
  return (
    <div className="fixed z-50 inset-0 h-full w-full bg-white/90 flex justify-center items-center">
      <div className="p-4 rounded-lg bg-white flex flex-col shadow-lg w-[400px]">
        {children}
      </div>
    </div>
  )
}