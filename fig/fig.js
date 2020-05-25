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
      run : function(cmd) {
          window.webkit.messageHandlers.executeHandler.postMessage(cmd);
      },
      execute : function(cmd, handler) {
          let handlerId = random_identifier(5)
          let type = "execute"
          console.log(JSON.stringify({type, cmd,handlerId}))
          this[handlerId] = handler
        console.log("test")
            console.log(JSON.stringify(window.webkit))
          window.webkit.messageHandlers.callbackHandler.postMessage({type, cmd, handlerId});
          console.log(`Added callback handler "${handlerId}" for command "${cmd}"`)
      },
      stdin : function(input) {
          console.log("fig.stdin must be overwritten in order to recieve standard input.")
      },
      
      stdout : function(out) {
          window.webkit.messageHandlers.stdoutHandler.postMessage(out);
      },
      
      callback : function(handlerId, value, error) {
          this[handlerId](value, error)
          delete this[handlerId]
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
console.log(JSON.stringify(window))
