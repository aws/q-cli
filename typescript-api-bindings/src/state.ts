import {
  sendGetLocalStateRequest,
  sendUpdateLocalStateRequest
} from './requests';

export async function get(key: string) {
  const response = await sendGetLocalStateRequest({
    key
  });

  if (response.jsonBlob) {
    return JSON.parse(response.jsonBlob);
  } 
    return null;
  
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export async function set(key: string, value: any): Promise<void> {
  return sendUpdateLocalStateRequest({
    key,
    value: JSON.stringify(value)
  });
}
