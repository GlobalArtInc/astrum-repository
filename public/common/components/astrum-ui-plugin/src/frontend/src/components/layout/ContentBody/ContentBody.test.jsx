
import React from 'react';
import {render} from '@testing-library/react';

import ContentBody from './ContentBody';

describe('ContentBody', () => {
  it('renders correctly', () => {
    const body = 'Content Body'
    const {container} = render(
        <ContentBody className="test">{body}</ContentBody>
    );
    expect(container).toHaveTextContent(body);
  });
});
