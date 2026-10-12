import { renderHook } from '@testing-library/react';

const mockIsGlobalArtDevMode = jest.fn(() => false);
const mockIsProEdition = jest.fn(() => true);
const mockUseDefaultAdminRouteName = jest.fn<string | null, []>(() => 'preview.admin.repository');
const mockUseSideNavbarOpenState = jest.fn(() => [true, jest.fn()]);
const mockUseStatus = jest.fn<{ version?: string } | undefined, []>(() => ({ version: '3.96.4' }));

jest.mock('@globalart/astrum-ui-plugin', () => ({
  ...jest.requireActual('@globalart/astrum-ui-plugin'),
  isGlobalArtDevMode: (...args: unknown[]) => mockIsGlobalArtDevMode(...args),
  useSideNavbarOpenState: (...args: unknown[]) => mockUseSideNavbarOpenState(...args),
  ExtJS: {
    isProEdition: (...args: unknown[]) => mockIsProEdition(...args),
    useStatus: () => mockUseStatus(),
  },
}));

// The menu drawer closes after every navigation; the hook only registers for it here.
jest.mock('@uirouter/react', () => ({
  useTransitionHook: jest.fn(),
}));

jest.mock('./useDefaultAdminRouteName', () => ({
  useDefaultAdminRouteName: () => mockUseDefaultAdminRouteName(),
}));

import { useLeftNavigationMenu } from './useLeftNavigationMenu';

describe('useLeftNavigationMenu', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockIsGlobalArtDevMode.mockReturnValue(false);
    mockIsProEdition.mockReturnValue(true);
    mockUseDefaultAdminRouteName.mockReturnValue('preview.admin.repository');
    // useSideNavbarOpenState returns [isOpen, toggle]; open (true) => not collapsed.
    mockUseSideNavbarOpenState.mockReturnValue([true, jest.fn()]);
    mockUseStatus.mockReturnValue({ version: '3.96.4' });
  });

  it('opens the menu expanded until the user chose otherwise', () => {
    renderHook(() => useLeftNavigationMenu());

    expect(mockUseSideNavbarOpenState).toHaveBeenCalledWith(true);
  });

  it('reports the version from the server status', () => {
    const { result } = renderHook(() => useLeftNavigationMenu());

    expect(result.current.version).toBe('3.96.4');
  });

  it('reports no version while the status is not known', () => {
    mockUseStatus.mockReturnValue(undefined);

    const { result } = renderHook(() => useLeftNavigationMenu());

    expect(result.current.version).toBeNull();
  });

  it('returns the collapsed state and a visible settings entry', () => {
    const { result } = renderHook(() => useLeftNavigationMenu());

    expect(result.current.isCollapsed).toBe(false);
    expect(result.current.isProEdition).toBe(true);
    expect(result.current.showTestHub).toBe(false);
    expect(result.current.adminInitialRouteName).toBe('preview.admin.repository');
    expect(result.current.isSettingsVisible).toBe(true);
  });

  it('reflects the collapsed state from useSideNavbarOpenState', () => {
    // isOpen === false => collapsed.
    mockUseSideNavbarOpenState.mockReturnValue([false, jest.fn()]);

    const { result } = renderHook(() => useLeftNavigationMenu());

    expect(result.current.isCollapsed).toBe(true);
  });

  it('reports CE mode (isProEdition false) when ExtJS is not Pro', () => {
    mockIsProEdition.mockReturnValue(false);

    const { result } = renderHook(() => useLeftNavigationMenu());

    expect(result.current.isProEdition).toBe(false);
  });

  it('shows the Test Hub flag when isGlobalArtDevMode is true', () => {
    mockIsGlobalArtDevMode.mockReturnValue(true);

    const { result } = renderHook(() => useLeftNavigationMenu());

    expect(result.current.showTestHub).toBe(true);
  });

  it('hides the settings entry when no admin route is visible', () => {
    mockUseDefaultAdminRouteName.mockReturnValue(null);

    const { result } = renderHook(() => useLeftNavigationMenu());

    expect(result.current.adminInitialRouteName).toBeNull();
    expect(result.current.isSettingsVisible).toBe(false);
  });
});
