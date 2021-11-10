import { PTYOptions } from '../src/pty';
import Shell from '../src/shell';

const filterOutliers = (arr: number[]): number[] => {
  const values = [...arr];
  values.sort((a, b) => a - b);

  const q1 = values[Math.floor(values.length / 4)];
  const q3 = values[Math.ceil(values.length * (3 / 4))];
  const iqr = q3 - q1;

  const maxValue = q3 + iqr * 1.5;
  const minValue = q1 - iqr * 1.5;

  return values.filter(x => x <= maxValue && x >= minValue);
};

const computeAverageStartupTime = async (opts: PTYOptions, n = 5) => {
  const times: number[] = [];
  for (let i = 0; i < n; i += 1) {
    const shell = new Shell(opts);
    // eslint-disable-next-line no-await-in-loop
    await shell.initialized();
    times.push(shell.startupTime);
    shell.kill();
  }
  const filtered = filterOutliers(times);
  return filtered.reduce((a, b) => a + b) / filtered.length;
};

test('zsh: fig startup time', async () => {
  const figMinimal = await computeAverageStartupTime(
    {
      shell: 'zsh',
      env: { ...process.env, ZDOTDIR: `/usr/home/withfig` },
    },
    100
  );
  const withoutFig = await computeAverageStartupTime(
    {
      shell: 'zsh',
      env: { ...process.env, ZDOTDIR: `/usr/home/withoutfig` },
    },
    100
  );
  console.log({ figMinimal, withoutFig });
  expect(withoutFig).toBeGreaterThan(0);
  expect(figMinimal).toBeGreaterThan(0);
  expect(figMinimal - withoutFig).toBeLessThan(50);
}, 20000);

test('bash: fig startup time', async () => {
  const figMinimal = await computeAverageStartupTime(
    {
      shell: 'bash',
      args: ['--init-file', '/usr/home/withfig/.bashrc'],
    },
    100
  );
  const withoutFig = await computeAverageStartupTime(
    {
      shell: 'bash',
      args: ['--init-file', '/usr/home/withoutfig/.bashrc'],
    },
    100
  );
  console.log({ figMinimal, withoutFig });
  expect(withoutFig).toBeGreaterThan(0);
  expect(figMinimal).toBeGreaterThan(0);
  expect(figMinimal - withoutFig).toBeLessThan(50);
}, 20000);
