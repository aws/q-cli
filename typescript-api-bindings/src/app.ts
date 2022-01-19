import { sendUpdateApplicationPropertiesRequest, sendDebuggerUpdateRequest } from './requests';
import { Action } from './fig.pb';

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
}


/**
 * @param {string}  message - the message that will appear in the debugger UI.
 * @param {color} [color] - the hex color to associate with the debugger current state

 * @returns {Promise<void>} 
 */
export function reportError({
   message,
   color,
}: {
  message: string[];
  color?: string;
}) {
  return sendDebuggerUpdateRequest({ color: color, layout: message})
}

/**
 * Reset the debugger UI. Any previous value written from JS will be removed.
 * @returns {Promise<void>} 
 */
export function resetDebugger() {
 return sendDebuggerUpdateRequest({ layout: []})
}