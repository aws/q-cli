import {
  sendGetConfigPropertyRequest,
  sendUpdateConfigPropertyRequest,
} from './requests';

export async function get(key: string) {
  let response = await sendGetConfigPropertyRequest({ key: key });
  return response.value;
}

export function set(key: string, value: string) {
  return sendUpdateConfigPropertyRequest({ key: key, value: value });
}

export function remove(key: string) {
  return sendUpdateConfigPropertyRequest({ key: key, value: undefined });
}
