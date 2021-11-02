/* Autogenerated by generate-requests.ts on 11/2/2021
 * Do not edit directly! Instead run 'npm run generate-requests'
 */

import {
    ContentsOfDirectoryRequest,
    ContentsOfDirectoryResponse,
    DestinationOfSymbolicLinkRequest,
    DestinationOfSymbolicLinkResponse,
    GetConfigPropertyRequest,
    GetConfigPropertyResponse,
    GetDefaultsPropertyRequest,
    GetDefaultsPropertyResponse,
    GetSettingsPropertyRequest,
    GetSettingsPropertyResponse,
    InsertTextRequest,
    OnboardingRequest,
    OpenInExternalApplicationRequest,
    PositionWindowRequest,
    PositionWindowResponse,
    PseudoterminalExecuteRequest,
    PseudoterminalExecuteResponse,
    PseudoterminalWriteRequest,
    ReadFileRequest,
    ReadFileResponse,
    TelemetryAliasRequest,
    TelemetryIdentifyRequest,
    TelemetryTrackRequest,
    UpdateApplicationPropertiesRequest,
    UpdateConfigPropertyRequest,
    UpdateDefaultsPropertyRequest,
    UpdateSettingsPropertyRequest,
    WindowFocusRequest,
    WriteFileRequest
} from "./fig";
import { sendMessage } from "./core"

export const sendPositionWindowRequest = async (
    request: PositionWindowRequest
): Promise<PositionWindowResponse> =>
    new Promise((resolve, reject) => {
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

export const sendPseudoterminalExecuteRequest = async (
    request: PseudoterminalExecuteRequest
): Promise<PseudoterminalExecuteResponse> =>
    new Promise((resolve, reject) => {
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

export const sendReadFileRequest = async (
    request: ReadFileRequest
): Promise<ReadFileResponse> =>
    new Promise((resolve, reject) => {
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

export const sendContentsOfDirectoryRequest = async (
    request: ContentsOfDirectoryRequest
): Promise<ContentsOfDirectoryResponse> =>
    new Promise((resolve, reject) => {
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

export const sendGetSettingsPropertyRequest = async (
    request: GetSettingsPropertyRequest
): Promise<GetSettingsPropertyResponse> =>
    new Promise((resolve, reject) => {
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

export const sendDestinationOfSymbolicLinkRequest = async (
    request: DestinationOfSymbolicLinkRequest
): Promise<DestinationOfSymbolicLinkResponse> =>
    new Promise((resolve, reject) => {
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

export const sendGetDefaultsPropertyRequest = async (
    request: GetDefaultsPropertyRequest
): Promise<GetDefaultsPropertyResponse> =>
    new Promise((resolve, reject) => {
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

export const sendGetConfigPropertyRequest = async (
    request: GetConfigPropertyRequest
): Promise<GetConfigPropertyResponse> =>
    new Promise((resolve, reject) => {
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

export const sendPseudoterminalWriteRequest = async (
    request: PseudoterminalWriteRequest
): Promise<void> =>
    new Promise((resolve, reject) => {
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


export const sendWriteFileRequest = async (
    request: WriteFileRequest
): Promise<void> =>
    new Promise((resolve, reject) => {
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


export const sendUpdateSettingsPropertyRequest = async (
    request: UpdateSettingsPropertyRequest
): Promise<void> =>
    new Promise((resolve, reject) => {
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


export const sendInsertTextRequest = async (
    request: InsertTextRequest
): Promise<void> =>
    new Promise((resolve, reject) => {
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


export const sendUpdateApplicationPropertiesRequest = async (
    request: UpdateApplicationPropertiesRequest
): Promise<void> =>
    new Promise((resolve, reject) => {
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


export const sendUpdateDefaultsPropertyRequest = async (
    request: UpdateDefaultsPropertyRequest
): Promise<void> =>
    new Promise((resolve, reject) => {
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


export const sendTelemetryAliasRequest = async (
    request: TelemetryAliasRequest
): Promise<void> =>
    new Promise((resolve, reject) => {
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


export const sendTelemetryIdentifyRequest = async (
    request: TelemetryIdentifyRequest
): Promise<void> =>
    new Promise((resolve, reject) => {
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


export const sendTelemetryTrackRequest = async (
    request: TelemetryTrackRequest
): Promise<void> =>
    new Promise((resolve, reject) => {
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


export const sendOnboardingRequest = async (
    request: OnboardingRequest
): Promise<void> =>
    new Promise((resolve, reject) => {
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


export const sendWindowFocusRequest = async (
    request: WindowFocusRequest
): Promise<void> =>
    new Promise((resolve, reject) => {
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


export const sendOpenInExternalApplicationRequest = async (
    request: OpenInExternalApplicationRequest
): Promise<void> =>
    new Promise((resolve, reject) => {
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


export const sendUpdateConfigPropertyRequest = async (
    request: UpdateConfigPropertyRequest
): Promise<void> =>
    new Promise((resolve, reject) => {
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


