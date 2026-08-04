

import React from 'react';
import {render, screen} from '@testing-library/react';
import FieldWrapper from './FieldWrapper';

describe('FieldWrapper', () => {
  it('renders with label and description', () => {
    render(
      <FieldWrapper labelText="Field Label" descriptionText="Field description">
        <input id="test-field" type="text" />
      </FieldWrapper>
    );

    expect(screen.getByText('Field Label')).toBeInTheDocument();
    expect(screen.getByText('Field description')).toBeInTheDocument();
    expect(screen.getByRole('textbox')).toBeInTheDocument();
  });

  it('renders with label only', () => {
    render(
      <FieldWrapper labelText="Field Label">
        <input id="test-field" type="text" />
      </FieldWrapper>
    );

    expect(screen.getByText('Field Label')).toBeInTheDocument();
    expect(screen.queryByText('Field description')).not.toBeInTheDocument();
  });

  it('renders without label when not provided', () => {
    render(
      <FieldWrapper>
        <input id="test-field" type="text" />
      </FieldWrapper>
    );

    expect(screen.queryByRole('label')).not.toBeInTheDocument();
    expect(screen.getByRole('textbox')).toBeInTheDocument();
  });

  it('applies optional class when isOptional is true', () => {
    const {container} = render(
      <FieldWrapper labelText="Optional Field" isOptional={true}>
        <input id="test-field" type="text" />
      </FieldWrapper>
    );

    const label = container.querySelector('label');
    expect(label).toHaveClass('nx-label');
    expect(label).toHaveClass('nx-label--optional');
  });

  it('does not apply optional class when isOptional is false', () => {
    const {container} = render(
      <FieldWrapper labelText="Required Field" isOptional={false}>
        <input id="test-field" type="text" />
      </FieldWrapper>
    );

    const label = container.querySelector('label');
    expect(label).toHaveClass('nx-label');
    expect(label).not.toHaveClass('nx-label--optional');
  });

  it('associates label with field using htmlFor and field id', () => {
    render(
      <FieldWrapper labelText="Field Label">
        <input id="test-field" type="text" />
      </FieldWrapper>
    );

    const label = screen.getByText('Field Label').closest('label');
    expect(label).toHaveAttribute('for', 'test-field');
  });

  it('uses field name when id is not provided', () => {
    render(
      <FieldWrapper labelText="Field Label">
        <input name="test-name" type="text" />
      </FieldWrapper>
    );

    const label = screen.getByText('Field Label').closest('label');
    expect(label).toHaveAttribute('for', 'test-name');
  });

  it('applies custom id to form group', () => {
    const {container} = render(
      <FieldWrapper labelText="Field Label" id="custom-group-id">
        <input id="test-field" type="text" />
      </FieldWrapper>
    );

    const formGroup = container.querySelector('.nx-form-group');
    expect(formGroup).toHaveAttribute('id', 'custom-group-id');
  });

  it('renders description with correct class', () => {
    const {container} = render(
      <FieldWrapper labelText="Field Label" descriptionText="Test description">
        <input id="test-field" type="text" />
      </FieldWrapper>
    );

    const description = container.querySelector('p');
    expect(description).toHaveClass('nx-p');
    expect(description).toHaveTextContent('Test description');
  });

  it('renders children correctly', () => {
    render(
      <FieldWrapper labelText="Field Label">
        <input id="test-field" type="text" placeholder="Enter value" />
      </FieldWrapper>
    );

    const input = screen.getByPlaceholderText('Enter value');
    expect(input).toBeInTheDocument();
    expect(input).toHaveAttribute('id', 'test-field');
  });

  it('handles multiple children', () => {
    render(
      <FieldWrapper labelText="Field Label">
        <input id="field1" type="text" />
        <input id="field2" type="text" />
      </FieldWrapper>
    );

    // Should use the first child's id for the label
    const label = screen.getByText('Field Label').closest('label');
    expect(label).toHaveAttribute('for', 'field1');

    // Both inputs should be rendered
    const inputs = screen.getAllByRole('textbox');
    expect(inputs).toHaveLength(2);
  });

  it('applies nx-form-group class to wrapper', () => {
    const {container} = render(
      <FieldWrapper labelText="Field Label">
        <input id="test-field" type="text" />
      </FieldWrapper>
    );

    const formGroup = container.querySelector('.nx-form-group');
    expect(formGroup).toBeInTheDocument();
  });

  it('wraps label text in nx-label__text span', () => {
    const {container} = render(
      <FieldWrapper labelText="Field Label">
        <input id="test-field" type="text" />
      </FieldWrapper>
    );

    const labelTextSpan = container.querySelector('.nx-label__text');
    expect(labelTextSpan).toBeInTheDocument();
    expect(labelTextSpan).toHaveTextContent('Field Label');
  });
});
