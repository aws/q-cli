import {
  sendWriteFileRequest,
  sendReadFileRequest,
  sendDestinationOfSymbolicLinkRequest,
  sendContentsOfDirectoryRequest,
} from './requests';

export const write = async (path: string, contents: string) =>
  sendWriteFileRequest({
    path: { path: path, expandTildeInPath: true },
    data: { $case: 'text', text: contents },
  });

export const read = async (path: string) => {
  let response = await sendReadFileRequest({
    path: { path: path, expandTildeInPath: true },
  });
  if (response.type?.$case === 'text') {
    return response.type.text;
  } else {
    return null;
  }
};

export const list = async (path: string) => {
  let response = await sendContentsOfDirectoryRequest({
    directory: { path: path, expandTildeInPath: true },
  });
  return response.fileNames;
};

export const destinationOfSymbolicLink = async (path: string) => {
  let response = await sendDestinationOfSymbolicLinkRequest({
    path: { path: path, expandTildeInPath: true },
  });
  return response.destination?.path;
};
