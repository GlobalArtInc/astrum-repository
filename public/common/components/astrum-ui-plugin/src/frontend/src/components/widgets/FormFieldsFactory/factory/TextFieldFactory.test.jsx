
import React from 'react';
import {render, screen} from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import TextFieldFactory from './TextFieldFactory';
import {SUPPORTED_FIELD_TYPES} from '../FormFieldsFactoryConstants';

describe('TextFieldFactory', () => {
  describe('metadata', () => {
    it('exports supported field types', () => {
      expect(TextFieldFactory.types).toBe(SUPPORTED_FIELD_TYPES.TEXT);
    });

    it('exports a component', () => {
      expect(TextFieldFactory.component).toBeDefined();
      expect(typeof TextFieldFactory.component).toBe('function');
    });
  });

  describe('Field component', () => {
    const Field = TextFieldFactory.component;
    const mockOnChange = jest.fn();

    const defaultProps = {
      id: 'testField',
      dynamicProps: {
        type: 'textfield'
      },
      current: {
        context: {
          data: {testField: 'test value'},
          pristineData: {testField: ''}
        }
      },
      onChange: mockOnChange
    };

    beforeEach(() => {
      mockOnChange.mockClear();
    });

    it('renders a text input field', () => {
      render(<Field {...defaultProps} />);

      const input = screen.getByRole('textbox');
      expect(input).toBeInTheDocument();
      expect(input).toHaveValue('test value');
    });

    it('renders a password field when type is password', () => {
      const props = {
        ...defaultProps,
        dynamicProps: {
          type: 'password'
        }
      };

      render(<Field {...props} />);

      const input = screen.getByDisplayValue('test value');
      expect(input).toHaveAttribute('type', 'password');
    });

    it('renders disabled field when disabled prop is true', () => {
      const props = {
        ...defaultProps,
        dynamicProps: {
          type: 'textfield',
          disabled: true
        }
      };

      render(<Field {...props} />);

      const input = screen.getByRole('textbox');
      expect(input).toBeDisabled();
    });

    it('renders read-only value as plain text when readOnly is true', () => {
      const props = {
        ...defaultProps,
        dynamicProps: {
          type: 'textfield',
          readOnly: true
        }
      };

      const {container} = render(<Field {...props} />);

      expect(screen.queryByRole('textbox')).not.toBeInTheDocument();
      expect(container.textContent).toBe('test value');
    });

    it('calls onChange when input value changes', async () => {
      render(<Field {...defaultProps} />);

      const input = screen.getByRole('textbox');
      await userEvent.type(input, 'new value');

      expect(mockOnChange).toHaveBeenCalled();
      expect(mockOnChange).toHaveBeenCalledWith('testField', expect.any(String));
    });

    it('handles empty string value', () => {
      const props = {
        ...defaultProps,
        current: {
          context: {
            data: {testField: ''},
            pristineData: {testField: ''}
          }
        }
      };

      render(<Field {...props} />);

      const input = screen.getByRole('textbox');
      expect(input).toHaveValue('');
    });
  });
});
