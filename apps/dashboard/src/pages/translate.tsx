import { UserPrefSection } from "@/components/preference/list";
import settings from "@/data/translate";


export default function Page() {
  console.log(settings)
  return (
    <>
      {settings.map((section, i) => <UserPrefSection data={section} index={i} key={i} />)}
    </>
  );
}
