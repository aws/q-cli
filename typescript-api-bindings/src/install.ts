import {
  InstallAction,
  InstallComponent,
  InstallResponse,
  // eslint-disable-next-line camelcase
  InstallResponse_InstallationStatus,
  // eslint-disable-next-line camelcase
  Result_ResultEnum
} from "./fig.pb";

import { sendInstallRequest } from "./requests";

export type Component = "dotfiles" | "ibus";

function componentToProto(component: Component) {
  switch (component) {
    case "dotfiles":
      return InstallComponent.DOTFILES;
    case "ibus":
      return InstallComponent.IBUS;
    default:
      throw Error("Invalid component");
  }
}

function handleBasicResponse(response: InstallResponse) {
  switch (response.response?.$case) {
    case "result":
      // eslint-disable-next-line camelcase
      if (response.response.result.result === Result_ResultEnum.RESULT_OK) {
        return;
      }
      // eslint-disable-next-line camelcase
      if (response.response.result.result === Result_ResultEnum.RESULT_ERROR) {
        throw Error(response.response.result.error);
      } else {
        throw Error(`Unexpected result: ${response.response.result.result}`);
      }
    default:
      throw Error(`Unexpected result: ${response.response?.$case}`);
  }
}

export async function install(component: Component) {
  const response = await sendInstallRequest({
    action: InstallAction.INSTALL_ACTION,
    component: componentToProto(component)
  });
  handleBasicResponse(response);
}

export async function uninstall(component: Component) {
  const response = await sendInstallRequest({
    action: InstallAction.UNINSTALL_ACTION,
    component: componentToProto(component)
  });
  handleBasicResponse(response);
}

export async function isInstalled(component: Component) {
  const response = await sendInstallRequest({
    action: InstallAction.STATUS_ACTION,
    component: componentToProto(component)
  });
  switch (response.response?.$case) {
    case "installationStatus":
      if (
        response.response.installationStatus ===
        // eslint-disable-next-line camelcase
        InstallResponse_InstallationStatus.INSTALL_INSTALLED
      ) {
        return true;
      }
      if (
        response.response.installationStatus ===
        // eslint-disable-next-line camelcase
        InstallResponse_InstallationStatus.INSTALL_NOT_INSTALLED
      ) {
        return false;
      }
      throw Error(`Unexpected result: ${response.response.installationStatus}`);

    default:
      throw Error(`Unexpected result: ${response.response?.$case}`);
  }
}
