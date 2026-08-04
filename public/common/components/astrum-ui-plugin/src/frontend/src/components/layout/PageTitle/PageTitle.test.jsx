
import React from 'react';

import {render} from '@testing-library/react';

import {faInfo} from '@fortawesome/free-solid-svg-icons';

import PageTitle from './PageTitle';

describe('PageTitle', () => {
  it('renders the text with an icon', () => {
    const {container} = render(
        <PageTitle icon={faInfo} text="text"/>
    );
    expect(container.querySelector('.nx-icon')).toBeInTheDocument();
  });

  it('renders the text without an icon', () => {
    const {container} = render(
        <PageTitle text="text"/>
    );
    expect(container).toHaveTextContent('text');
  });

  it('renders the text with a description', () => {
    const {container} = render(
        <PageTitle text="text" description="description"/>
    );
    expect(container).toHaveTextContent('text');
    expect(container).toHaveTextContent('description');
  });
});
