import { sendGetConfigPropertyRequest, sendUpdateConfigPropertyRequest } from "./requests";

const get = async(key: string) =>  {
    let response = await sendGetConfigPropertyRequest({ key: key})
    return response.value
}

const set = (key: string, value: string) => 
    sendUpdateConfigPropertyRequest({ key: key, value: value})
    
const remove = (key: string) => 
    sendUpdateConfigPropertyRequest({ key: key, value: undefined})

const Config = { get, set, remove };

export default Config;