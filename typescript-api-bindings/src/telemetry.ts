import { TelemetryProperty } from './fig.pb';
import {
  sendTelemetryAliasRequest,
  sendTelemetryIdentifyRequest,
  sendTelemetryPageRequest,
  sendTelemetryTrackRequest
} from './requests';

type Property = string | boolean | number;

export function track(event: string, properties: Record<string, Property>) {
  // convert to internal type 'TelemetryProperty'
  const props = Object.keys(properties).reduce((array, key) => {
    const entry: TelemetryProperty = 
      { key, value: JSON.stringify(JSON.stringify(properties[key])) };
    array.push(entry);
    return array;
  }, ([] as unknown) as [TelemetryProperty]);

  return sendTelemetryTrackRequest({ event, properties: props });
}

export function alias(userId: string) {
  return sendTelemetryAliasRequest({ userId });
}

export function identify(traits: Record<string, Property>) {
  // convert to internal type 'TelemetryProperty'
  const props = Object.keys(traits).reduce((array, key) => {
    const entry: TelemetryProperty = { key, value: JSON.stringify(traits[key]) };
    array.push(entry);
    return array;
  }, ([] as unknown) as [TelemetryProperty]);

  return sendTelemetryIdentifyRequest({ traits: props, jsonBlob: JSON.stringify(traits) });
}

export function page(category: string, name: string, properties: Record<string, Property>) {
  // See more: https://segment.com/docs/connections/spec/page/
  const props = properties;
  props.title = document.title;
  props.path = window.location.pathname;
  props.search = window.location.search;
  props.url = window.location.href;
  props.referrer = document.referrer;

  return sendTelemetryPageRequest({ category, name, jsonBlob: JSON.stringify(props) });
}
