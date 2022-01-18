import { sendOpenInExternalApplicationRequest } from './requests';

export function open(url: string) {
  return sendOpenInExternalApplicationRequest({ url });
}
