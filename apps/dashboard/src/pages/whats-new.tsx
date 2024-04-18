import { Link } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { PRODUCT_NAME } from "@/lib/constants";

export default function WhatsNew() {
  return (
    <div className="flex flex-col gap-4">
      <div className="w-full gradient-cw-secondary-light rounded-lg flex flex-col items-start gap-4 text-white p-6">
        <div className="flex flex-col">
          <h1 className="text-xl font-bold drop-shadow">
            {PRODUCT_NAME} brings AI to your favorite dev tools
          </h1>
          <p className="drop-shadow">
            We want to be everywhere you work. Not seeing a tool you use?
          </p>
        </div>
        <Link to="/account">
          <Button variant="glass">Tell us about it</Button>
        </Link>
      </div>
    </div>
  );
}
