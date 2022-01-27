import {
  sendWriteFileRequest,
  sendReadFileRequest,
  sendDestinationOfSymbolicLinkRequest,
  sendContentsOfDirectoryRequest,
  sendAppendToFileRequest,
} from './requests';

export async function write(path: string, contents: string) {
  return sendWriteFileRequest({
    path: { path: path, expandTildeInPath: true },
    data: { $case: 'text', text: contents },
  });
}

export async function append(path: string, contents: string) {
  return sendAppendToFileRequest({
    path: { path: path, expandTildeInPath: true },
    data: { $case: 'text', text: contents },
  });
}

export async function read(path: string) {
  let response = await sendReadFileRequest({
    path: { path: path, expandTildeInPath: true },
  });
  if (response.type?.$case === 'text') {
    return response.type.text;
  } else {
    return null;
  }
}

export async function list(path: string) {
  let response = await sendContentsOfDirectoryRequest({
    directory: { path: path, expandTildeInPath: true },
  });
  return response.fileNames;
}

export async function destinationOfSymbolicLink(path: string) {
  let response = await sendDestinationOfSymbolicLinkRequest({
    path: { path: path, expandTildeInPath: true },
  });
  return response.destination?.path;
}
