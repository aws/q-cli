import {
  sendPositionWindowRequest,
  sendDragWindowRequest,
} from "./requests.js";

// Developer Facing API
export async function isValidFrame(frame: {
  width: number;
  height: number;
  anchorX: number;
}) {
  return sendPositionWindowRequest({
    size: { width: frame.width, height: frame.height },
    anchor: { x: frame.anchorX, y: 0 },
    dryrun: true,
  });
}

export async function setFrame(frame: {
  width: number;
  height: number;
  anchorX: number;
  offsetFromBaseline: number | undefined;
}) {
  return sendPositionWindowRequest({
    size: { width: frame.width, height: frame.height },
    anchor: { x: frame.anchorX, y: frame.offsetFromBaseline ?? 0 },
  });
}

export async function dragWindow() {
  return sendDragWindowRequest({});
}
