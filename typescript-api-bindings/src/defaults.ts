import {
  sendGetDefaultsPropertyRequest,
  sendUpdateDefaultsPropertyRequest
} from './requests';

export async function get(
  key: string
): Promise<boolean | string | number | null> {
  const response = await sendGetDefaultsPropertyRequest({
    key
  });

  switch (response.value?.type?.$case) {
    case 'boolean':
      return response.value?.type?.boolean;
    case 'integer':
      return response.value?.type?.integer;
    case 'string':
      return response.value.type.string;
    case 'null':
    default:
      return null;
  }
}

export async function setString(key: string, value: string): Promise<void> {
  return sendUpdateDefaultsPropertyRequest({
    key,
    value: { type: { $case: 'string', string: value } }
  });
}

export async function setBoolean(key: string, value: boolean): Promise<void> {
  return sendUpdateDefaultsPropertyRequest({
    key,
    value: { type: { $case: 'boolean', boolean: value } }
  });
}

export async function setNumber(key: string, value: number): Promise<void> {
  return sendUpdateDefaultsPropertyRequest({
    key,
    value: { type: { $case: 'integer', integer: value } }
  });
}

export async function remove(key: string): Promise<void> {
  return sendUpdateDefaultsPropertyRequest({
    key
  });
}
