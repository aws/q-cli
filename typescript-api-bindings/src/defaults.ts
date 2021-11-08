import { sendGetDefaultsPropertyRequest, sendUpdateDefaultsPropertyRequest } from "./requests"

const get = async (
  key: string,
): Promise<boolean | string | number | null> => {
    let response = await sendGetDefaultsPropertyRequest({
        key: key,
    });

    switch (response.value?.type?.$case) {
        case "boolean":
            return response.value?.type?.boolean
        case "integer":
            return response.value?.type?.integer
        case "string":
            return response.value.type.string
        case "null":
            return null
    }

    return null
}


const setString = async (
  key: string,
  value: string
): Promise<void> =>
    sendUpdateDefaultsPropertyRequest({
    key: key,
    value: { type: { $case: "string", string: value } },
  });

  const setBoolean = async (
    key: string,
    value: boolean
  ): Promise<void> =>
      sendUpdateDefaultsPropertyRequest({
      key: key,
      value: { type: { $case: "boolean", boolean: value } },
    });

  const setNumber = async (
    key: string,
    value: number
  ): Promise<void> =>
      sendUpdateDefaultsPropertyRequest({
      key: key,
      value: { type: { $case: "integer", integer: value } },
    });

  const remove = async (
    key: string,
  ): Promise<void> =>
    sendUpdateDefaultsPropertyRequest({
      key: key,
    });

const Defaults = { get, setString, setBoolean, setNumber, remove };

export default Defaults;