

import React from 'react';
import { render, screen, fireEvent, waitFor, act } from '@testing-library/react';
import '@testing-library/jest-dom';
import { GASearchInput } from '../GASearchInput';

jest.mock('../mockData', () => ({
  ...jest.requireActual('../mockData'),
  mockSuggestApi: jest.fn(),
}));

import { mockSuggestApi } from '../mockData';
const mockSuggest = mockSuggestApi as jest.Mock;

beforeEach(() => {
  jest.clearAllMocks();
  jest.useFakeTimers();
  mockSuggest.mockResolvedValue({ suggestions: [] });
});

afterEach(() => {
  jest.runOnlyPendingTimers();
  jest.useRealTimers();
});

describe('GASearchInput', () => {
  it('renders with placeholder', () => {
    render(
      <GASearchInput
        value=""
        onChange={jest.fn()}
        onSearch={jest.fn()}
        placeholder="Search Maven artifacts..."
      />
    );

    expect(screen.getByRole('combobox', { name: /search/i })).toBeInTheDocument();
  });

  it('calls onChange when typing', () => {
    const onChange = jest.fn();
    render(<GASearchInput value="" onChange={onChange} onSearch={jest.fn()} />);

    const input = screen.getByRole('combobox');
    fireEvent.change(input, { target: { value: 'commons' } });

    expect(onChange).toHaveBeenCalledWith('commons');
  });

  it('calls onSearch when Enter pressed with no suggestions', () => {
    const onSearch = jest.fn();
    render(<GASearchInput value="commons" onChange={jest.fn()} onSearch={onSearch} />);

    const input = screen.getByRole('combobox');
    fireEvent.keyDown(input, { key: 'Enter' });

    expect(onSearch).toHaveBeenCalledWith('commons');
  });

  it('does not fetch suggestions when value length < 2', () => {
    render(<GASearchInput value="c" onChange={jest.fn()} onSearch={jest.fn()} />);

    act(() => {
      jest.runAllTimers();
    });

    expect(mockSuggest).not.toHaveBeenCalled();
  });

  it('fetches suggestions when value length >= 2', async () => {
    const suggestions = [
      { gaId: 'maven:org.apache.commons:commons-lang3', displayText: 'commons-lang3', highlights: [] },
    ];
    mockSuggest.mockResolvedValue({ suggestions });

    render(<GASearchInput value="co" onChange={jest.fn()} onSearch={jest.fn()} />);

    await act(async () => {
      jest.runAllTimers();
      await Promise.resolve();
    });

    await waitFor(() => {
      expect(screen.getByText('commons-lang3')).toBeInTheDocument();
    });
  });

  it('calls onSuggestionSelect when suggestion clicked', async () => {
    const suggestions = [
      { gaId: 'maven:org.apache:commons-lang3', displayText: 'commons-lang3', highlights: [] },
    ];
    mockSuggest.mockResolvedValue({ suggestions });
    const onSuggestionSelect = jest.fn();

    render(
      <GASearchInput
        value="co"
        onChange={jest.fn()}
        onSearch={jest.fn()}
        onSuggestionSelect={onSuggestionSelect}
      />
    );

    await act(async () => {
      jest.runAllTimers();
      await Promise.resolve();
    });

    await waitFor(() => screen.getByText('commons-lang3'));
    fireEvent.click(screen.getByText('commons-lang3'));

    expect(onSuggestionSelect).toHaveBeenCalledWith('maven:org.apache:commons-lang3');
  });

  it('calls onChange and onSearch when suggestion clicked without onSuggestionSelect', async () => {
    const suggestions = [
      { gaId: 'maven:org.apache:commons-lang3', displayText: 'commons-lang3', highlights: [] },
    ];
    mockSuggest.mockResolvedValue({ suggestions });
    const onChange = jest.fn();
    const onSearch = jest.fn();

    render(
      <GASearchInput value="co" onChange={onChange} onSearch={onSearch} />
    );

    await act(async () => {
      jest.runAllTimers();
      await Promise.resolve();
    });

    await waitFor(() => screen.getByText('commons-lang3'));
    fireEvent.click(screen.getByText('commons-lang3'));

    expect(onChange).toHaveBeenCalledWith('commons-lang3');
    expect(onSearch).toHaveBeenCalledWith('commons-lang3');
  });

  it('is disabled when disabled prop is true', () => {
    render(
      <GASearchInput value="" onChange={jest.fn()} onSearch={jest.fn()} disabled />
    );

    expect(screen.getByRole('combobox')).toBeDisabled();
  });
});
