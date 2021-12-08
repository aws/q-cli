import { TelemetryProperty } from './fig';
import {
  sendTelemetryAliasRequest,
  sendTelemetryIdentifyRequest,
  sendTelemetryTrackRequest,
} from './requests';

export const track = async (event: string, properties: Record<string, string>) => {
  // convert to internal type 'TelemetryProperty'
  const props = Object.keys(properties).reduce((array, key) => {
    const entry: TelemetryProperty = { key, value: properties[key] };
    array.push(entry);
    return array;
  }, ([] as unknown) as [TelemetryProperty]);

  return await sendTelemetryTrackRequest({ event, properties: props });
};

export const alias = async (userId: string) => sendTelemetryAliasRequest({ userId });

export const identify = async (traits: Record<string, string>) => {
  // convert to internal type 'TelemetryProperty'
  const props = Object.keys(traits).reduce((array, key) => {
    const entry: TelemetryProperty = { key, value: traits[key] };
    array.push(entry);
    return array;
  }, ([] as unknown) as [TelemetryProperty]);

  return await sendTelemetryIdentifyRequest({ traits: props });
};
