
import React from 'react';
import {render} from '@testing-library/react';

import PageActions from './PageActions';

describe('PageActions', () => {
  it('renders correctly', () => {
    const action = 'Page action';
    const {container} = render(
        <PageActions><span>{action}</span></PageActions>
    );
    expect(container).toHaveTextContent(action);
  });
});
