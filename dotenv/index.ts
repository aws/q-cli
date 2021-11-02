import Shell from './shell';
import Config from './config';
import Settings from './settings';

const main = async () => {
  Config.reset();
  Settings.reset();
  Config.set({ NEW_VERSION_AVAILABLE: 'v1.0.49' });

  const shell1 = new Shell({
    shell: 'bash',
    mockedCLICommands: { 'app:running': '1' },
  });
  await shell1.initialized();
  const transcript = shell1.pty.getSessionTranscript();
  console.log({ transcript });

  const shell2 = new Shell({
    shell: 'bash',
    mockedCLICommands: { 'app:running': '1' },
  });
  await shell2.initialized();
  const env = await shell2.pty.getEnv();
  Config.reset();
  Settings.reset();
  shell2.pty.resize({ rows: 30, cols: 80 });
  shell2.pty.write('\r');
  await shell2.cli.waitForNextPrompt();
  console.log({ env });

  shell1.kill();
  shell2.kill();
};

main();
