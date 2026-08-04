
import React from 'react';

import {render} from '@testing-library/react';

import Page from './Page';

describe('Page', () => {
  it('renders with custom class', () => {
    const content = 'Page content';
    const {container} = render(
        <Page className="test">{content}</Page>
    );
    expect(container).toHaveTextContent(content);
    expect(container.querySelector('.test')).toBeInTheDocument();
  });
});
