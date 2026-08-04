
import React from 'react';
import {render, screen} from '@testing-library/react';
import ReadOnlyField from './ReadOnlyField';

describe('readOnlyRenderField', () => {
  const label = 'Best Label';
  const data = 'email@test.com';

  const selectors = {
    label: () => screen.getByText(label),
    data: () => screen.getByText(data),
    queryLabel: () => screen.queryByText(label),
    queryData: () => screen.queryByText(data),
  };

  const renderComponent = () =>
    render(<ReadOnlyField label={label} value={data} />);

  it('returns an element', () => {
    renderComponent();

    expect(selectors.label()).toBeInTheDocument();
    expect(selectors.data()).toBeInTheDocument();
  });

  it('does not return an element if the data is empty, null or undefined', async () => {
    const {rerender} = renderComponent();

    rerender(<ReadOnlyField label={label} />);

    expect(selectors.queryLabel()).not.toBeInTheDocument();
    expect(selectors.queryData()).not.toBeInTheDocument();

    rerender(<ReadOnlyField label={label} value="" />);

    expect(selectors.queryLabel()).not.toBeInTheDocument();
    expect(selectors.queryData()).not.toBeInTheDocument();

    rerender(<ReadOnlyField label={label} value={null} />);

    expect(selectors.queryLabel()).not.toBeInTheDocument();
    expect(selectors.queryData()).not.toBeInTheDocument();

    rerender(<ReadOnlyField label={label} value={undefined} />);

    expect(selectors.queryLabel()).not.toBeInTheDocument();
    expect(selectors.queryData()).not.toBeInTheDocument();
  });
});
