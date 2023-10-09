import { useState } from "react";
import { Link } from "react-router-dom";
import { Auth, Native, Internal } from "@withfig/api-bindings";
import { Button } from "@/components/ui/button";

export default function WhatsNew() {
  const [code, setCode] = useState("");

  return (
    <div className="flex flex-col gap-4">
      <div className="w-full gradient-cw-secondary-light rounded-lg flex flex-col items-start gap-4 text-white p-6">
        <div className="flex flex-col">
          <h1 className="text-xl font-bold drop-shadow">
            CodeWhisperer brings AI to your favorite dev tools
          </h1>
          <p className="drop-shadow">
            We want to be everywhere you work. Not seeing a tool you use?
          </p>
        </div>
        <Link to="/account">
          <Button variant='glass'>Tell us about it</Button>
        </Link>
      </div>

      <div className="w-full">
        <button
          className="bg-violet-200 hover:bg-violet-300 p-2 rounded"
          onClick={async () => {
            setCode("LOADING");

            const init = await Auth.builderIdStartDeviceAuthorization();
            setCode(init.code);

            await Native.open(init.url);

            await Auth.builderIdPollCreateToken(init).catch(console.error);
            setCode("Logged in!");

            await Internal.sendWindowFocusRequest({});
          }}
        >
          Builder ID
        </button>

        {code}
      </div>
    </div>
  );
}
