import { useEffect } from "react";
import {
  BedrockRuntimeClient,
  BedrockRuntimeClientConfigType,
} from "@aws-sdk/client-bedrock-runtime";
import { Event } from "@amzn/fig-io-api-bindings";

const config: BedrockRuntimeClientConfigType = {
  region: "us-west-2",
  credentials: {
    accessKeyId: import.meta.env.VITE_AWS_ACCESS_KEY_ID,
    secretAccessKey: import.meta.env.VITE_AWS_SECRET_ACCESS_KEY,
    sessionToken: import.meta.env.VITE_AWS_SESSION_TOKEN,
  },
};

export const client = new BedrockRuntimeClient(config);
export const maxImagesPerTurn = 5;
export const maxImages = 20;

type TextPrompt = {
  type: "text";
  text: string;
};

type ImagePrompt = {
  type: "image";
  source: {
    type: "base64";
    media_type: "image/jpeg" | "image/png" | "image/gif" | "image.webp";
    data: string;
  };
};

export type Prompt = TextPrompt | ImagePrompt;

type Turn = {
  role: "user" | "assistant";
  content: Prompt[] | string;
};

export type Body = {
  anthropic_version: string;
  max_tokens: number;
  messages: Turn[];
};

type Unwrap<T> = T extends Promise<infer U> ? U : T;

type Subscription = Unwrap<
  NonNullable<ReturnType<(typeof Event)["subscribe"]>>
>;

export const useFigSubscriptionEffect = (
  getSubscription: () => Promise<Subscription> | undefined,
  deps: React.DependencyList,
) => {
  useEffect(() => {
    let unsubscribe: () => void;
    let isStale = false;
    // if the component is unmounted before the subscription is awaited we
    // unsubscribe from the event
    getSubscription()?.then((result) => {
      unsubscribe = result.unsubscribe;
      if (isStale) unsubscribe();
    });
    return () => {
      if (unsubscribe) unsubscribe();
      isStale = true;
    };
  }, deps);
};

export async function readBlobAsDataURL(blob: Blob): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onloadend = () => resolve(reader.result as string);
    reader.onerror = reject;
    reader.readAsDataURL(blob);
  });
}
