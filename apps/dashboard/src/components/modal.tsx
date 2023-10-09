export default function Modal ({children}: {children: React.ReactNode}) {
  return (
    <div className="fixed z-50 inset-0 h-full w-full bg-white/70 backdrop-blur-lg flex justify-center items-center">
      <div className="p-4 rounded-lg bg-white flex flex-col shadow-xl w-[400px]">
        {children}
      </div>
    </div>
  )
}