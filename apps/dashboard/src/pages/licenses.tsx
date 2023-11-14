import { useEffect, useState } from 'react';

type Cargo = {
  authors: string
  description: string
  license: string
  license_file: null
  name: string
  repository: string
  version: string
}

type Npm = {
  author?: string
  description: string
  homepage: string
  license: string
  name: string
  path: string
  version: string
}

export default function Page() {
  const [npm, setNpm] = useState<Npm[]>([])
  const [cargo, setCargo] = useState<Cargo[]>([])

  useEffect(() => {
    fetch('/assets/license/npm.json')
  .then(response => {
      if (!response.ok) {
          throw new Error("HTTP error " + response.status);
      }
      return response.json();
  })
  .then(json => {

    // @ts-expect-error whining about item
    const flatJson = [...Object.entries(json).map(e => e[1]).flat()].filter((item) => item.name.includes('aws-'))
    console.log({ npm: flatJson })
    // @ts-expect-error idk why it's mad about this... they're all the same type.
    setNpm(flatJson)
  })
  .catch((e) => {
      console.error(e)
  })
  }, [])

  useEffect(() => {
    fetch('/assets/license/cargo.json')
  .then(response => {
      if (!response.ok) {
          throw new Error("HTTP error " + response.status);
      }
      return response.json();
  })
  .then(json => {
    console.log({ cargo: json })
    // @ts-expect-error whining about item
    setCargo(json.filter((item) => !item.name.includes('aws-')))
  })
  .catch((e) => {
      console.error(e)
  })
  }, [])

  if (!cargo || !npm) {
    return (
      <>
        <span>Loading...</span>
      </>
    )
  }

  return (
    <>
      <section className={`flex flex-col py-4`}>
        <h2
          id={`subhead-licenses`}
          className="font-bold text-medium text-zinc-400 leading-none mt-2"
        >
          Licenses
        </h2>
        <div className={`flex p-4 pl-0 gap-4`}>
          <div className="flex flex-col gap-1">
            <ul className="flex flex-col gap-4">
              {cargo.map((l, i) => {
                if (!l.license) return null

                return (
                  <li key={i} className='flex flex-col'>
                    <span>{l.name} v{l.version}</span>
                    <span className="text-sm text-black/50">
                      <span>{l.license}</span>
                      {l.authors && <span> · © {l.authors} 2023</span>}
                    </span>
                  </li>
                )
              })}
              {npm.map((l, i) => {
                if (!l.license) return null
                
                return (
                  <li key={i} className='flex flex-col'>
                    <span>{l.name} v{l.version}</span>
                    <span className="text-sm text-black/50">
                      <span>{l.license}</span> 
                      {l.author && <span> · © {l.author} 2023</span>}
                    </span>
                  </li>
                )
              })}
            </ul>
          </div>
        </div>
      </section>
    </>
  );
}
