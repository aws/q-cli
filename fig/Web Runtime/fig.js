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
          if (fig.debug) { console.log(`Inserting command "${cmd}" as user`) }
          window.webkit.messageHandlers.insertHandler.postMessage(cmd);
      },
      run : function(cmd) {
          if (fig.debug) { console.log(`Running command "${cmd}" as user`) }
          window.webkit.messageHandlers.executeHandler.postMessage(cmd);
      },
      execute : function(cmd, handler) {
          let handlerId = random_identifier(5)
          let type = "execute"
          this[handlerId] = handler
          let env = JSON.stringify(fig.env)
          if (!env) {
              console.log("Error: Attempting to call `fig.execute` before `fig.env` has loaded. To fix this error, move this code inside of either the `fig.stdin` or the `fig.init` callbacks.\n\nIf you don't need to run the shell script from the users working directory, use fig.executeInHomeDirectory\n")
              console.log(`Could not execute '${cmd}'...`)

              return
          }
          window.webkit.messageHandlers.callbackHandler.postMessage({type, cmd, handlerId, env});
          if (fig.debug) { console.log(`Added callback handler "${handlerId}" for command "${cmd}"`) }
      },
      executeInHomeDirectory : function(cmd, handler) {
          let handlerId = random_identifier(5)
          let type = "execute"
          this[handlerId] = handler
          window.webkit.messageHandlers.globalExecuteHandler.postMessage({type, cmd, handlerId});
          if (fig.debug) { console.log(`Added callback handler "${handlerId}" for command "${cmd}"`) }

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
//          console.log("fig.stdin must be overwritten in order to recieve standard input.")
      },
      stdout : function(out) {
          window.webkit.messageHandlers.stdoutHandler.postMessage({out});
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
          if (fig.debug) { console.log(handlerId) }
          this[handlerId](response, error)
          let tokens = handlerId.split(':')
          if (tokens.length == 1) {
              delete this[handlerId]
          } else if (!value) {
              delete this[handlerId]
          } else {
              if (fig.debug) { console.log(`Continue streaming to callback '${handlerId}'`) }
          }
      },
      callbackASCII : function(handlerId, value, error) {
          this[handlerId](value, error)
          let tokens = handlerId.split(':')
          if (tokens.length == 1) {
              delete this[handlerId]
              if (fig.debug) { console.log(`End of stream to callback '${handlerId}'`) }
          } else {
              if (fig.debug) { console.log(`Continue streaming to callback '${handlerId}'`) }
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
      init : function(input, options) {
          console.log("fig.init must be overwritten. The behavior of other fig functions is undefined if called prior to the fig.init entrypoint.")
      },
      callinit : function() {
          let urlParams = new URLSearchParams(window.location.search);
          let inputParam = urlParams.get('input');
          var opts = null
          try {
              opts = inputParam.split('%20')
          } catch(e) {}
          
          // if fig.env is not set, set pwd & home to HOME directory
          if (!fig.env) {
              fig.executeInHomeDirectory("pwd", (out) => {
                 fig.env = {}
                 //fig.env.PWD = out.trim()
                 fig.env.HOME = out.trim()
                 console.log("hello", fig.env)
              })
              
          }
          
          let stdin = fig['_stdin']
          let options = fig.options //|| opts
//          fig.pty.init()
          fig.init(stdin, options)
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
          if (fig.debug) { console.log("Repositioning") }
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
              console.log("Error: Attempting to call `fig.stream` before `fig.env` has loaded. To fix this error, move this code inside of either the `fig.stdin` or the `fig.init` callbacks.\n\n")
              console.log(`Could not stream '${cmd}'...`)

              return
          }
          window.webkit.messageHandlers.streamHandler.postMessage({type, cmd, handlerId, env});
          if (fig.debug) { console.log(`Added callback handler "${handlerId}" for command "${cmd}"`) }
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
      },
      appinfo() {
          var name = null
          let figapp = document.head.querySelector('meta[fig\\:app]');
          if (figapp){
              name = figapp.getAttribute('fig:app');
          }
          
          var icon = null
          let figicon = document.head.querySelector('meta[fig\\:icon]');
          if (figicon){
              icon = figicon.getAttribute('fig:icon');
          }
          
          var color = null
          let figcolor = document.head.querySelector('meta[fig\\:color]');
          if (figcolor){
              color = figcolor.getAttribute('fig:color');
          }
          
          var position = null
          let figposition = document.head.querySelector('meta[initial-position]');
          if (figposition){
              position = figposition.getAttribute('initial-position');
          }
          
          return {name, icon, color, position};
      },
      detatch() {
          window.webkit.messageHandlers.detachHandler.postMessage("")
      },
      close() {
          fig.reposition("7")
      },
      notify(title, text) {
          window.webkit.messageHandlers.notificationHandler.postMessage({title, text});
      },
      pty : {
          init() {
              let env = JSON.stringify(fig.env)
              window.webkit.messageHandlers.ptyHandler.postMessage({env, type: 'init'});
          },
          execute(cmd, handler) {
              let handlerId = `${random_identifier(5)}:pty`
              fig[handlerId] = handler
              window.webkit.messageHandlers.ptyHandler.postMessage({handlerId, cmd, type: 'execute'});

          },
          stream(cmd, handler) {
              let handlerId = `${random_identifier(5)}:pty`
              fig[handlerId] = handler
              window.webkit.messageHandlers.ptyHandler.postMessage({handlerId, cmd, type: 'stream'});
          },
          write(cmd) {
              window.webkit.messageHandlers.ptyHandler.postMessage({cmd, type: 'write'});
          },

          exit() {
//              Object.keys(fig).filter(key => key.endsWith(":pty"))
              console.log("Don't run `fig.pty.exit()` unless you know what you're doing.")
              window.webkit.messageHandlers.ptyHandler.postMessage({type: 'exit'});
          }
      },
      private(func) {
          window.webkit.messageHandlers.privateHandler.postMessage(func);
      },
      cursor : {
          forward() {
              fig.insert("fig::forward")
          },
          backward() {
              fig.insert("fig::backward")
          },
          backspace() {
              fig.insert("fig::backspace")
          }
      }
  }
    
    let watchedProperties = [ "icon", "title", "color", "maxheight"]
    watchedProperties.forEach(prop => {
          Object.defineProperty(fig, prop, {
              get : function () {
                  return this[`_${prop}`];
              },
              set : function (value) {
                  window.webkit.messageHandlers.propertyUpdateHandler.postMessage({prop, value});
                  if (fig.debug) { console.log("SET:", prop, value) }
                  this[`_${prop}`] = value;
              }
          });
    })

  window.fig = fig;
    
//    window.opener.postMessage = function(message, targetOrigin) {
//        console.log(message, targetOrigin)
//    }
    
    window.close = function() {
        fig.close()
    }
}
// console.log(Object.keys(fig));
//
//})(window);

setup(window)
fig.log("Loaded fig.js...")
fig.debug = false


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
