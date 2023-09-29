import { useState } from "react";
import { Link } from "react-router-dom";
import { Auth, Native, Internal } from "@withfig/api-bindings";

export default function WhatsNew() {
  const [code, setCode] = useState("");

  return (
    <div className="flex flex-col gap-4">
      <div className="w-full h-40 gradient-cw-secondary-light rounded-lg flex flex-col items-center justify-center text-white px-4 py-2">
        <h1 className="text-xl font-bold drop-shadow">
          Welcome to AWS CodeWhisperer
        </h1>
        <p className="drop-shadow">
          This is an internal beta version, goto{" "}
          <Link to="/account" className="underline">
            Account
          </Link>{" "}
          to setup your account.
        </p>
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
