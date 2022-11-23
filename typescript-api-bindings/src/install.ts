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

export type Component = "dotfiles" | "ibus" | "inputMethod" | "accessibility";

function componentToProto(component: Component) {
  switch (component) {
    case "dotfiles":
      return InstallComponent.INSTALL_COMPONENT_DOTFILES;
    case "ibus":
      return InstallComponent.INSTALL_COMPONENT_IBUS;
    case "accessibility":
      return InstallComponent.INSTALL_COMPONENT_ACCESSIBILITY;
    case "inputMethod":
      return InstallComponent.INSTALL_COMPONENT_INPUT_METHOD;
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
    action: InstallAction.INSTALL_ACTION_INSTALL,
    component: componentToProto(component)
  });
  handleBasicResponse(response);
}

export async function uninstall(component: Component) {
  const response = await sendInstallRequest({
    action: InstallAction.INSTALL_ACTION_UNINSTALL,
    component: componentToProto(component)
  });
  handleBasicResponse(response);
}

export async function isInstalled(component: Component) {
  const response = await sendInstallRequest({
    action: InstallAction.INSTALL_ACTION_STATUS,
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
