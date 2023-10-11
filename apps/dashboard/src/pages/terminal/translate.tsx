import { UserPrefSection, UserPrefView } from "@/components/preference/list";
import settings from "@/data/translate";


export default function Page() {
  return (
    <UserPrefView array={settings} />
  );
}
