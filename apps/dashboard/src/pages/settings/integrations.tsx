import { UserPrefView } from "@/components/preference/list";
import settings from "@/data/integrations";


export default function Page() {
  return (
    <UserPrefView array={settings} />
  );
}
