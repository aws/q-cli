/* Autogenerated by generate-requests.ts on 5/31/2022
 * Do not edit directly! Instead run 'npm run generate-requests'
 */

import {
  AppendToFileRequest,
  ApplicationUpdateStatusRequest,
  ApplicationUpdateStatusResponse,
  ContentsOfDirectoryRequest,
  ContentsOfDirectoryResponse,
  CreateDirectoryRequest,
  DebuggerUpdateRequest,
  DestinationOfSymbolicLinkRequest,
  DestinationOfSymbolicLinkResponse,
  GetConfigPropertyRequest,
  GetConfigPropertyResponse,
  GetDefaultsPropertyRequest,
  GetDefaultsPropertyResponse,
  GetLocalStateRequest,
  GetLocalStateResponse,
  GetSettingsPropertyRequest,
  GetSettingsPropertyResponse,
  InsertTextRequest,
  MacosInputMethodRequest,
  MacosInputMethodResponse,
  OnboardingRequest,
  OpenInExternalApplicationRequest,
  PositionWindowRequest,
  PositionWindowResponse,
  PseudoterminalExecuteRequest,
  PseudoterminalExecuteResponse,
  PseudoterminalRestartRequest,
  PseudoterminalWriteRequest,
  ReadFileRequest,
  ReadFileResponse,
  RunProcessRequest,
  RunProcessResponse,
  TelemetryAliasRequest,
  TelemetryIdentifyRequest,
  TelemetryTrackRequest,
  TerminalSessionInfoRequest,
  TerminalSessionInfoResponse,
  UpdateApplicationPropertiesRequest,
  UpdateConfigPropertyRequest,
  UpdateDefaultsPropertyRequest,
  UpdateLocalStateRequest,
  UpdateSettingsPropertyRequest,
  WindowFocusRequest,
  WriteFileRequest
} from "./fig.pb";
import { sendMessage } from "./core"

export async function sendPositionWindowRequest(
  request: PositionWindowRequest
): Promise<PositionWindowResponse> {
  return new Promise((resolve, reject) => {
    sendMessage(
      { $case: "positionWindowRequest", positionWindowRequest: request },
      (response) => {
        switch (response?.$case) {
          case "positionWindowResponse":
            resolve(response.positionWindowResponse);
            break;
          case "error":
            reject(Error(response.error));
            break;
          default:
            reject(
              Error(
                "Invalid response '" + response?.$case + "' for 'PositionWindowRequest'"
              )
            );
        }
      }
    );
  });
}

export async function sendPseudoterminalExecuteRequest(
  request: PseudoterminalExecuteRequest
): Promise<PseudoterminalExecuteResponse> {
  return new Promise((resolve, reject) => {
    sendMessage(
      { $case: "pseudoterminalExecuteRequest", pseudoterminalExecuteRequest: request },
      (response) => {
        switch (response?.$case) {
          case "pseudoterminalExecuteResponse":
            resolve(response.pseudoterminalExecuteResponse);
            break;
          case "error":
            reject(Error(response.error));
            break;
          default:
            reject(
              Error(
                "Invalid response '" + response?.$case + "' for 'PseudoterminalExecuteRequest'"
              )
            );
        }
      }
    );
  });
}

export async function sendReadFileRequest(
  request: ReadFileRequest
): Promise<ReadFileResponse> {
  return new Promise((resolve, reject) => {
    sendMessage(
      { $case: "readFileRequest", readFileRequest: request },
      (response) => {
        switch (response?.$case) {
          case "readFileResponse":
            resolve(response.readFileResponse);
            break;
          case "error":
            reject(Error(response.error));
            break;
          default:
            reject(
              Error(
                "Invalid response '" + response?.$case + "' for 'ReadFileRequest'"
              )
            );
        }
      }
    );
  });
}

export async function sendContentsOfDirectoryRequest(
  request: ContentsOfDirectoryRequest
): Promise<ContentsOfDirectoryResponse> {
  return new Promise((resolve, reject) => {
    sendMessage(
      { $case: "contentsOfDirectoryRequest", contentsOfDirectoryRequest: request },
      (response) => {
        switch (response?.$case) {
          case "contentsOfDirectoryResponse":
            resolve(response.contentsOfDirectoryResponse);
            break;
          case "error":
            reject(Error(response.error));
            break;
          default:
            reject(
              Error(
                "Invalid response '" + response?.$case + "' for 'ContentsOfDirectoryRequest'"
              )
            );
        }
      }
    );
  });
}

export async function sendGetSettingsPropertyRequest(
  request: GetSettingsPropertyRequest
): Promise<GetSettingsPropertyResponse> {
  return new Promise((resolve, reject) => {
    sendMessage(
      { $case: "getSettingsPropertyRequest", getSettingsPropertyRequest: request },
      (response) => {
        switch (response?.$case) {
          case "getSettingsPropertyResponse":
            resolve(response.getSettingsPropertyResponse);
            break;
          case "error":
            reject(Error(response.error));
            break;
          default:
            reject(
              Error(
                "Invalid response '" + response?.$case + "' for 'GetSettingsPropertyRequest'"
              )
            );
        }
      }
    );
  });
}

export async function sendDestinationOfSymbolicLinkRequest(
  request: DestinationOfSymbolicLinkRequest
): Promise<DestinationOfSymbolicLinkResponse> {
  return new Promise((resolve, reject) => {
    sendMessage(
      { $case: "destinationOfSymbolicLinkRequest", destinationOfSymbolicLinkRequest: request },
      (response) => {
        switch (response?.$case) {
          case "destinationOfSymbolicLinkResponse":
            resolve(response.destinationOfSymbolicLinkResponse);
            break;
          case "error":
            reject(Error(response.error));
            break;
          default:
            reject(
              Error(
                "Invalid response '" + response?.$case + "' for 'DestinationOfSymbolicLinkRequest'"
              )
            );
        }
      }
    );
  });
}

export async function sendGetDefaultsPropertyRequest(
  request: GetDefaultsPropertyRequest
): Promise<GetDefaultsPropertyResponse> {
  return new Promise((resolve, reject) => {
    sendMessage(
      { $case: "getDefaultsPropertyRequest", getDefaultsPropertyRequest: request },
      (response) => {
        switch (response?.$case) {
          case "getDefaultsPropertyResponse":
            resolve(response.getDefaultsPropertyResponse);
            break;
          case "error":
            reject(Error(response.error));
            break;
          default:
            reject(
              Error(
                "Invalid response '" + response?.$case + "' for 'GetDefaultsPropertyRequest'"
              )
            );
        }
      }
    );
  });
}

export async function sendGetConfigPropertyRequest(
  request: GetConfigPropertyRequest
): Promise<GetConfigPropertyResponse> {
  return new Promise((resolve, reject) => {
    sendMessage(
      { $case: "getConfigPropertyRequest", getConfigPropertyRequest: request },
      (response) => {
        switch (response?.$case) {
          case "getConfigPropertyResponse":
            resolve(response.getConfigPropertyResponse);
            break;
          case "error":
            reject(Error(response.error));
            break;
          default:
            reject(
              Error(
                "Invalid response '" + response?.$case + "' for 'GetConfigPropertyRequest'"
              )
            );
        }
      }
    );
  });
}

export async function sendTerminalSessionInfoRequest(
  request: TerminalSessionInfoRequest
): Promise<TerminalSessionInfoResponse> {
  return new Promise((resolve, reject) => {
    sendMessage(
      { $case: "terminalSessionInfoRequest", terminalSessionInfoRequest: request },
      (response) => {
        switch (response?.$case) {
          case "terminalSessionInfoResponse":
            resolve(response.terminalSessionInfoResponse);
            break;
          case "error":
            reject(Error(response.error));
            break;
          default:
            reject(
              Error(
                "Invalid response '" + response?.$case + "' for 'TerminalSessionInfoRequest'"
              )
            );
        }
      }
    );
  });
}

export async function sendApplicationUpdateStatusRequest(
  request: ApplicationUpdateStatusRequest
): Promise<ApplicationUpdateStatusResponse> {
  return new Promise((resolve, reject) => {
    sendMessage(
      { $case: "applicationUpdateStatusRequest", applicationUpdateStatusRequest: request },
      (response) => {
        switch (response?.$case) {
          case "applicationUpdateStatusResponse":
            resolve(response.applicationUpdateStatusResponse);
            break;
          case "error":
            reject(Error(response.error));
            break;
          default:
            reject(
              Error(
                "Invalid response '" + response?.$case + "' for 'ApplicationUpdateStatusRequest'"
              )
            );
        }
      }
    );
  });
}

export async function sendGetLocalStateRequest(
  request: GetLocalStateRequest
): Promise<GetLocalStateResponse> {
  return new Promise((resolve, reject) => {
    sendMessage(
      { $case: "getLocalStateRequest", getLocalStateRequest: request },
      (response) => {
        switch (response?.$case) {
          case "getLocalStateResponse":
            resolve(response.getLocalStateResponse);
            break;
          case "error":
            reject(Error(response.error));
            break;
          default:
            reject(
              Error(
                "Invalid response '" + response?.$case + "' for 'GetLocalStateRequest'"
              )
            );
        }
      }
    );
  });
}

export async function sendRunProcessRequest(
  request: RunProcessRequest
): Promise<RunProcessResponse> {
  return new Promise((resolve, reject) => {
    sendMessage(
      { $case: "runProcessRequest", runProcessRequest: request },
      (response) => {
        switch (response?.$case) {
          case "runProcessResponse":
            resolve(response.runProcessResponse);
            break;
          case "error":
            reject(Error(response.error));
            break;
          default:
            reject(
              Error(
                "Invalid response '" + response?.$case + "' for 'RunProcessRequest'"
              )
            );
        }
      }
    );
  });
}

export async function sendMacosInputMethodRequest(
  request: MacosInputMethodRequest
): Promise<MacosInputMethodResponse> {
  return new Promise((resolve, reject) => {
    sendMessage(
      { $case: "macosInputMethodRequest", macosInputMethodRequest: request },
      (response) => {
        switch (response?.$case) {
          case "macosInputMethodResponse":
            resolve(response.macosInputMethodResponse);
            break;
          case "error":
            reject(Error(response.error));
            break;
          default:
            reject(
              Error(
                "Invalid response '" + response?.$case + "' for 'MacosInputMethodRequest'"
              )
            );
        }
      }
    );
  });
}

export async function sendPseudoterminalWriteRequest(
  request: PseudoterminalWriteRequest
): Promise<void> {
  return new Promise((resolve, reject) => {
    sendMessage(
      { $case: "pseudoterminalWriteRequest", pseudoterminalWriteRequest: request },
      (response) => {
        switch (response?.$case) {
          case "success":
            resolve();
            break;
          case "error":
            reject(Error(response.error));
            break;
          default:
            reject(
              Error(
                "Invalid response '" + response?.$case + "' for 'PseudoterminalWriteRequest'"
              )
            );
        }
      }
    );
  });
}

export async function sendWriteFileRequest(
  request: WriteFileRequest
): Promise<void> {
  return new Promise((resolve, reject) => {
    sendMessage(
      { $case: "writeFileRequest", writeFileRequest: request },
      (response) => {
        switch (response?.$case) {
          case "success":
            resolve();
            break;
          case "error":
            reject(Error(response.error));
            break;
          default:
            reject(
              Error(
                "Invalid response '" + response?.$case + "' for 'WriteFileRequest'"
              )
            );
        }
      }
    );
  });
}

export async function sendUpdateSettingsPropertyRequest(
  request: UpdateSettingsPropertyRequest
): Promise<void> {
  return new Promise((resolve, reject) => {
    sendMessage(
      { $case: "updateSettingsPropertyRequest", updateSettingsPropertyRequest: request },
      (response) => {
        switch (response?.$case) {
          case "success":
            resolve();
            break;
          case "error":
            reject(Error(response.error));
            break;
          default:
            reject(
              Error(
                "Invalid response '" + response?.$case + "' for 'UpdateSettingsPropertyRequest'"
              )
            );
        }
      }
    );
  });
}

export async function sendInsertTextRequest(
  request: InsertTextRequest
): Promise<void> {
  return new Promise((resolve, reject) => {
    sendMessage(
      { $case: "insertTextRequest", insertTextRequest: request },
      (response) => {
        switch (response?.$case) {
          case "success":
            resolve();
            break;
          case "error":
            reject(Error(response.error));
            break;
          default:
            reject(
              Error(
                "Invalid response '" + response?.$case + "' for 'InsertTextRequest'"
              )
            );
        }
      }
    );
  });
}

export async function sendUpdateApplicationPropertiesRequest(
  request: UpdateApplicationPropertiesRequest
): Promise<void> {
  return new Promise((resolve, reject) => {
    sendMessage(
      { $case: "updateApplicationPropertiesRequest", updateApplicationPropertiesRequest: request },
      (response) => {
        switch (response?.$case) {
          case "success":
            resolve();
            break;
          case "error":
            reject(Error(response.error));
            break;
          default:
            reject(
              Error(
                "Invalid response '" + response?.$case + "' for 'UpdateApplicationPropertiesRequest'"
              )
            );
        }
      }
    );
  });
}

export async function sendUpdateDefaultsPropertyRequest(
  request: UpdateDefaultsPropertyRequest
): Promise<void> {
  return new Promise((resolve, reject) => {
    sendMessage(
      { $case: "updateDefaultsPropertyRequest", updateDefaultsPropertyRequest: request },
      (response) => {
        switch (response?.$case) {
          case "success":
            resolve();
            break;
          case "error":
            reject(Error(response.error));
            break;
          default:
            reject(
              Error(
                "Invalid response '" + response?.$case + "' for 'UpdateDefaultsPropertyRequest'"
              )
            );
        }
      }
    );
  });
}

export async function sendTelemetryAliasRequest(
  request: TelemetryAliasRequest
): Promise<void> {
  return new Promise((resolve, reject) => {
    sendMessage(
      { $case: "telemetryAliasRequest", telemetryAliasRequest: request },
      (response) => {
        switch (response?.$case) {
          case "success":
            resolve();
            break;
          case "error":
            reject(Error(response.error));
            break;
          default:
            reject(
              Error(
                "Invalid response '" + response?.$case + "' for 'TelemetryAliasRequest'"
              )
            );
        }
      }
    );
  });
}

export async function sendTelemetryIdentifyRequest(
  request: TelemetryIdentifyRequest
): Promise<void> {
  return new Promise((resolve, reject) => {
    sendMessage(
      { $case: "telemetryIdentifyRequest", telemetryIdentifyRequest: request },
      (response) => {
        switch (response?.$case) {
          case "success":
            resolve();
            break;
          case "error":
            reject(Error(response.error));
            break;
          default:
            reject(
              Error(
                "Invalid response '" + response?.$case + "' for 'TelemetryIdentifyRequest'"
              )
            );
        }
      }
    );
  });
}

export async function sendTelemetryTrackRequest(
  request: TelemetryTrackRequest
): Promise<void> {
  return new Promise((resolve, reject) => {
    sendMessage(
      { $case: "telemetryTrackRequest", telemetryTrackRequest: request },
      (response) => {
        switch (response?.$case) {
          case "success":
            resolve();
            break;
          case "error":
            reject(Error(response.error));
            break;
          default:
            reject(
              Error(
                "Invalid response '" + response?.$case + "' for 'TelemetryTrackRequest'"
              )
            );
        }
      }
    );
  });
}

export async function sendOnboardingRequest(
  request: OnboardingRequest
): Promise<void> {
  return new Promise((resolve, reject) => {
    sendMessage(
      { $case: "onboardingRequest", onboardingRequest: request },
      (response) => {
        switch (response?.$case) {
          case "success":
            resolve();
            break;
          case "error":
            reject(Error(response.error));
            break;
          default:
            reject(
              Error(
                "Invalid response '" + response?.$case + "' for 'OnboardingRequest'"
              )
            );
        }
      }
    );
  });
}

export async function sendWindowFocusRequest(
  request: WindowFocusRequest
): Promise<void> {
  return new Promise((resolve, reject) => {
    sendMessage(
      { $case: "windowFocusRequest", windowFocusRequest: request },
      (response) => {
        switch (response?.$case) {
          case "success":
            resolve();
            break;
          case "error":
            reject(Error(response.error));
            break;
          default:
            reject(
              Error(
                "Invalid response '" + response?.$case + "' for 'WindowFocusRequest'"
              )
            );
        }
      }
    );
  });
}

export async function sendOpenInExternalApplicationRequest(
  request: OpenInExternalApplicationRequest
): Promise<void> {
  return new Promise((resolve, reject) => {
    sendMessage(
      { $case: "openInExternalApplicationRequest", openInExternalApplicationRequest: request },
      (response) => {
        switch (response?.$case) {
          case "success":
            resolve();
            break;
          case "error":
            reject(Error(response.error));
            break;
          default:
            reject(
              Error(
                "Invalid response '" + response?.$case + "' for 'OpenInExternalApplicationRequest'"
              )
            );
        }
      }
    );
  });
}

export async function sendUpdateConfigPropertyRequest(
  request: UpdateConfigPropertyRequest
): Promise<void> {
  return new Promise((resolve, reject) => {
    sendMessage(
      { $case: "updateConfigPropertyRequest", updateConfigPropertyRequest: request },
      (response) => {
        switch (response?.$case) {
          case "success":
            resolve();
            break;
          case "error":
            reject(Error(response.error));
            break;
          default:
            reject(
              Error(
                "Invalid response '" + response?.$case + "' for 'UpdateConfigPropertyRequest'"
              )
            );
        }
      }
    );
  });
}

export async function sendPseudoterminalRestartRequest(
  request: PseudoterminalRestartRequest
): Promise<void> {
  return new Promise((resolve, reject) => {
    sendMessage(
      { $case: "pseudoterminalRestartRequest", pseudoterminalRestartRequest: request },
      (response) => {
        switch (response?.$case) {
          case "success":
            resolve();
            break;
          case "error":
            reject(Error(response.error));
            break;
          default:
            reject(
              Error(
                "Invalid response '" + response?.$case + "' for 'PseudoterminalRestartRequest'"
              )
            );
        }
      }
    );
  });
}

export async function sendDebuggerUpdateRequest(
  request: DebuggerUpdateRequest
): Promise<void> {
  return new Promise((resolve, reject) => {
    sendMessage(
      { $case: "debuggerUpdateRequest", debuggerUpdateRequest: request },
      (response) => {
        switch (response?.$case) {
          case "success":
            resolve();
            break;
          case "error":
            reject(Error(response.error));
            break;
          default:
            reject(
              Error(
                "Invalid response '" + response?.$case + "' for 'DebuggerUpdateRequest'"
              )
            );
        }
      }
    );
  });
}

export async function sendAppendToFileRequest(
  request: AppendToFileRequest
): Promise<void> {
  return new Promise((resolve, reject) => {
    sendMessage(
      { $case: "appendToFileRequest", appendToFileRequest: request },
      (response) => {
        switch (response?.$case) {
          case "success":
            resolve();
            break;
          case "error":
            reject(Error(response.error));
            break;
          default:
            reject(
              Error(
                "Invalid response '" + response?.$case + "' for 'AppendToFileRequest'"
              )
            );
        }
      }
    );
  });
}

export async function sendUpdateLocalStateRequest(
  request: UpdateLocalStateRequest
): Promise<void> {
  return new Promise((resolve, reject) => {
    sendMessage(
      { $case: "updateLocalStateRequest", updateLocalStateRequest: request },
      (response) => {
        switch (response?.$case) {
          case "success":
            resolve();
            break;
          case "error":
            reject(Error(response.error));
            break;
          default:
            reject(
              Error(
                "Invalid response '" + response?.$case + "' for 'UpdateLocalStateRequest'"
              )
            );
        }
      }
    );
  });
}

export async function sendCreateDirectoryRequest(
  request: CreateDirectoryRequest
): Promise<void> {
  return new Promise((resolve, reject) => {
    sendMessage(
      { $case: "createDirectoryRequest", createDirectoryRequest: request },
      (response) => {
        switch (response?.$case) {
          case "success":
            resolve();
            break;
          case "error":
            reject(Error(response.error));
            break;
          default:
            reject(
              Error(
                "Invalid response '" + response?.$case + "' for 'CreateDirectoryRequest'"
              )
            );
        }
      }
    );
  });
}

