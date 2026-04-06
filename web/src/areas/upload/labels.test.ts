import { describe, expect, it } from 'vitest';
import { makeTranslate } from '../../i18n';
import { fieldHint, fieldLabel, fieldPlaceholder } from './labels';
import type { UploadFieldSpec } from './model';

const t = makeTranslate('en');

const pathId: UploadFieldSpec = {
  name: 'pathId',
  type: 'STRING',
  description: 'Package path, such as src/contrib',
  optional: true,
  group: null,
  groupHelpText: null,
  options: null,
};

describe('labels of the R upload form', () => {
  it('name the package path in the words of the page', () => {
    expect(fieldLabel(t, 'r', pathId)).toBe('Package Path');
    expect(fieldHint(t, 'r', pathId)).toBe(
      'Directory of the package, for example bin/windows/contrib/4.4; source packages go to src/contrib when it is empty',
    );
    expect(fieldPlaceholder(t, 'r', pathId)).toBe('src/contrib');
    expect(fieldLabel(t, 'other', pathId)).toBe('Path Id');
  });
});
