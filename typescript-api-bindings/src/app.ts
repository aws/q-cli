import { sendUpdateApplicationPropertiesRequest } from './requests';
import { Action } from './fig';

const registerActions = (actions: Array<Action>) =>
  sendUpdateApplicationPropertiesRequest({ actionList: { actions } });

const setKeystrokeIntercept = ({
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

const App = { registerActions, setKeystrokeIntercept };

export default App;
