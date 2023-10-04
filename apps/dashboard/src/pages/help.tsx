import StatusCheck from "@/components/installs/statusCheck";
import installChecks from "@/data/install";

export default function Page() {

  return (
    <div className="flex flex-col items-start">
      <div className="flex justify-between gap-4 self-stretch">
        <h1 className="text-3xl font-black select-none mb-2">
          Automated checks
        </h1>
      </div>
      {installChecks.map((check) => {
        return <StatusCheck check={check} key={check.id} />;
      })}
    </div>
  );
}
