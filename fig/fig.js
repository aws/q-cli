//(function(window) {

let setup = function(window) {
 
 function random_identifier(len) {
   let alphabet = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
   var id = ""
   for ( var i = 0 ; i < len; i++){
     let idx = Math.floor(Math.random() * alphabet.length)
     id += alphabet[idx]
   }
   return id
 }

  var fig = {
      insert : function(cmd) {
          console.log(`Inserting command "${cmd}" as user`)
          window.webkit.messageHandlers.insertHandler.postMessage(cmd);
      },
      run : function(cmd) {
          console.log(`Running command "${cmd}" as user`)
          window.webkit.messageHandlers.executeHandler.postMessage(cmd);
      },
      execute : function(cmd, handler) {
          let handlerId = random_identifier(5)
          let type = "execute"
          console.log(JSON.stringify({type, cmd,handlerId}))
          this[handlerId] = handler
          let env = JSON.stringify(fig.env)
          window.webkit.messageHandlers.callbackHandler.postMessage({type, cmd, handlerId, env});
          console.log(`Added callback handler "${handlerId}" for command "${cmd}"`)
      },
      stdin : function(input) {
          console.log("fig.stdin must be overwritten in order to recieve standard input.")
      },
      
      stdout : function(out) {
          window.webkit.messageHandlers.stdoutHandler.postMessage(out);
      },
      fwrite : function(path, data, handler) {
          let handlerId = random_identifier(5)
          this[handlerId] = handler
          let env = JSON.stringify(fig.env)
          window.webkit.messageHandlers.fwriteHandler.postMessage({path, data, handlerId, env});
      },
      fread : function(path, handler) {
          let handlerId = random_identifier(5)
          this[handlerId] = handler
          let env = JSON.stringify(fig.env)
          window.webkit.messageHandlers.freadHandler.postMessage({path, handlerId, env});
      },
      focus : function () {
          window.webkit.messageHandlers.focusHandler.postMessage("");
      },
      blur : function () {
          window.webkit.messageHandlers.blurHandler.postMessage("");
      },
      callback : function(handlerId, value, error) {
          this[handlerId](atob(value), error)
          delete this[handlerId]
      },
      stdinb64 : function(data) {
          fig.stdin(atob(data))
      },
      log : function(msg) {
          console.log(JSON.stringify(msg))
      }
  }

  //fig.init((stdin, options) => {

  //})
  window.fig = fig;
}
// console.log(Object.keys(fig));
//
//})(window);

setup(window)
//console.log(JSON.stringify(window))
