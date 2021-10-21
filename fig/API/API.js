// API.js is for internal use.
// It should only be used in contexts where we cannot use @withfig/api-bindings


let id = 0;
const sendMessage = (json) => {
  json.id = id++;
  window.webkit.messageHandlers[fig.constants.jsonMessageHandler].postMessage(JSON.stringify(json))
}
