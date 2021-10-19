import { sendUpdateApplicationPropertiesRequest } from "./requests";
import { Action } from "./fig";
const registerActions = (actions: Array<Action>) => 
    sendUpdateApplicationPropertiesRequest({ actions: actions})

const App = { registerActions };

export default App;