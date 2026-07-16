

import React from 'react';
import { render } from '@testing-library/react';
import SearchComposerExt from './SearchComposerExt';

const mockSearchFeatureExt = jest.fn(() => <div data-testid="search-feature-ext" />);

jest.mock('./SearchFeatureExt', () => ({
  __esModule: true,
  default: (props) => mockSearchFeatureExt(props),
}));

describe('SearchComposerExt', () => {
  beforeEach(() => {
    mockSearchFeatureExt.mockClear();
  });

  it('renders without crashing', () => {
    const { getByTestId } = render(<SearchComposerExt />);
    expect(getByTestId('search-feature-ext')).toBeInTheDocument();
  });

  it('passes the composer format filter to SearchFeatureExt', () => {
    render(<SearchComposerExt />);
    const [props] = mockSearchFeatureExt.mock.calls[0];
    expect(props.filter.id).toBe('composer');
  });

  it('includes hidden format criteria with value composer', () => {
    render(<SearchComposerExt />);
    const [props] = mockSearchFeatureExt.mock.calls[0];
    const formatCriteria = props.filter.criterias.find((c) => c.id === 'format');
    expect(formatCriteria).toBeDefined();
    expect(formatCriteria.value).toBe('composer');
    expect(formatCriteria.hidden).toBe(true);
  });

  it('references the generic version criterion in the filter (not composer-specific)', () => {
    render(<SearchComposerExt />);
    const [props] = mockSearchFeatureExt.mock.calls[0];
    const ids = props.filter.criterias.map((c) => c.id);
    expect(ids).toContain('version');
    expect(ids).not.toContain('assets.attributes.composer.version');
  });

  it('does not re-register the generic version as a composer-specific criterion', () => {
    render(<SearchComposerExt />);
    const [props] = mockSearchFeatureExt.mock.calls[0];
    const ids = (props.criterias || []).map((c) => c.id);
    expect(ids).not.toContain('assets.attributes.composer.version');
  });

  it('sets the filter as readOnly', () => {
    render(<SearchComposerExt />);
    const [props] = mockSearchFeatureExt.mock.calls[0];
    expect(props.filter.readOnly).toBe(true);
  });

  it('includes composer description and keywords in the default filter', () => {
    render(<SearchComposerExt />);
    const [props] = mockSearchFeatureExt.mock.calls[0];
    const ids = props.filter.criterias.map((c) => c.id);
    expect(ids).toContain('composer.description');
    expect(ids).toContain('composer.keywords');
  });

  it('registers composer description and keywords as composer-specific criteria', () => {
    render(<SearchComposerExt />);
    const [props] = mockSearchFeatureExt.mock.calls[0];
    const ids = (props.criterias || []).map((c) => c.id);
    expect(ids).toContain('composer.description');
    expect(ids).toContain('composer.keywords');
  });
});
