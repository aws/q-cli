import { sendUpdateApplicationPropertiesRequest } from './requests';
import { Action } from './fig';

export function registerActions(actions: Array<Action>) {
  return sendUpdateApplicationPropertiesRequest({ actionList: { actions } });
}

export function setKeystrokeIntercept({
  interceptBoundKeystrokes,
  interceptGlobalKeystrokes,
}: {
  interceptBoundKeystrokes?: boolean;
  interceptGlobalKeystrokes?: boolean;
}) {
  return sendUpdateApplicationPropertiesRequest({
    interceptBoundKeystrokes,
    interceptGlobalKeystrokes,
  });
