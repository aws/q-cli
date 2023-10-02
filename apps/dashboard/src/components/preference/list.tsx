import { alphaByTitle } from "@/lib/sort"
import { Pref } from "@/types/preferences"
import { Setting } from "./listItem"

type PrefSection = {
  title: string,
  properties: Pref[]
}

export function UserPrefSection ({data, index}: {data: PrefSection, index: number}) {
  return(
    <section className="flex flex-col">
      <h1 id={`subhead-${index}`} className="font-bold text-2xl leading-none mt-2">{data.title}</h1>
      
      {data.properties.sort(alphaByTitle).map((p, i) => {
        if (p.popular) return
        
        return (
          <Setting data={p} key={i} />
        )
      })}
    </section>
  )
}