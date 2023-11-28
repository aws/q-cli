import { UserPrefView } from "@/components/preference/list";
import { Button } from "@/components/ui/button";
import settings from "@/data/preferences";
import { Auth, Native, User } from "@withfig/api-bindings";
import { useEffect, useState } from "react";

export default function Page() {
  const [authed, setAuthed] = useState<boolean>(false);
  const [accountType, setAccountType] = useState<string | undefined>();

  useEffect(() => {
    Auth.status().then((s) => {
      setAuthed(s.authed);
      if (!s.authed) return;

      if (s.authKind === "BuilderId") {
        setAccountType("Builder ID");
        return;
      }

      if (s.authKind === "IamIdentityCenter") {
        setAccountType("IAM Identity Center");
        return;
      }

      setAccountType(s.authKind);
      return;
    });
  }, []);

  function logout() {
    User.logout().then(() => {
      window.location.pathname = "/";
      window.location.reload();
    });
  }

  return (
    <>
      <UserPrefView array={settings} />
      <section className={`flex flex-col py-4`}>
        <h2
          id={`subhead-account`}
          className="font-bold text-medium text-zinc-400 leading-none mt-2"
        >
          Account
        </h2>
        <div className={`flex p-4 pl-0 gap-4`}>
          <div className="flex flex-col gap-1">
            <h3 className="font-medium leading-none">Account type</h3>
            <p className="font-light leading-tight text-sm">
              Users can log in with either AWS Builder ID or AWS IAM Identity
              Center
            </p>
            <p className="font-light leading-tight text-sm text-black/50 dark:text-white/50">
              {authed
                ? accountType
                  ? `Logged in with ${accountType}`
                  : "Logged in"
                : "Not logged in"}
            </p>
            <div className="pt-2">
              <Button
                variant="outline"
                onClick={() => logout()}
                disabled={!authed}
              >
                Log out
              </Button>
            </div>
          </div>
        </div>
      </section>
      <section className={`py-4 gap-4`}>
        <h2
          id={`subhead-licenses`}
          className="font-bold text-medium text-zinc-400 leading-none mt-2"
        >
          Licenses
        </h2>
        <Button
          variant="link"
          className="px-0 text-blue-500 hover:underline decoration-1 underline-offset-1 hover:text-blue-800 hover:underline-offset-4 transition-all duration-100 text-sm"
          onClick={() => {
            Native.open(
              "file:///Applications/CodeWhisperer.app/Contents/Resources/dashboard/license/NOTICE.txt"
            );
          }}
        >
          View licenses
        </Button>
      </section>
    </>
  );
}
