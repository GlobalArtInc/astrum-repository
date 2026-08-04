
import React from 'react';
import {render} from '@testing-library/react';

import Textfield from './Textfield';
import UIStrings from '../../../constants/UIStrings';

describe('Textfield', () => {
  it('renders correctly without an error message', () => {
    const {queryByText} = render(<Textfield value="" />);

    expect(queryByText(UIStrings.ERROR.FIELD_REQUIRED)).not.toBeInTheDocument();
  });

  it('renders with a single error message', () => {
    const {queryByText} = render(<Textfield value="" validationErrors={UIStrings.ERROR.FIELD_REQUIRED} />);

    expect(queryByText(UIStrings.ERROR.FIELD_REQUIRED)).toBeInTheDocument();
  });

  it('shows the first error', () => {
    const {queryByText} = render(<Textfield value="" validationErrors={[UIStrings.ERROR.FIELD_REQUIRED, "ERROR_MESSAGE"]}/>);

    expect(queryByText(UIStrings.ERROR.FIELD_REQUIRED)).toBeInTheDocument();
    expect(queryByText("ERROR_MESSAGE")).not.toBeInTheDocument();
  });
});
