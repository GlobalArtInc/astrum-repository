
import React from 'react';

import {render} from '@testing-library/react';

import PageHeader from './PageHeader';

describe('PageHeader', () => {
  it('renders with children', () => {
    const content = 'test';
    const {container} = render(
        <PageHeader>{content}</PageHeader>
    );
    expect(container).toHaveTextContent(content);
  });
});
