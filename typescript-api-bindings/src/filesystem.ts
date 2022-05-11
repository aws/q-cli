import {
  sendWriteFileRequest,
  sendReadFileRequest,
  sendDestinationOfSymbolicLinkRequest,
  sendContentsOfDirectoryRequest,
  sendAppendToFileRequest
} from './requests';

export async function write(path: string, contents: string) {
  return sendWriteFileRequest({
    path: { path, expandTildeInPath: true },
    data: { $case: 'text', text: contents }
  });
}

export async function append(path: string, contents: string) {
  return sendAppendToFileRequest({
    path: { path, expandTildeInPath: true },
    data: { $case: 'text', text: contents }
  });
}

export async function read(path: string) {
  const response = await sendReadFileRequest({
    path: { path, expandTildeInPath: true }, isBinaryFile: true
  });
  if (response.type?.$case === 'text') {
    return response.type.text;
  } 
    return null;
  
}

export async function list(path: string) {
  const response = await sendContentsOfDirectoryRequest({
    directory: { path, expandTildeInPath: true }
  });
  return response.fileNames;
}

export async function destinationOfSymbolicLink(path: string) {
  const response = await sendDestinationOfSymbolicLinkRequest({
    path: { path, expandTildeInPath: true }
  });
  return response.destination?.path;
}
