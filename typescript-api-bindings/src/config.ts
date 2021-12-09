import {
  sendGetConfigPropertyRequest,
  sendUpdateConfigPropertyRequest,
} from './requests';

export const get = async (key: string) => {
  let response = await sendGetConfigPropertyRequest({ key: key });
  return response.value;
};

export const set = (key: string, value: string) =>
  sendUpdateConfigPropertyRequest({ key: key, value: value });

export const remove = (key: string) =>
  sendUpdateConfigPropertyRequest({ key: key, value: undefined });
