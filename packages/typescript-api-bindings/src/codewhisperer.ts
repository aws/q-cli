import { CodewhispererCustomization as Customization } from "@fig/fig-api-proto/fig";
import { sendCodewhispererListCustomizationRequest } from "./requests.js";

const listCustomizations = async () =>
  (await sendCodewhispererListCustomizationRequest({})).customizations;

export { listCustomizations, Customization };
