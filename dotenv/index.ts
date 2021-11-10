import { PTYOptions } from './src/pty';
import Shell from './src/shell';

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
    console.log({ i });
    const shell = new Shell(opts);
    // eslint-disable-next-line no-await-in-loop
    await shell.initialized();
    times.push(shell.startupTime);
    shell.kill('SIGKILL');
  }
  console.log({ times });
  const filtered = filterOutliers(times);
  return filtered.reduce((a, b) => a + b) / filtered.length;
};

const main = async () => {
  const N = 100;
  await computeAverageStartupTime(
    {
      shell: 'bash',
      args: ['--init-file', '/usr/home/withoutfig/.bashrc'],
    },
    N
  );
};

main();
