import React from 'react';
import { fireEvent, render, screen } from '@testing-library/react';
import { Theme } from '@radix-ui/themes';

import { CSELEditor } from '../CSELEditor';

function renderEditor(value: string, onValidationChange = jest.fn()) {
  render(
    <Theme>
      <CSELEditor value={value} onChange={jest.fn()} onValidationChange={onValidationChange} />
    </Theme>
  );
  return screen.getByTestId('content-selector-expression');
}

describe('CSELEditor', () => {
  it('does not report a missing expression on a fresh form', () => {
    const onValidationChange = jest.fn();
    const textarea = renderEditor('', onValidationChange);

    expect(screen.queryByText('Expression is required')).not.toBeInTheDocument();
    expect(textarea).not.toHaveClass('csel-editor__textarea--error');
    expect(onValidationChange).toHaveBeenLastCalledWith(expect.objectContaining({ hasBlockingErrors: true }));
  });

  it('reports a missing expression once the field has been left', () => {
    const textarea = renderEditor('');

    fireEvent.focus(textarea);
    fireEvent.blur(textarea);

    expect(screen.getByText('Expression is required')).toBeInTheDocument();
    expect(textarea).toHaveClass('csel-editor__textarea--error');
  });

  it('reports errors of an expression that is there', () => {
    renderEditor('format ==');

    expect(document.querySelector('.csel-editor__validation')).toBeInTheDocument();
  });
});
