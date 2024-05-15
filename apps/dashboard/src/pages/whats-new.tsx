import { Link } from "@/components/ui/link";
import feedData from "../../../../feed.json";
import { useState } from "react";

export default function WhatsNew() {
  const [showMoreStates, setShowMoreStates] = useState<boolean[]>(
    feedData.map(() => false),
  );

  const toggleShowMoreStates = (index: number) => {
    setShowMoreStates((prevStates) => {
      const updatedStates = [...prevStates];
      updatedStates[index] = !updatedStates[index];
      return updatedStates;
    });
  };

  return (
    <div className="flex flex-row gap-4 relative mt-4 h-full">
      <div className=" w-0.5 left-[0.3rem] mt-6 h-full absolute z-0 bg-zinc-200 dark:bg-zinc-700"></div>
      <div className="flex flex-col item">
        {feedData.map((item, itemIndex) => (
          <div className="flex flex-row gap-4 pb-10">
            <div className="h-5 rounded-full mt-5 p-1.5 bg-white dark:bg-zinc-800 z-20 absolute" />
            <div className="h-2 rounded-full mt-6 p-1.5 bg-dusk-600 z-20" />
            <div className="flex flex-col">
              <div className="text-xs font-mono text-dusk-600 dark:text-dusk-400 select-none">
                {item.kind == "announcement"
                  ? "Product Announcement"
                  : "Release Notes"}
              </div>
              <h1 className="text-xl font-bold select-none">{item.title}</h1>
              <div className="text-sm mb-1">{item.description}</div>
              {item.changes && item.changes?.length > 5 ? (
                showMoreStates[itemIndex] ? (
                  <div>
                    <div className="relative">
                      <ul className="text-sm ml-4">
                        {item.changes
                          ?.sort((a, b) => a.kind.localeCompare(b.kind))
                          .slice(0, 5)
                          .map((change) => (
                            <li>
                              <b>• {change.kind}</b>: {change.description}
                            </li>
                          ))}
                      </ul>
                      <div className="w-full h-[60%] bg-gradient-to-t from-white dark:from-zinc-800 bottom-0 absolute" />
                    </div>
                    <button
                      onClick={() => toggleShowMoreStates(itemIndex)}
                      className="text-sm hover:opacity-70 text-dusk-600 dark:text-dusk-400"
                    >
                      Show more →
                    </button>
                  </div>
                ) : (
                  <div>
                    <div className="relative">
                      <ul className="text-sm ml-4">
                        {item.changes
                          ?.sort((a, b) => a.kind.localeCompare(b.kind))
                          .map((change) => (
                            <li>
                              <b>• {change.kind}</b>: {change.description}
                            </li>
                          ))}
                      </ul>
                    </div>
                    <button
                      onClick={() => toggleShowMoreStates(itemIndex)}
                      className="text-sm hover:opacity-70 text-dusk-600 dark:text-dusk-400"
                    >
                      Show less ←
                    </button>
                  </div>
                )
              ) : (
                <ul className="text-sm ml-4">
                  {item.changes
                    ?.sort((a, b) => a.kind.localeCompare(b.kind))
                    .map((change) => (
                      <li>
                        <b>• {change.kind}</b>: {change.description}
                      </li>
                    ))}
                </ul>
              )}

              {item.link ? (
                <Link
                  className="text-sm inline-block no-underline text-dusk-600 dark:text-dusk-400"
                  href={item.link}
                >
                  Learn more →
                </Link>
              ) : (
                <></>
              )}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
