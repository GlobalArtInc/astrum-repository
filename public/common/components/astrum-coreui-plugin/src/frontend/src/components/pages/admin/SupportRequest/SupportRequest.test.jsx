
import React from 'react';
import {render} from '@testing-library/react';
import UIStrings from '../../../../constants/UIStrings';

import SupportRequest from './SupportRequest';

describe('SupportRequest', () => {
  it('renders correctly', async () => {
    const {container} = render(<SupportRequest/>);

    expect(container).toHaveTextContent(UIStrings.SUPPORT_REQUEST.ACTIONS.submitRequest);
    expect(container).toHaveTextContent(UIStrings.SUPPORT_REQUEST.DESCRIPTION);
  });
});
