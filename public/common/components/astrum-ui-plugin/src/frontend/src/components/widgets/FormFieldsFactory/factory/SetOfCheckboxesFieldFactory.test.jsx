
import React from 'react';
import {render, screen} from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import SetOfCheckboxesFieldFactory from './SetOfCheckboxesFieldFactory';
import {SUPPORTED_FIELD_TYPES} from '../FormFieldsFactoryConstants';

describe('SetOfCheckboxesFieldFactory', () => {
  describe('metadata', () => {
    it('exports supported field types', () => {
      expect(SetOfCheckboxesFieldFactory.types).toBe(SUPPORTED_FIELD_TYPES.SET_OF_CHECKBOXES);
    });

    it('exports a component', () => {
      expect(SetOfCheckboxesFieldFactory.component).toBeDefined();
      expect(typeof SetOfCheckboxesFieldFactory.component).toBe('function');
    });
  });

  describe('Field component', () => {
    const Field = SetOfCheckboxesFieldFactory.component;
    const mockOnChange = jest.fn();

    const defaultProps = {
      id: 'permissions',
      dynamicProps: {
        attributes: {
          options: ['read', 'write', 'delete']
        }
      },
      current: {
        context: {
          data: {permissions: {read: true, write: false, delete: true}},
          pristineData: {permissions: {}}
        }
      },
      onChange: mockOnChange
    };

    beforeEach(() => {
      mockOnChange.mockClear();
    });

    it('renders all checkbox options', () => {
      render(<Field {...defaultProps} />);

      expect(screen.getByLabelText('Read')).toBeInTheDocument();
      expect(screen.getByLabelText('Write')).toBeInTheDocument();
      expect(screen.getByLabelText('Delete')).toBeInTheDocument();
    });

    it('capitalizes checkbox labels', () => {
      render(<Field {...defaultProps} />);

      expect(screen.getByText('Read')).toBeInTheDocument();
      expect(screen.getByText('Write')).toBeInTheDocument();
      expect(screen.getByText('Delete')).toBeInTheDocument();
    });

    it('checks boxes based on current data', () => {
      render(<Field {...defaultProps} />);

      expect(screen.getByLabelText('Read')).toBeChecked();
      expect(screen.getByLabelText('Write')).not.toBeChecked();
      expect(screen.getByLabelText('Delete')).toBeChecked();
    });

    it('calls onChange when checkbox is clicked', async () => {
      render(<Field {...defaultProps} />);

      const writeCheckbox = screen.getByLabelText('Write');
      await userEvent.click(writeCheckbox);

      expect(mockOnChange).toHaveBeenCalledWith(
        'permissions',
        expect.objectContaining({write: true})
      );
    });

    it('merges checkbox changes with existing data', async () => {
      render(<Field {...defaultProps} />);

      const writeCheckbox = screen.getByLabelText('Write');
      await userEvent.click(writeCheckbox);

      expect(mockOnChange).toHaveBeenCalledWith(
        'permissions',
        expect.objectContaining({
          read: true,
          write: true,
          delete: true
        })
      );
    });

    it('renders read-only value as comma-separated capitalized text', () => {
      const props = {
        ...defaultProps,
        dynamicProps: {
          ...defaultProps.dynamicProps,
          readOnly: true
        }
      };

      const {container} = render(<Field {...props} />);

      expect(screen.queryByRole('checkbox')).not.toBeInTheDocument();
      // Displays all keys from the permissions object as capitalized labels
      expect(container.textContent).toBe('Read, Write, Delete');
    });

    it('handles empty permissions object in read-only mode', () => {
      const props = {
        ...defaultProps,
        dynamicProps: {
          ...defaultProps.dynamicProps,
          readOnly: true
        },
        current: {
          context: {
            data: {permissions: {}},
            pristineData: {permissions: {}}
          }
        }
      };

      const {container} = render(<Field {...props} />);

      expect(container.textContent).toBe('');
    });

    it('handles single option', () => {
      const props = {
        ...defaultProps,
        dynamicProps: {
          attributes: {
            options: ['admin']
          }
        },
        current: {
          context: {
            data: {permissions: {admin: true}},
            pristineData: {permissions: {}}
          }
        }
      };

      render(<Field {...props} />);

      expect(screen.getByLabelText('Admin')).toBeInTheDocument();
      expect(screen.getByLabelText('Admin')).toBeChecked();
    });
  });
});
