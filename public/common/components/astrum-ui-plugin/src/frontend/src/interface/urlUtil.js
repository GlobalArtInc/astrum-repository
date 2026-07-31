
import { compose, filter, not, isNil, nth, toPairs, join, map } from 'ramda';

const toNonNullPairs = compose(filter(compose(not, isNil, nth(1))), toPairs);
const pairToURIParam = compose(join('='), map(encodeURIComponent));
/**
 * {k: String} -> String
 * Converts object to URI params string omitting empty values
 */
export const toURIParams = compose(join('&'), map(pairToURIParam), toNonNullPairs);
