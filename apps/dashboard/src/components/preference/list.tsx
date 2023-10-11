import { alphaByTitle } from "@/lib/sort"
import { Action, Pref } from "@/types/preferences"
import { Setting } from "./listItem"

type PrefSection = {
  title: string,
  properties?: Pref[]
  actions?: Action[]
}

export function UserPrefSection ({data, index}: {data: PrefSection, index: number}) {
  const list = data.properties ?? data.actions
  
  return(
    <section className="flex flex-col">
      <h1 id={`subhead-${index}`} className="font-bold text-2xl leading-none mt-2">{data.title}</h1>
      
      {list?.sort(alphaByTitle).map((p, i) => {
        if (p.popular) return
        
        return (
          <Setting data={p} key={i} />
        )
      })}
    </section>
  )
}

export function UserPrefView ({array, children}: {array: PrefSection[], children?: React.ReactNode}) {
  return (
    <>
      {children}
      {array.map((section, i) => <UserPrefSection data={section} index={i} key={i} />)}
    </>
  )
}