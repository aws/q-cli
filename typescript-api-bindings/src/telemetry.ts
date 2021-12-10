import { TelemetryProperty } from './fig';
import {
  sendTelemetryAliasRequest,
  sendTelemetryIdentifyRequest,
  sendTelemetryTrackRequest,
} from './requests';

export async function track(event: string, properties: Record<string, string>) {
  // convert to internal type 'TelemetryProperty'
  const props = Object.keys(properties).reduce((array, key) => {
    const entry: TelemetryProperty = { key, value: properties[key] };
    array.push(entry);
    return array;
  }, ([] as unknown) as [TelemetryProperty]);

  return await sendTelemetryTrackRequest({ event, properties: props });
}

export async function alias(userId: string) {
  return sendTelemetryAliasRequest({ userId });
}

export async function identify(traits: Record<string, string>) {
  // convert to internal type 'TelemetryProperty'
  const props = Object.keys(traits).reduce((array, key) => {
    const entry: TelemetryProperty = { key, value: traits[key] };
    array.push(entry);
    return array;
  }, ([] as unknown) as [TelemetryProperty]);

  return await sendTelemetryIdentifyRequest({ traits: props });
}
