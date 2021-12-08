import {
  sendGetDefaultsPropertyRequest,
  sendUpdateDefaultsPropertyRequest,
} from './requests';

export const get = async (key: string): Promise<boolean | string | number | null> => {
  let response = await sendGetDefaultsPropertyRequest({
    key: key,
  });

  switch (response.value?.type?.$case) {
    case 'boolean':
      return response.value?.type?.boolean;
    case 'integer':
      return response.value?.type?.integer;
    case 'string':
      return response.value.type.string;
    case 'null':
      return null;
  }

  return null;
};

export const setString = async (key: string, value: string): Promise<void> =>
  sendUpdateDefaultsPropertyRequest({
    key: key,
    value: { type: { $case: 'string', string: value } },
  });

export const setBoolean = async (key: string, value: boolean): Promise<void> =>
  sendUpdateDefaultsPropertyRequest({
    key: key,
    value: { type: { $case: 'boolean', boolean: value } },
  });

export const setNumber = async (key: string, value: number): Promise<void> =>
  sendUpdateDefaultsPropertyRequest({
    key: key,
    value: { type: { $case: 'integer', integer: value } },
  });

export const remove = async (key: string): Promise<void> =>
  sendUpdateDefaultsPropertyRequest({
    key: key,
  });
