import { sendUpdateApplicationPropertiesRequest } from './requests';
import { Action } from './fig';

export const registerActions = (actions: Array<Action>) =>
  sendUpdateApplicationPropertiesRequest({ actionList: { actions } });

export const setKeystrokeIntercept = ({
  interceptBoundKeystrokes,
  interceptGlobalKeystrokes,
}: {
  interceptBoundKeystrokes?: boolean;
  interceptGlobalKeystrokes?: boolean;
}) =>
  sendUpdateApplicationPropertiesRequest({
    interceptBoundKeystrokes,
    interceptGlobalKeystrokes,
  });
