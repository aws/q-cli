import { UserPrefView } from "@/components/preference/list";
import settings from "@/data/ghostText";

export default function Page() {
  return (
    <UserPrefView array={settings} />
  );
}
