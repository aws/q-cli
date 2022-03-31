import { sendRunProcessRequest } from './requests';

export async function run({
  executable,
  args,
  environment,
  workingDirectory
}: {
  executable: string;
  args: string[];
  environment?: Record<string, string>;
  workingDirectory?: string;
}) {
  const env = environment ?? {};
  return sendRunProcessRequest({
    executable,
    arguments: args,
    env: Object.keys(env).map(key => ({ key, value: env[key] })),
    workingDirectory: workingDirectory ?? '/'
  });
}
