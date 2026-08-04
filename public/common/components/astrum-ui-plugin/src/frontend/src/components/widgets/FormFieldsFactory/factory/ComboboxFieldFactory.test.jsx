
import React from 'react';
import {render, screen, waitFor} from '@testing-library/react';
import {when} from 'jest-when';
import ComboboxFieldFactory from './ComboboxFieldFactory';
import {SUPPORTED_FIELD_TYPES} from '../FormFieldsFactoryConstants';
import ExtAPIUtils from '../../../../interface/ExtAPIUtils';

jest.mock('../../../../interface/ExtAPIUtils');

describe('ComboboxFieldFactory', () => {
  describe('metadata', () => {
    it('exports supported field types', () => {
      expect(ComboboxFieldFactory.types).toBe(SUPPORTED_FIELD_TYPES.COMBOBOX);
    });

    it('exports a component', () => {
      expect(ComboboxFieldFactory.component).toBeDefined();
      expect(typeof ComboboxFieldFactory.component).toBe('function');
    });
  });

  describe('Field component', () => {
    const Field = ComboboxFieldFactory.component;
    const mockOnChange = jest.fn();

    const mockData = [
      {id: 'option1', name: 'Option 1'},
      {id: 'option2', name: 'Option 2'},
      {id: 'option3', name: 'Option 3'}
    ];

    const defaultProps = {
      id: 'selector',
      dynamicProps: {
        storeApi: 'TestStore.read'
      },
      current: {
        context: {
          data: {selector: ''},
          pristineData: {selector: ''}
        }
      },
      onChange: mockOnChange
    };

    beforeEach(() => {
      mockOnChange.mockClear();
      ExtAPIUtils.extAPIRequest.mockClear();
      ExtAPIUtils.checkForError.mockClear();
      ExtAPIUtils.extractResult.mockClear();

      when(ExtAPIUtils.extAPIRequest)
        .mockResolvedValue({result: {data: mockData}});
      when(ExtAPIUtils.checkForError)
        .mockReturnValue(undefined);
      when(ExtAPIUtils.extractResult)
        .mockReturnValue(mockData);
    });

    it('throws error when storeApi is not a string', () => {
      const props = {
        ...defaultProps,
        dynamicProps: {
          storeApi: null
        }
      };

      expect(() => render(<Field {...props} />)).toThrow('storeApi must be a string');
    });

    it('renders NxFormSelect when allowAutocomplete is false', async () => {
      render(<Field {...defaultProps} />);

      await waitFor(() => {
        expect(screen.getByRole('combobox')).toBeInTheDocument();
      });
    });

    it('renders NxCombobox when allowAutocomplete is true', async () => {
      const props = {
        ...defaultProps,
        dynamicProps: {
          ...defaultProps.dynamicProps,
          allowAutocomplete: true
        }
      };

      render(<Field {...props} />);

      await waitFor(() => {
        expect(screen.getByLabelText('combobox')).toBeInTheDocument();
      });
    });

    it('fetches data on mount when allowAutocomplete is false', async () => {
      render(<Field {...defaultProps} />);

      await waitFor(() => {
        expect(ExtAPIUtils.extAPIRequest).toHaveBeenCalledWith('TestStore', 'read', null);
      });
    });

    it('does not fetch data on mount when allowAutocomplete is true and no value', async () => {
      const props = {
        ...defaultProps,
        dynamicProps: {
          ...defaultProps.dynamicProps,
          allowAutocomplete: true
        }
      };

      render(<Field {...props} />);

      // Verify no fetch happens by checking API was not called
      await waitFor(() => {
        expect(ExtAPIUtils.extAPIRequest).not.toHaveBeenCalled();
      });
    });

    it('fetches data on mount when allowAutocomplete is true and has value', async () => {
      const props = {
        ...defaultProps,
        dynamicProps: {
          ...defaultProps.dynamicProps,
          allowAutocomplete: true
        },
        current: {
          context: {
            data: {selector: 'option1'},
            pristineData: {selector: ''}
          }
        }
      };

      render(<Field {...props} />);

      await waitFor(() => {
        expect(ExtAPIUtils.extAPIRequest).toHaveBeenCalledWith('TestStore', 'read', {query: 'option1'});
      });
    });


    it('renders read-only value with display name', async () => {
      const props = {
        ...defaultProps,
        dynamicProps: {
          ...defaultProps.dynamicProps,
          readOnly: true
        },
        current: {
          context: {
            data: {selector: 'option1'},
            pristineData: {selector: ''}
          }
        }
      };

      render(<Field {...props} />);

      await waitFor(() => {
        expect(screen.queryByRole('combobox')).not.toBeInTheDocument();
      });
    });

    it('renders read-only value as-is when not found in data', async () => {
      const props = {
        ...defaultProps,
        dynamicProps: {
          ...defaultProps.dynamicProps,
          readOnly: true
        },
        current: {
          context: {
            data: {selector: 'unknown-value'},
            pristineData: {selector: ''}
          }
        }
      };

      const {container} = render(<Field {...props} />);

      await waitFor(() => {
        expect(container.textContent).toBe('unknown-value');
      });
    });

    it('includes empty option in select dropdown', async () => {
      render(<Field {...defaultProps} />);

      await waitFor(() => {
        const options = screen.getAllByRole('option');
        expect(options[0]).toHaveValue('');
      });
    });

    it('handles API error', async () => {
      when(ExtAPIUtils.checkForError)
        .mockImplementation(() => {
          throw new Error('API Error');
        });

      render(<Field {...defaultProps} />);

      await waitFor(() => {
        // Component should handle error gracefully
        expect(screen.getByRole('combobox')).toBeInTheDocument();
      });
    });
  });
});
