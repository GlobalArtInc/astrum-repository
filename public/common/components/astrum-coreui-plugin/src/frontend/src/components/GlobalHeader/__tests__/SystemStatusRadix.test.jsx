

import React from 'react';
import {render} from '@testing-library/react';
import '@testing-library/jest-dom';

const mockSystemStatusBell = jest.fn(() => null);

jest.mock('@globalart/astrum-ui-plugin', () => ({
  SystemStatusBell: (...args) => mockSystemStatusBell(...args),
}));

import SystemStatusRadix from '../SystemStatusRadix';

describe('SystemStatusRadix', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('renders SystemStatusBell with the self-hosted class-name prefix', () => {
    render(<SystemStatusRadix />);

    expect(mockSystemStatusBell).toHaveBeenCalledTimes(1);
    expect(mockSystemStatusBell.mock.calls[0][0]).toEqual({
      className: 'as-system-status-radix',
    });
  });
});
