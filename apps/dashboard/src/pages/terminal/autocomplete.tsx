import { UserPrefSection, UserPrefView } from "@/components/preference/list";
import { Setting } from "@/components/preference/listItem";
import settings from "@/data/autocomplete";
import { alphaByTitle } from "@/lib/sort";


export default function Page() {
  const popular = (settings).map((s) => {
   return s.properties.filter((p) => p.popular)
  }).flat()
  return (
    <UserPrefView array={settings}>
      <section className="flex flex-col">
        <h1 id={`subhead-popular`} className="font-bold text-2xl leading-none mt-2">Popular</h1>
        {popular.sort(alphaByTitle).map((p, i) => <Setting data={p} key={i} />)}
      </section>
    </UserPrefView>
  );
}
