import { UserPrefView } from "@/components/preference/list";
import settings, { intro} from "@/data/ghostText";

export default function Page() {
  return (
    <UserPrefView array={settings} intro={intro} />
  );
}
