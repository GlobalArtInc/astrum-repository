

import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { Theme } from '@radix-ui/themes';

import { EmailVerify } from '../EmailVerify';

// Wrapper component for Radix Theme
function TestWrapper({ children }: { children: React.ReactNode }) {
  return <Theme>{children}</Theme>;
}

describe('EmailVerify', () => {
  const mockOnSendTest = jest.fn();

  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('renders the test email form', () => {
    render(<EmailVerify onSendTest={mockOnSendTest} />, { wrapper: TestWrapper });

    expect(screen.getByLabelText('Test Email Address')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /Send Test/i })).toBeInTheDocument();
  });

  it('disables send button when email is empty', () => {
    render(<EmailVerify onSendTest={mockOnSendTest} />, { wrapper: TestWrapper });

    const sendButton = screen.getByRole('button', { name: /Send Test/i });
    expect(sendButton).toBeDisabled();
  });

  it('disables send button when email is invalid', () => {
    render(<EmailVerify onSendTest={mockOnSendTest} />, { wrapper: TestWrapper });

    const emailInput = screen.getByLabelText('Test Email Address');
    fireEvent.change(emailInput, { target: { value: 'invalid-email' } });

    const sendButton = screen.getByRole('button', { name: /Send Test/i });
    expect(sendButton).toBeDisabled();
  });

  it('enables send button when valid email is entered', () => {
    render(<EmailVerify onSendTest={mockOnSendTest} />, { wrapper: TestWrapper });

    const emailInput = screen.getByLabelText('Test Email Address');
    fireEvent.change(emailInput, { target: { value: 'test@example.com' } });

    const sendButton = screen.getByRole('button', { name: /Send Test/i });
    expect(sendButton).not.toBeDisabled();
  });

  it('calls onSendTest with email when button is clicked', async () => {
    mockOnSendTest.mockResolvedValue({ success: true });

    render(<EmailVerify onSendTest={mockOnSendTest} />, { wrapper: TestWrapper });

    const emailInput = screen.getByLabelText('Test Email Address');
    fireEvent.change(emailInput, { target: { value: 'test@example.com' } });

    const sendButton = screen.getByRole('button', { name: /Send Test/i });
    fireEvent.click(sendButton);

    await waitFor(() => {
      expect(mockOnSendTest).toHaveBeenCalledWith('test@example.com');
    });
  });

  it('displays success message when test succeeds', async () => {
    mockOnSendTest.mockResolvedValue({ success: true });

    render(<EmailVerify onSendTest={mockOnSendTest} />, { wrapper: TestWrapper });

    const emailInput = screen.getByLabelText('Test Email Address');
    fireEvent.change(emailInput, { target: { value: 'test@example.com' } });

    const sendButton = screen.getByRole('button', { name: /Send Test/i });
    fireEvent.click(sendButton);

    await waitFor(() => {
      expect(screen.getByText(/Test email sent successfully/)).toBeInTheDocument();
    });
  });

  it('displays error message when test fails', async () => {
    mockOnSendTest.mockResolvedValue({ success: false, reason: 'Connection refused' });

    render(<EmailVerify onSendTest={mockOnSendTest} />, { wrapper: TestWrapper });

    const emailInput = screen.getByLabelText('Test Email Address');
    fireEvent.change(emailInput, { target: { value: 'test@example.com' } });

    const sendButton = screen.getByRole('button', { name: /Send Test/i });
    fireEvent.click(sendButton);

    await waitFor(() => {
      expect(screen.getByText(/Connection refused/)).toBeInTheDocument();
    });
  });

  it('disables button when disabled prop is true', () => {
    render(<EmailVerify onSendTest={mockOnSendTest} disabled />, { wrapper: TestWrapper });

    const emailInput = screen.getByLabelText('Test Email Address');
    expect(emailInput).toBeDisabled();

    const sendButton = screen.getByRole('button', { name: /Send Test/i });
    expect(sendButton).toBeDisabled();
  });

  it('shows loading state when loading prop is true', () => {
    render(<EmailVerify onSendTest={mockOnSendTest} loading />, { wrapper: TestWrapper });

    const sendButton = screen.getByRole('button', { name: /Send Test/i });
    expect(sendButton).toBeDisabled();
  });

  it('clears previous results when email changes', async () => {
    mockOnSendTest.mockResolvedValue({ success: true });

    render(<EmailVerify onSendTest={mockOnSendTest} />, { wrapper: TestWrapper });

    const emailInput = screen.getByLabelText('Test Email Address');
    fireEvent.change(emailInput, { target: { value: 'test@example.com' } });

    const sendButton = screen.getByRole('button', { name: /Send Test/i });
    fireEvent.click(sendButton);

    await waitFor(() => {
      expect(screen.getByText(/Test email sent successfully/)).toBeInTheDocument();
    });

    // Change email
    fireEvent.change(emailInput, { target: { value: 'other@example.com' } });

    // Success message should be gone
    expect(screen.queryByText(/Test email sent successfully/)).not.toBeInTheDocument();
  });

  it('sends test on Enter key press', async () => {
    mockOnSendTest.mockResolvedValue({ success: true });

    render(<EmailVerify onSendTest={mockOnSendTest} />, { wrapper: TestWrapper });

    const emailInput = screen.getByLabelText('Test Email Address');
    fireEvent.change(emailInput, { target: { value: 'test@example.com' } });
    fireEvent.keyDown(emailInput, { key: 'Enter' });

    await waitFor(() => {
      expect(mockOnSendTest).toHaveBeenCalledWith('test@example.com');
    });
  });

  it('has as-email-test analytics ID on the Send Test button', () => {
    render(
      <Theme>
        <EmailVerify onSendTest={jest.fn()} />
      </Theme>
    );

    const sendButton = screen.getByRole('button', { name: /send test/i });
    expect(sendButton).toHaveAttribute('data-analytics-id', 'as-email-test');
  });
});


