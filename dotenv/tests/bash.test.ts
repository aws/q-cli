import Shell from '../src/shell';
import { makeTestsForShell } from './utils';

makeTestsForShell(new Shell({ shell: 'bash' }));
