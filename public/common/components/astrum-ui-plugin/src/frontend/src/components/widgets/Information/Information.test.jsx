
import React from 'react';
import {render} from '@testing-library/react';

import Information from './Information';

describe('Information', () => {
  it('renders boolean values correctly', () => {
    const info = {
      'boolean': false
    };
    const {getByText} = render(<Information information={info}/>);

    expect(getByText('false')).toBeInTheDocument();
  });

  it('renders numeric values correctly', () => {
    const info = {
      'numeric': 0
    };
    const {getByText} = render(<Information information={info}/>);

    expect(getByText('0')).toBeInTheDocument();
  });

  it('renders text values correctly', () => {
    const info = {
      'text': 'test'
    };
    const {getByText} = render(<Information information={info}/>);

    expect(getByText('test')).toBeInTheDocument();
  });
});
