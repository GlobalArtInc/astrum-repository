
import React from 'react';
import {render} from '@testing-library/react';

import SectionFooter from './SectionFooter';

describe('SectionFooter', () => {
  it('renders correctly', () => {
    const footer = 'Footer Content';
    const {container} = render(
        <SectionFooter>{footer}</SectionFooter>
    );
    expect(container).toHaveTextContent(footer);
  });
});
