import { CodewhispererCustomization as Customization } from "@fig/fig-api-proto/dist/fig.pb";
import { sendCodewhispererListCustomizationRequest } from "./requests";

const listCustomizations = async () =>
  (await sendCodewhispererListCustomizationRequest({})).customizations;

export { listCustomizations, Customization };
