import StatusCheck from "@/components/installs/statusCheck";
import { Code } from "@/components/text/code";
import support from "@/data/help";
import installChecks from "@/data/install";

export default function Page() {
  return (
    <div className="flex flex-col items-start gap-8 pb-4">
      <div className="flex flex-col justify-between gap-2 self-stretch">
        <h1 className="text-3xl font-black select-none">Automated checks</h1>
        <div className="flex flex-col">
          {installChecks.map((check) => {
            return <StatusCheck check={check} key={check.id} />;
          })}
        </div>
      </div>
      <div className="flex flex-col justify-between gap-4 self-stretch">
        <h1 className="text-3xl font-black select-none mb-2">
          Still having issues?
        </h1>
        <div className="flex flex-col gap-4">
          <ol className="flex flex-col gap-2">
            {support.steps.map((step, i) => {
              const stringAsArray = step.split("`");
              return (
                <li key={i} className="flex gap-2 items-baseline">
                  <span>{i + 1}.</span>
                  {stringAsArray.map((substr, i) => {
                    if (i === 1) {
                      return (
                        <Code>
                          {substr}
                        </Code>
                      );
                    }

                    return <span key={i}>{substr}</span>;
                  })}
                </li>
              );
            })}
          </ol>
          <div className="flex flex-col">
            <span className="text-slate-500">
              You can also check out the following:
            </span>
            <div className="flex gap-4">
              {support.links.map((link, i) => {
                return (
                  <a
                    key={i}
                    href={link.url}
                    className="text-blue-500 font-medium"
                  >
                    {link.text} →
                  </a>
                );
              })}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
