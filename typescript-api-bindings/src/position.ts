import { sendPositionWindowRequest } from "./requests";

// Developer Facing API
const isValidFrame = async (frame: {
  width: number;
  height: number;
  anchorX: number;
}) =>
sendPositionWindowRequest({
    size: { width: frame.width, height: frame.height },
    anchor: { x: frame.anchorX, y: 0 },
    dryrun: true,
  });

const WindowPosition = { isValidFrame };

export default WindowPosition;
