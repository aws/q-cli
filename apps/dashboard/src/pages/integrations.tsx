import { UserPrefSection } from "@/components/preference/list";
import settings from "@/data/integrations";


export default function Page() {
  return (
    <>
      {settings.map((section, i) => <UserPrefSection data={section} index={i} key={i} />)}
    </>
  );
}
