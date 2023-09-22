import { Link } from "react-router-dom";

export default function WhatsNew() {
  return (
    <div className="flex flex-col gap-4">
      <div className="w-full h-40 gradient-cw-secondary-light rounded-lg flex flex-col items-center justify-center text-white px-4 py-2">
        <h1 className="text-xl font-bold drop-shadow">Welcome to AWS CodeWhisperer</h1>
        <p className="drop-shadow">
          This is an internal beta version, goto{" "}
          <Link to="/account" className="underline">Account</Link> to setup your account.
        </p>
      </div>
    </div>
  );
}
