
import React from 'react';
import {render} from '@testing-library/react';

import Section from './Section';

describe('Section', () => {
  it('renders correctly', () => {
    const name = 'Section Name';
    const {container} = render(
        <Section>{name}</Section>
    );
    expect(container).toHaveTextContent(name);
  });
});
