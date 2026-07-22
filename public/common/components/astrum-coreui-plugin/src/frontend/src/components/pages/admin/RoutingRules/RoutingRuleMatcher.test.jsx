
import React from 'react';
import RoutingRuleMatcherRow from './RoutingRuleMatcherRow';
import {
  screen,
  render
} from "@testing-library/react";
import userEvent from '@testing-library/user-event';

const selectors = {
  removeButton: () => screen.queryByRole('button')
};

describe('RoutingRuleMatcherRow', function() {
  it('should render without remove button', function() {
    const onRemove = jest.fn();
    render(<RoutingRuleMatcherRow
        label="label"
        isPristine={true}
        value=""
        showRemoveButton={false}
        onRemove={onRemove}
    />);

    expect(selectors.removeButton()).not.toBeInTheDocument();
  });

  it('should render with remove button', function() {
    const onRemove = jest.fn();
    render(<RoutingRuleMatcherRow
        label="label"
        isPristine={true}
        value=""
        showRemoveButton={true}
        onRemove={onRemove}
    />);
  
    expect(selectors.removeButton()).toBeInTheDocument();
  });
  
  it('should remove the row when the button is clicked', function() {
    const onRemove = jest.fn();
    render(<RoutingRuleMatcherRow
        label="label"
        isPristine={true}
        value=""
        showRemoveButton={true}
        onRemove={onRemove}
    />);
  
    userEvent.click(selectors.removeButton());
  
    expect(onRemove).toHaveBeenCalled();
  });
});
