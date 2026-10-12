import { describe, expect, it } from 'vitest';
import { ApiError } from '../../api';
import { makeTranslate } from '../../i18n';
import {
  browseHref,
  EMPTY_SELECTOR,
  insertAt,
  toCreateRequest,
  toUpdateRequest,
  validateSelector,
} from './model';
import { expressionProblem } from './SelectorPreview';
import { filterSelectors } from './SelectorsPage';

const t = makeTranslate('en');

describe('the content selector form', () => {
  it('trims what it sends and never sends the name on update', () => {
    const values = { name: ' team ', description: ' Team files ', expression: ' format == "raw" ' };
    expect(toCreateRequest(values)).toEqual({
      name: 'team',
      description: 'Team files',
      expression: 'format == "raw"',
    });
    expect(toUpdateRequest(values)).toEqual({ description: 'Team files', expression: 'format == "raw"' });
  });

  it('checks the name only when creating', () => {
    expect(validateSelector(t, EMPTY_SELECTOR, true)).toEqual({
      name: 'Fill this in',
      expression: 'Fill this in',
    });
    expect(validateSelector(t, { ...EMPTY_SELECTOR, name: '_x', expression: 'x' }, true).name).toContain(
      'letters',
    );
    expect(validateSelector(t, { ...EMPTY_SELECTOR, expression: 'x' }, false)).toEqual({});
  });

  it('inserts syntax at the cursor with spaces around it', () => {
    expect(insertAt('format', '==', 6, 6)).toEqual({ text: 'format ==', caret: 9 });
    expect(insertAt('format "raw"', '==', 7, 7)).toEqual({ text: 'format == "raw"', caret: 9 });
    expect(insertAt('', 'path', 0, 0)).toEqual({ text: 'path', caret: 4 });
    expect(insertAt('a or b', 'and', 2, 4)).toEqual({ text: 'a and b', caret: 5 });
  });

  it('links matches to the browse page', () => {
    expect(browseHref('raw files', '/a b/c.txt')).toBe('/browse/raw%20files/a%20b/c.txt');
  });

  it('reads the expression problem out of a 400', () => {
    const error = new ApiError(400, 'x', { fieldErrors: [{ id: 'expression', message: 'unexpected end' }] });
    expect(expressionProblem(error)).toBe('unexpected end');
    expect(expressionProblem(new ApiError(404, 'gone'))).toBeNull();
  });

  it('filters the list by name, description and expression', () => {
    const list = [
      { name: 'a', type: 'csel', description: null, expression: 'format == "npm"' },
      { name: 'b', type: 'csel', description: 'Team', expression: 'path =^ "/x"' },
    ];
    expect(filterSelectors(list, 'npm').map((item) => item.name)).toEqual(['a']);
    expect(filterSelectors(list, 'team').map((item) => item.name)).toEqual(['b']);
  });
});
