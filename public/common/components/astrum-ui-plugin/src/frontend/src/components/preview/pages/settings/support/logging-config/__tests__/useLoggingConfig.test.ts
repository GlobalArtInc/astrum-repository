import { renderHook, act } from '@testing-library/react';
import { useLoggingConfig } from '../useLoggingConfig';

const mockGo = jest.fn();
let mockRoute: { name: string; params: Record<string, string | undefined> } = {
  name: 'preview.admin.support.logging.list',
  params: {},
};

jest.mock('@uirouter/react', () => ({
  useRouter: () => ({ stateService: { go: mockGo } }),
  useCurrentStateAndParams: () => ({ state: { name: mockRoute.name }, params: mockRoute.params }),
}));

jest.mock('../../../../../shared', () => ({
  useToast: () => ({ success: jest.fn(), error: jest.fn(), info: jest.fn(), warning: jest.fn() }),
}));

jest.mock('../../../../../../../interface/api', () => ({
  restClient: { delete: jest.fn().mockResolvedValue(undefined), post: jest.fn().mockResolvedValue(undefined) },
  parseApiError: (e: Error) => ({ message: e.message }),
}));

describe('useLoggingConfig', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockRoute = { name: 'preview.admin.support.logging.list', params: {} };
  });

  it('opens the create form for the create route', () => {
    mockRoute = { name: 'preview.admin.support.logging.create', params: {} };

    const { result } = renderHook(() => useLoggingConfig());

    expect(result.current.viewMode).toBe('create');
  });

  it('opens a logger for its route', () => {
    mockRoute = { name: 'preview.admin.support.logging.detail', params: { itemId: 'org.globalart.astrum' } };

    const { result } = renderHook(() => useLoggingConfig());

    expect(result.current.viewMode).toBe('detail');
    expect(result.current.selectedLogger).toBe('org.globalart.astrum');
  });

  it('shows the list for the list route', () => {
    const { result } = renderHook(() => useLoggingConfig());

    expect(result.current.viewMode).toBe('list');
    expect(mockGo).not.toHaveBeenCalled();
  });

  it('goes to the route of the view the user opens', () => {
    const { result } = renderHook(() => useLoggingConfig());

    act(() => result.current.handleSelectLogger('org.eclipse.jetty'));
    expect(result.current.viewMode).toBe('detail');
    expect(mockGo).toHaveBeenLastCalledWith('preview.admin.support.logging.detail', { itemId: 'org.eclipse.jetty' });

    act(() => result.current.handleBack());
    expect(result.current.viewMode).toBe('list');
    expect(mockGo).toHaveBeenLastCalledWith('preview.admin.support.logging.list');

    act(() => result.current.handleCreate());
    expect(mockGo).toHaveBeenLastCalledWith('preview.admin.support.logging.create');

    act(() => result.current.handleSave());
    expect(mockGo).toHaveBeenLastCalledWith('preview.admin.support.logging.list');
  });

  it('returns to the list when the route goes back to it', () => {
    mockRoute = { name: 'preview.admin.support.logging.detail', params: { itemId: 'org.globalart.astrum' } };
    const { result, rerender } = renderHook(() => useLoggingConfig());
    expect(result.current.viewMode).toBe('detail');

    mockRoute = { name: 'preview.admin.support.logging.list', params: {} };
    rerender();

    expect(result.current.viewMode).toBe('list');
  });
});
