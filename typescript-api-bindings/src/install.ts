import {
  InstallAction,
  InstallComponent,
  InstallResponse,
  // eslint-disable-next-line camelcase
  InstallResponse_InstallationStatus,
  // eslint-disable-next-line camelcase
  Result_Result
} from "./fig.pb";

import { sendInstallRequest } from "./requests";

export type Component =
  | "dotfiles"
  | "ibus"
  | "inputMethod"
  | "accessibility"
  | "ssh";

function componentToProto(component: Component) {
  switch (component) {
    case "dotfiles":
      return InstallComponent.DOTFILES;
    case "ibus":
      return InstallComponent.IBUS;
    case "accessibility":
      return InstallComponent.ACCESSIBILITY;
    case "inputMethod":
      return InstallComponent.INPUT_METHOD;
    case "ssh":
      return InstallComponent.SSH;
    default:
      throw Error("Invalid component");
  }
}

function handleBasicResponse(response: InstallResponse) {
  switch (response.response?.$case) {
    case "result":
      // eslint-disable-next-line camelcase
      if (response.response.result.result === Result_Result.RESULT_OK) {
        return;
      }
      // eslint-disable-next-line camelcase
      if (response.response.result.result === Result_Result.RESULT_ERROR) {
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
    action: InstallAction.INSTALL,
    component: componentToProto(component)
  });
  handleBasicResponse(response);
}

export async function uninstall(component: Component) {
  const response = await sendInstallRequest({
    action: InstallAction.UNINSTALL,
    component: componentToProto(component)
  });
  handleBasicResponse(response);
}

export async function isInstalled(component: Component) {
  const response = await sendInstallRequest({
    action: InstallAction.STATUS,
    component: componentToProto(component)
  });
  switch (response.response?.$case) {
    case "installationStatus":
      if (
        response.response.installationStatus ===
        // eslint-disable-next-line camelcase
        InstallResponse_InstallationStatus.INSTALLATION_STATUS_INSTALLED
      ) {
        return true;
      }
      if (
        response.response.installationStatus ===
        // eslint-disable-next-line camelcase
        InstallResponse_InstallationStatus.INSTALLATION_STATUS_NOT_INSTALLED
      ) {
        return false;
      }
      throw Error(`Unexpected result: ${response.response.installationStatus}`);

    default:
      throw Error(`Unexpected result: ${response.response?.$case}`);
  }
}
