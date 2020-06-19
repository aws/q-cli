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
          this[handlerId] = handler
          let env = JSON.stringify(fig.env)
          if (!env) {
              console.log("Error: Attempting to call `fig.execute` before `fig.env` has loaded. To fix this error, move this code inside of either the `fig.stdin` or the `fig.init` callbacks.\n\nIf you don't need to run the shell script from the users working directory, use fig.executeInGlobalScope\n")
              console.log(`Could not execute '${cmd}'...`)

              return
          }
          window.webkit.messageHandlers.callbackHandler.postMessage({type, cmd, handlerId, env});
          console.log(`Added callback handler "${handlerId}" for command "${cmd}"`)
      },
//      execute : function(cmd, handler) {
//          var out = ""
//          fig.stream(cmd, (data, error) => {
//             if (data) {
//               out += data + "\n"
//             } else {
//                handler(out)
//             }
//          })
//      },
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
      appwrite : function(path, data, handler) {
          let handlerId = random_identifier(5)
          this[handlerId] = handler
          let env = JSON.stringify(fig.env)
          window.webkit.messageHandlers.appwriteHandler.postMessage({path, data, handlerId, env});
      },
      appread : function(path, handler) {
          let handlerId = random_identifier(5)
          this[handlerId] = handler
          let env = JSON.stringify(fig.env)
          window.webkit.messageHandlers.appreadHandler.postMessage({path, handlerId, env});
      },
      focus : function () {
          window.webkit.messageHandlers.focusHandler.postMessage("");
      },
      blur : function () {
          window.webkit.messageHandlers.blurHandler.postMessage("");
      },
      callback : function(handlerId, value, error) {
          let response = value ? b64DecodeUnicode(value) : null
          this[handlerId](response, error)
          let tokens = handlerId.split(':')
          if (tokens.length == 1) {
              delete this[handlerId]
          } else if (!value) {
              delete this[handlerId]
          } else {
              console.log(`Continue streaming to callback '${handlerId}'`)
          }
      },
      callbackASCII : function(handlerId, value, error) {
          this[handlerId](value, error)
          let tokens = handlerId.split(':')
          if (tokens.length == 1) {
              delete this[handlerId]
              console.log(`End of stream to callback '${handlerId}'`)
          } else {
              console.log(`Continue streaming to callback '${handlerId}'`)
          }
      },
      stdinb64 : function(data) {
          fig['_stdin'] = b64DecodeUnicode(data)
//          fig.init(atob(data))
//          fig.stdin(atob(data))
      },
      log : function(msg) {
          console.log(JSON.stringify(msg))
      },
      init : function(input) {
          console.log("fig.init must be overwritten. The behavior of other fig functions is undefined if called prior to the fig.init entrypoint.")
      },
      callinit : function() {
          let stdin = fig['_stdin']
          fig.init(stdin)
          fig.stdin(stdin)
      },
      ttyinb64 : function(input, session) {
//          if ()
//          fig.log(JSON.stringify(fig.env["TERMSESSION"]))
//          if (fig.env["TERMSESSION"] == session) {
//          }
          if (fig.ttyin) {
              fig.ttyin(b64DecodeUnicode(input))
          }
      },
      ttyoutb64 : function(input, session) {
          if (fig.ttyout) {
              fig.ttyout(b64DecodeUnicode(input))
          }
//          if (fig.env["TERMSESSION"] == session) {
//          }
      },
      reposition : function(position) {
          console.log("Repositioning")
          window.webkit.messageHandlers.positionHandler.postMessage({position});

      },
      open : function(url) {
          window.webkit.messageHandlers.openHandler.postMessage({url});
      },
      stream : function(cmd, handler) {
          let handlerId = `${random_identifier(5)}:stream`
          let type = "stream"
          this[handlerId] = handler
          let env = JSON.stringify(fig.env)
          
          if (!env) {
              console.log("Error: Attempting to call `fig.stream` before `fig.env` has loaded. To fix this error, move this code inside of either the `fig.stdin` or the `fig.init` callbacks.\n\nIf you don't need to run the shell script from the users working directory, use fig.executeInGlobalScope\n")
              console.log(`Could not stream '${cmd}'...`)

              return
          }
          window.webkit.messageHandlers.streamHandler.postMessage({type, cmd, handlerId, env});
          console.log(`Added callback handler "${handlerId}" for command "${cmd}"`)
      },
      onboarding : function(action, handler) {
          let handlerId = random_identifier(5)
          this[handlerId] = handler
          window.webkit.messageHandlers.onboardingHandler.postMessage({action, handlerId});
      },
      defaults : {
          set: function(key, value) {
            window.webkit.messageHandlers.defaultsHandler.postMessage({key, value});
          },
          get: function(key, handler) {
            let handlerId = random_identifier(5)
            fig[handlerId] = handler
            window.webkit.messageHandlers.defaultsHandler.postMessage({key, handlerId});
          }
      },
      normalizeFilePath(path, handler) {
          let handlerId = random_identifier(5)
          fig[handlerId] = handler
          window.webkit.messageHandlers.filepathHandler.postMessage({path, handlerId});
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
fig.log("Loaded fig.js...")

function b64EncodeUnicode(str) {
    // first we use encodeURIComponent to get percent-encoded UTF-8,
    // then we convert the percent encodings into raw bytes which
    // can be fed into btoa.
    return btoa(encodeURIComponent(str).replace(/%([0-9A-F]{2})/g,
        function toSolidBytes(match, p1) {
            return String.fromCharCode('0x' + p1);
    }));
}

function b64DecodeUnicode(str) {
    // Going backwards: from bytestream, to percent-encoding, to original string.
    return decodeURIComponent(atob(str).split('').map(function(c) {
        return '%' + ('00' + c.charCodeAt(0).toString(16)).slice(-2);
    }).join(''));
}
