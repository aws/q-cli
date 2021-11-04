import Shell from '../shell';

import { makeTestsForShell } from './utils';

const shell = new Shell({ shell: 'zsh' });
makeTestsForShell(shell, process.env.ENV_NAME ?? '');
