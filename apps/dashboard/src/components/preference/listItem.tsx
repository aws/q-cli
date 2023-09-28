export function Setting ({ type, title, desc, example }: { type: string, title: string, desc: string, example?: string }) {
    return(
        <div className="flex p-4 gap-4">
                <div className="w-12">
                    {type === 'boolean' && <Boolean />}
                </div>
                <div className="flex flex-col">
                    <h2>{title}</h2>
                    {desc && <p>{desc}</p>}
                    {example && <p>{example}</p>}
                    {type !== 'boolean' && (
                        <div></div>
                    )}
                </div>
        </div>
    )
}