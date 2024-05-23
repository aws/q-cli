import { useContext, useEffect, useRef, useState, useCallback } from "react";
import logo from "../../../../fig_desktop/icons/32x32.png";
import user from "../../assets/images/user.png";
import stop from "../../assets/images/stop.svg";
import { InvokeModelWithResponseStreamCommand } from "@aws-sdk/client-bedrock-runtime";
import { Screen, WindowPosition, Event } from "@amzn/fig-io-api-bindings";
import Loading from "./Loading";
import Modal from "./Modal";
import {
  client,
  maxImages,
  maxImagesPerTurn,
  Body,
  Prompt,
  useFigSubscriptionEffect,
  readBlobAsDataURL,
} from "./utils";
import Markdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { GeneratingContext } from "./state";

const body: Body = {
  anthropic_version: "bedrock-2023-05-31",
  max_tokens: 4000,
  messages: [],
};

const input = {
  modelId: "anthropic.claude-3-sonnet-20240229-v1:0",
  contentType: "application/json",
  accept: "application/json",
  body: new TextEncoder().encode(""),
};

async function extractImages(div: HTMLDivElement): Promise<string[]> {
  const imgs = div.getElementsByTagName("img");
  const res: string[] = [];

  for (const img of imgs) {
    const response = await fetch(img.src);
    const blob = await response.blob();
    const base64 = await readBlobAsDataURL(blob);
    res.push(base64.replace(/^data:image\/\w+;base64,/, ""));
  }

  return res;
}

const ChatEntry = ({
  text,
  isUser,
  onEnter,
  setImgSources,
}: {
  text: string | null;
  isUser: boolean;
  onEnter: (div: HTMLDivElement) => void;
  setImgSources: React.Dispatch<React.SetStateAction<string[]>>;
}) => {
  const [imgs, setImgs] = useState<string[]>([]);
  const inputRef = useRef<HTMLDivElement>(null);
  const { generating, stopGenerating } = useContext(GeneratingContext);

  // If text === null, then it is a new empty user input
  // If text === "", then it is a new Claude response
  const current = text === null;
  const stopStatus = current && generating;

  useEffect(
    () => document.getElementById("current")!.scrollIntoView(false),
    [text],
  );

  useFigSubscriptionEffect(
    () =>
      Event.subscribe("screenshot", (payload: string | null) => {
        if (current) {
          Screen.getScreenshot(payload ? payload : "ENTIRE").then((newImgs) => {
            setImgs((imgs) => [...imgs, ...newImgs]);
            setImgSources((imgs) => [...imgs, ...newImgs]);
          });
        }
        return { unsubscribe: !current };
      }),
    [current],
  );

  const onContextMenu = (
    e: React.MouseEvent<HTMLImageElement>,
    ind: number,
  ) => {
    e.preventDefault();
    e.stopPropagation();
    setImgs((imgs) => imgs.filter((_, i) => i != ind));
    setImgSources((imgs) => imgs.filter((_, i) => i != ind));
  };

  return (
    <div className="standard flex flex-col">
      {imgs.length > 0 ? (
        <div className="my-1">
          {imgs.map((img, ind) => (
            <img
              onContextMenu={(e) => onContextMenu(e, ind)}
              className="h-16 first:ml-1 mr-2 rounded"
              key={ind}
              src={`data:image/jpeg;base64,${img}`}
            />
          ))}
        </div>
      ) : null}
      <div className="flex flex-row items-start my-2">
        <img
          src={isUser ? (stopStatus ? stop : user) : logo}
          alt={isUser ? (stopStatus ? "Stop" : "User") : "Logo"}
          onClick={current && generating ? stopGenerating : () => {}}
          className={`mx-0 ${isUser ? "dark:invert" : ""} ${stopStatus ? "hover:cursor-pointer" : ""}`}
        />
        {isUser ? (
          <div
            ref={inputRef}
            contentEditable={current}
            id={current ? "current" : ""}
            className="ml-2.5 w-full bg-transparent border-none outline-none prose prose-xl dark:prose-invert"
            onKeyDown={(event) => {
              if (event.key == "Enter") {
                event.preventDefault();
                onEnter(inputRef.current!);
              }
            }}
          ></div>
        ) : (
          <div
            ref={inputRef}
            className="ml-2.5 w-full bg-transparent border-none outline-none prose prose-xl dark:prose-invert"
          >
            {text === "" || text === null ? (
              <Loading />
            ) : (
              <Markdown remarkPlugins={[remarkGfm]}>{text}</Markdown>
            )}
          </div>
        )}
      </div>
    </div>
  );
};

const resizeApp = () =>
  WindowPosition.setFrame({
    width: 600,
    height: Math.min(document.documentElement.scrollHeight, 500),
    anchorX: 0,
    offsetFromBaseline: 0,
  });

const Chat = () => {
  const [entries, setEntries] = useState<(string | null)[]>([null]);
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [message, setMessage] = useState("");
  const [imgSources, setImgSources] = useState<string[]>([]);
  const [generating, setGenerating] = useState(false);

  const currImages = useRef(0);
  const abortController = useRef<AbortController | null>(null);

  const addEntry = useCallback(
    (entry: string | null) =>
      setEntries((entries) => [...entries.slice(0, -1), entry, "", null]),
    [],
  );

  const modifyLastEntry = useCallback(
    (delta: string) =>
      setEntries((entries) => [
        ...entries.slice(0, -2),
        entries[entries.length - 2] + delta,
        null,
      ]),
    [],
  );

  const checkNumImages = useCallback((numImages: number) => {
    if (numImages > maxImagesPerTurn) {
      setIsModalOpen(true);
      setMessage(
        `You can only have ${maxImagesPerTurn} images per turn. You already inputted ${numImages} images.`,
      );
      return true;
    }

    if (numImages + currImages.current > maxImages) {
      setIsModalOpen(true);
      setMessage(
        `You can only have ${maxImages} images per conversation. You already inputted ${currImages.current} images and you are adding ${numImages} more.`,
      );
      return true;
    }

    return false;
  }, []);

  const onEnter = useCallback(
    async (div: HTMLDivElement) => {
      if (generating) return;

      abortController.current?.abort();

      const text = div.innerText;
      const noText = text.trim().length == 0;
      const images = await extractImages(div);
      let numImages = images.length;

      if (noText && numImages == 0) return;

      let content: Prompt[] = [];

      if (!noText) content.push({ type: "text", text });

      content = content.concat(
        images.map((image) => {
          return {
            type: "image",
            source: { type: "base64", media_type: "image/png", data: image },
          };
        }),
      );

      if (checkNumImages(numImages)) return;

      if (imgSources.length > 0) {
        if (checkNumImages(numImages + imgSources.length)) return;

        content = content.concat(
          imgSources.map((img) => {
            return {
              type: "image",
              source: { type: "base64", media_type: "image/jpeg", data: img },
            };
          }),
        );
        numImages += imgSources.length;
      }

      addEntry(text);
      currImages.current = Math.min(currImages.current + numImages, maxImages);
      setGenerating(true);
      abortController.current = new AbortController();

      body.messages.push({ role: "user", content });
      input.body = new TextEncoder().encode(JSON.stringify(body));
      const command = new InvokeModelWithResponseStreamCommand(input);
      const response = await client.send(command);
      let newText = "";

      for await (const chunk of response.body!) {
        if (
          abortController.current.signal &&
          abortController.current.signal.aborted
        )
          break;

        const delta = JSON.parse(new TextDecoder().decode(chunk.chunk!.bytes));

        if (delta.type == "content_block_delta") {
          newText += delta.delta.text;
          for (const char of delta.delta.text) modifyLastEntry(char);
        }
      }

      body.messages.push({ role: "assistant", content: newText });
      setImgSources([]);
      setGenerating(false);
    },
    [imgSources, generating],
  );

  useEffect(() => {
    if (entries[0] !== null) resizeApp();
    document.getElementById("current")!.focus();
  }, [entries, generating]);

  useEffect(() => {
    if (imgSources.length > 0) resizeApp();
    document.getElementById("current")!.focus();
  }, [imgSources]);

  const stopGenerating = useCallback(
    () => abortController.current?.abort(),
    [],
  );

  return (
    <>
      <Modal
        isOpen={isModalOpen}
        onClose={() => setIsModalOpen(!isModalOpen)}
        message={message}
      />
      <GeneratingContext.Provider value={{ generating, stopGenerating }}>
        {entries.map((entry, ind) => (
          <ChatEntry
            text={entry}
            isUser={ind % 2 == 0}
            onEnter={onEnter}
            key={ind}
            setImgSources={setImgSources}
          />
        ))}
      </GeneratingContext.Provider>
    </>
  );
};

export default Chat;
