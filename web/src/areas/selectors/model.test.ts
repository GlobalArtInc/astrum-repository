import { describe, expect, it } from 'vitest';
import { ApiError } from '../../api';
import { makeTranslate } from '../../i18n';
import { explainExpression } from './explain';
import { EMPTY_SELECTOR, toCreateRequest, toUpdateRequest, validateSelector } from './model';
import { expressionProblem } from './preview';
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

  it('explains an expression in words', () => {
    expect(explainExpression(t, 'format == "raw" and path =^ "/docs/"')).toBe(
      'Content matches when: format is "raw" AND path starts with "/docs/"',
    );
    expect(explainExpression(t, '(format != "a" || path =~ "/x/.*")')).toBe(
      'Content matches when: ( format is not "a" OR path matches the pattern "/x/.*" )',
    );
  });

  it('reads the expression problem out of a 400', () => {
    const error = new ApiError(400, 'x', { fieldErrors: [{ id: 'expression', message: 'unexpected end' }] });
    expect(expressionProblem(error)).toBe('unexpected end');
    expect(expressionProblem(new ApiError(404, 'gone'))).toBeNull();
  });

  it('filters the list by name and description', () => {
    const list = [
      { name: 'a', type: 'csel', description: null, expression: 'format == "npm"' },
      { name: 'b', type: 'csel', description: 'Team', expression: 'path =^ "/x"' },
    ];
    expect(filterSelectors(list, 'b').map((item) => item.name)).toEqual(['b']);
    expect(filterSelectors(list, 'team').map((item) => item.name)).toEqual(['b']);
    expect(filterSelectors(list, 'npm')).toEqual([]);
  });
});
