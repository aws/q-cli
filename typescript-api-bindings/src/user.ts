import {
  sendUserLogoutRequest,
} from './requests';

export async function logout() {
  return sendUserLogoutRequest();
}
