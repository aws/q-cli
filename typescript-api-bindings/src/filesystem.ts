import { 
    sendWriteFileRequest,
    sendReadFileRequest,
    sendDestinationOfSymbolicLinkRequest,
    sendContentsOfDirectoryRequest
} from './requests'

const write = async(path: string, contents: string) =>
    sendWriteFileRequest({ path: { path: path, expandTildeInPath: true}, data: { $case: "text", text: contents}})

const read = async(path: string) => {
    let response = await sendReadFileRequest({ path: { path: path, expandTildeInPath: true}})
    return response.data
}

const list = async(path: string) => {
    let response = await sendContentsOfDirectoryRequest({ directory: {path: path, expandTildeInPath: true} })
    return response.fileNames
}

const destinationOfSymbolicLink = async(path: string) => {
    let response = await sendDestinationOfSymbolicLinkRequest({ path: { path: path, expandTildeInPath: true}})
    return response.destination?.path
}

const fs = {write, read, list, destinationOfSymbolicLink}
export default fs;