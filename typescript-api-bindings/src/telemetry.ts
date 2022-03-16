import { TelemetryProperty } from './fig.pb';
import {
  sendTelemetryAliasRequest,
  sendTelemetryIdentifyRequest,
  sendTelemetryTrackRequest
} from './requests';

export function track(event: string, properties: Record<string, string>) {
  // convert to internal type 'TelemetryProperty'
  const props = Object.keys(properties).reduce((array, key) => {
    const entry: TelemetryProperty = { key, value: properties[key] };
    array.push(entry);
    return array;
  }, ([] as unknown) as [TelemetryProperty]);

  return sendTelemetryTrackRequest({ event, properties: props });
}

export function alias(userId: string) {
  return sendTelemetryAliasRequest({ userId });
}

export function identify(traits: Record<string, string>) {
  // convert to internal type 'TelemetryProperty'
  const props = Object.keys(traits).reduce((array, key) => {
    const entry: TelemetryProperty = { key, value: traits[key] };
    array.push(entry);
    return array;
  }, ([] as unknown) as [TelemetryProperty]);

  return sendTelemetryIdentifyRequest({ traits: props });
}
