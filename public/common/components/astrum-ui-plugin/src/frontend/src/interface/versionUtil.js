
import { unless, isNil, pipe, split, take, join } from 'ramda';

const SEPARATOR = '.';

export const getVersionMajorMinor = unless(isNil, pipe(split(SEPARATOR), take(2), join(SEPARATOR)));
