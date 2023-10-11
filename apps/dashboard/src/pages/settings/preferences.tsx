import { UserPrefView } from "@/components/preference/list";
import settings from "@/data/preferences";

export default function Page() {
  return (
    <UserPrefView array={settings} />
  );
}
