

import React from 'react';
import { render, screen } from '@testing-library/react';

import Welcome from '../Welcome';
import welcomeLogoUrl from '../../../art/logos/astrum-mark.svg';
import { useWizard } from '../useWizard';

jest.mock('../useWizard');
const mockedUseWizard = jest.mocked(useWizard);

describe('Welcome', () => {
  const registerStep = jest.fn();
  const getStarted = jest.fn();

  beforeEach(() => {
    jest.clearAllMocks();
    mockedUseWizard.mockReturnValue({
      state: 'stepReady',
      steps: [],
      currentIndex: -1,
      currentStep: null,
      errorMessage: null,
      isCurrentStepValid: true,
      registerStep,
      getStarted,
      submit: jest.fn(),
      skip: jest.fn(),
      finish: jest.fn(),
      actionButtonRef: { current: null },
    });
  });

  it('renders the logo with alt text', () => {
    render(<Welcome />);
    const logo = screen.getByAltText('GlobalArt Astrum Repository');
    expect(logo).toBeInTheDocument();
    expect(logo.tagName).toBe('IMG');
    expect(logo).toHaveAttribute('src', welcomeLogoUrl);
  });

  it('renders the title text', () => {
    render(<Welcome />);
    expect(screen.getByText(/Let's Get You Set Up/i)).toBeInTheDocument();
  });

  it('renders the description text', () => {
    render(<Welcome />);
    expect(
      screen.getByText(/This wizard will guide you through a few quick steps/i)
    ).toBeInTheDocument();
  });

  it('registers a valid step on mount', () => {
    render(<Welcome />);
    expect(registerStep).toHaveBeenCalledWith({
      valid: true,
      onSubmit: expect.any(Function),
    });
  });

  it('calls getStarted when onSubmit is invoked', () => {
    render(<Welcome />);
    const registration = registerStep.mock.calls[0][0];
    registration.onSubmit();
    expect(getStarted).toHaveBeenCalledTimes(1);
  });

  it('does not render a button', () => {
    render(<Welcome />);
    expect(screen.queryByRole('button')).not.toBeInTheDocument();
  });
});
