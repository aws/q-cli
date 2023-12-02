import { Button } from "@/components/ui/button";

type Integration = {
  id: string;
  title: string;
  background?: {
    color: string;
    image: string;
  };
  description?: string;
};

export default function IntegrationCard({
  integration,
  enabled,
}: {
  integration: Integration;
  enabled: boolean;
}) {
  const integrationId = integration.id.split(".")[1];

  return (
    <div className="flex flex-col relative overflow-hidden rounded-lg p-4 gap-4 max-w-xs border border-black/10 col-span-1">
      <div
        style={{
          backgroundImage: `url(${`/images/integrations/bg/${integrationId}.svg`})`,
        }}
        className="absolute left-0 right-0 bottom-1/2 w-full h-full bg-no-repeat bg-cover"
      />
      <div className="flex flex-col relative z-10 items-center text-center">
        <h3 className="text-white font-ember text-lg font-bold">
          {integration.title}
        </h3>
        <img
          className="h-40 w-40"
          src={`/images/integrations/icons/${integrationId}.png`}
        />
      </div>
      <p className="text-black/50 self-center text-center">
        {integration.description}
      </p>
      {enabled === true ? (
        <Button variant={"outline"}>Enabled</Button>
      ) : (
        <Button variant={"default"}>Enable</Button>
      )}
    </div>
  );
}
