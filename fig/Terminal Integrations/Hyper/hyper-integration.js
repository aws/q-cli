const { exec } = require("child_process");
console.log(exec)
exports.middleware = ({ getState, dispatch }) => next => async (action) => {
  switch (action.type) {
    case 'SESSION_SET_ACTIVE':
    case 'SESSION_ADD':
      console.log(action.type, action.uid)
      exec(`~/.fig/bin/fig keyboard-focus-changed hyper ${action.uid.split('-')[0]}`);
    default:
      break;
  }
  next(action);
}
