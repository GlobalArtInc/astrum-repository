

import React from 'react';
import { render, screen, fireEvent, waitFor, act } from '@testing-library/react';
import { Theme } from '@radix-ui/themes';

import { LogViewer } from '../LogViewer';
import * as useLogViewerModule from '../useLogViewer';

// Mock the integration hook
jest.mock('../useLogViewer');

// Mock ExtJS
jest.mock('../../../../../../../interface/ExtJS', () => ({
  ExtJS: {
    urlOf: jest.fn((url) => `http://localhost:8081/${url}`),
    downloadUrl: jest.fn(),
    checkPermission: jest.fn().mockReturnValue(true),
  },
}));

const mockedUseLogViewer = useLogViewerModule.useLogViewer as jest.MockedFunction<typeof useLogViewerModule.useLogViewer>;

// Wrapper component for Radix Theme
function TestWrapper({ children }: { children: React.ReactNode }) {
  return <Theme>{children}</Theme>;
}

const mockLogContent = `2024-01-01 12:00:00 INFO  [main] - Starting application
2024-01-01 12:00:01 INFO  [main] - Loading configuration
2024-01-01 12:00:02 INFO  [main] - Application started successfully`;

function makeHook(overrides: Partial<ReturnType<typeof useLogViewerModule.useLogViewer>> = {}) {
  return {
    logContent: mockLogContent,
    isLoading: false,
    error: null,
    mark: '',
    refreshPeriod: 0,
    logSize: 25,
    setMark: jest.fn(),
    setRefreshPeriod: jest.fn(),
    setLogSize: jest.fn(),
    handleInsertMark: jest.fn(),
    handleDownload: jest.fn(),
    textareaRef: { current: null } as React.RefObject<HTMLTextAreaElement>,
    ...overrides,
  };
}

describe('LogViewer', () => {
  const mockOnBack = jest.fn();

  beforeEach(() => {
    jest.clearAllMocks();
    mockedUseLogViewer.mockReturnValue(makeHook());
  });

  it('renders log viewer with filename in title', async () => {
    render(<LogViewer filename="astrum.log" onBack={mockOnBack} />, { wrapper: TestWrapper });

    await waitFor(() => {
      expect(screen.getByText('Viewing astrum.log')).toBeInTheDocument();
    });
  });

  it('displays log content in textarea', async () => {
    render(<LogViewer filename="astrum.log" onBack={mockOnBack} />, { wrapper: TestWrapper });

    await waitFor(() => {
      const textarea = screen.getByLabelText('Log content for astrum.log');
      expect(textarea).toHaveValue(mockLogContent);
    });
  });

  it('shows loading state while fetching content', () => {
    mockedUseLogViewer.mockReturnValue(makeHook({ isLoading: true, logContent: '' }));

    render(<LogViewer filename="astrum.log" onBack={mockOnBack} />, { wrapper: TestWrapper });

    expect(screen.getByText('Loading log content...')).toBeInTheDocument();
  });

  it('has download button', async () => {
    render(<LogViewer filename="astrum.log" onBack={mockOnBack} />, { wrapper: TestWrapper });

    await waitFor(() => {
      expect(screen.getByText('Download')).toBeInTheDocument();
    });
  });

  it('calls handleDownload when download button is clicked', async () => {
    const mockHandleDownload = jest.fn();
    mockedUseLogViewer.mockReturnValue(makeHook({ handleDownload: mockHandleDownload }));

    render(<LogViewer filename="astrum.log" onBack={mockOnBack} />, { wrapper: TestWrapper });

    await waitFor(() => {
      expect(screen.getByText('Download')).toBeInTheDocument();
    });

    fireEvent.click(screen.getByText('Download'));

    expect(mockHandleDownload).toHaveBeenCalled();
  });

  it('shows mark input for astrum.log', async () => {
    render(<LogViewer filename="astrum.log" onBack={mockOnBack} />, { wrapper: TestWrapper });

    await waitFor(() => {
      expect(screen.getByText('Marker to insert:')).toBeInTheDocument();
      expect(screen.getByPlaceholderText('MARK')).toBeInTheDocument();
      expect(screen.getByText('Insert')).toBeInTheDocument();
    });
  });

  it('wraps the marker and the options when the toolbar is narrow', async () => {
    const { container } = render(<LogViewer filename="astrum.log" onBack={mockOnBack} />, { wrapper: TestWrapper });

    await waitFor(() => expect(screen.getByText('Marker to insert:')).toBeInTheDocument());

    expect(container.querySelector('.log-viewer__mark-section')).toHaveClass('rt-r-fw-wrap');
    expect(container.querySelector('.log-viewer__options')).toHaveClass('rt-r-fw-wrap');
  });

  it('does not show mark input for other log files', async () => {
    render(<LogViewer filename="request.log" onBack={mockOnBack} />, { wrapper: TestWrapper });

    await waitFor(() => {
      expect(screen.getByText('Viewing request.log')).toBeInTheDocument();
    });

    expect(screen.queryByText('Marker to insert:')).not.toBeInTheDocument();
  });

  it('calls setMark when mark input changes', async () => {
    const mockSetMark = jest.fn();
    mockedUseLogViewer.mockReturnValue(makeHook({ setMark: mockSetMark }));

    render(<LogViewer filename="astrum.log" onBack={mockOnBack} />, { wrapper: TestWrapper });

    await waitFor(() => {
      expect(screen.getByPlaceholderText('MARK')).toBeInTheDocument();
    });

    const markInput = screen.getByPlaceholderText('MARK');
    fireEvent.change(markInput, { target: { value: 'TEST_MARK' } });

    expect(mockSetMark).toHaveBeenCalledWith('TEST_MARK', expect.anything());
  });

  it('calls handleInsertMark when Insert button is clicked', async () => {
    const mockHandleInsertMark = jest.fn();
    mockedUseLogViewer.mockReturnValue(makeHook({ mark: 'TEST_MARK', handleInsertMark: mockHandleInsertMark }));

    render(<LogViewer filename="astrum.log" onBack={mockOnBack} />, { wrapper: TestWrapper });

    await waitFor(() => {
      expect(screen.getByText('Insert')).toBeInTheDocument();
    });

    await act(async () => {
      fireEvent.click(screen.getByText('Insert'));
    });

    expect(mockHandleInsertMark).toHaveBeenCalled();
  });

  it('calls handleInsertMark when Enter is pressed in mark input', async () => {
    const mockHandleInsertMark = jest.fn();
    mockedUseLogViewer.mockReturnValue(makeHook({ mark: 'TEST_MARK', handleInsertMark: mockHandleInsertMark }));

    render(<LogViewer filename="astrum.log" onBack={mockOnBack} />, { wrapper: TestWrapper });

    await waitFor(() => {
      expect(screen.getByPlaceholderText('MARK')).toBeInTheDocument();
    });

    const markInput = screen.getByPlaceholderText('MARK');
    await act(async () => {
      fireEvent.keyDown(markInput, { key: 'Enter' });
    });

    expect(mockHandleInsertMark).toHaveBeenCalled();
  });

  it('has refresh rate selector', async () => {
    render(<LogViewer filename="astrum.log" onBack={mockOnBack} />, { wrapper: TestWrapper });

    await waitFor(() => {
      expect(screen.getByText('Refresh Rate:')).toBeInTheDocument();
    });
  });

  it('has log size selector', async () => {
    render(<LogViewer filename="astrum.log" onBack={mockOnBack} />, { wrapper: TestWrapper });

    await waitFor(() => {
      expect(screen.getByText('Size:')).toBeInTheDocument();
    });
  });

  it('has size selector available', async () => {
    render(<LogViewer filename="astrum.log" onBack={mockOnBack} />, { wrapper: TestWrapper });

    await waitFor(() => {
      expect(screen.getByText('Size:')).toBeInTheDocument();
    });

    const comboboxes = screen.getAllByRole('combobox');
    expect(comboboxes.length).toBeGreaterThan(0);
  });

  it('displays error alert when error is present', async () => {
    mockedUseLogViewer.mockReturnValue(makeHook({ error: 'Failed to load log content', logContent: '' }));

    render(<LogViewer filename="astrum.log" onBack={mockOnBack} />, { wrapper: TestWrapper });

    await waitFor(() => {
      expect(screen.getByText('Failed to load log content')).toBeInTheDocument();
    });
  });

  it('displays raw filenames including spaces and special characters', async () => {
    render(<LogViewer filename="test log.log" onBack={mockOnBack} />, { wrapper: TestWrapper });

    await waitFor(() => {
      expect(screen.getByText('Viewing test log.log')).toBeInTheDocument();
    });
  });

  it('has refresh rate selector available', async () => {
    render(<LogViewer filename="astrum.log" onBack={mockOnBack} />, { wrapper: TestWrapper });

    await waitFor(() => {
      expect(screen.getByText('Refresh Rate:')).toBeInTheDocument();
    });

    const comboboxes = screen.getAllByRole('combobox');
    expect(comboboxes.length).toBeGreaterThan(0);
  });
});
