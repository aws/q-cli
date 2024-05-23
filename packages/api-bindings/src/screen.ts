import {
  sendGetScreenshotRequest,
  sendOpenContextMenuRequest,
} from "./requests.js";
import { bytesToBase64 } from "./utils.js";

export async function getScreenshot(target: string) {
  const response = await sendGetScreenshotRequest({ target });
  return response.images.map(bytesToBase64);
}

export async function openContextMenu(position: { x: number; y: number }) {
  return sendOpenContextMenuRequest({ position });
}
