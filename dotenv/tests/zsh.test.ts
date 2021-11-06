import Shell from '../shell';
import { makeTestsForShell } from './utils';

makeTestsForShell(new Shell({ shell: 'zsh' }));
