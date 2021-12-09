import { sendPositionWindowRequest } from './requests';

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

const setFrame = async (frame: {
  width: number;
  height: number;
  anchorX: number;
  offsetFromBaseline: number | undefined;
}) =>
  sendPositionWindowRequest({
    size: { width: frame.width, height: frame.height },
    anchor: { x: frame.anchorX, y: frame.offsetFromBaseline ?? 0 },
  });

const WindowPosition = { isValidFrame, setFrame };

export default WindowPosition;
