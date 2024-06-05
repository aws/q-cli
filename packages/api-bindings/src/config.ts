import {
  sendGetConfigPropertyRequest,
  sendUpdateConfigPropertyRequest,
} from "./requests.js";

export async function get(key: string) {
  const response = await sendGetConfigPropertyRequest({ key });
  return response.value;
}

export function set(key: string, value: string) {
  return sendUpdateConfigPropertyRequest({ key, value });
}

export function remove(key: string) {
  return sendUpdateConfigPropertyRequest({ key, value: undefined });
}
