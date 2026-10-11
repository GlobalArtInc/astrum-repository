import React from 'react';
import { render, screen, within } from '@testing-library/react';
import { Theme } from '@radix-ui/themes';

import { RepositoryForm } from '../RepositoryForm';
import { RepositoryChips } from '../RepositoryChips';
import { useRepositoryForm } from '../useRepositoryForm';
import { ToastProvider } from '../../../../../shared/Toast';

jest.mock('../useRepositoryForm');
jest.mock('../RepositoryAuditTab', () => ({ RepositoryAuditTab: () => null }));
jest.mock('../RepositoryTasksCapabilitiesTab', () => ({ RepositoryTasksCapabilitiesTab: () => null }));
jest.mock('../RepositoryAccessSecurityTab', () => ({ RepositoryAccessSecurityTab: () => null }));
jest.mock('../RepositoryFirewallConfigTab', () => ({ RepositoryFirewallConfigTab: () => null }));
jest.mock('../RepositorySummary', () => ({ RepositorySummary: () => <div data-testid="summary" /> }));

jest.mock('../../../../../../../interface/ExtJS', () => {
  const mockExtJS = {
    checkPermission: jest.fn().mockReturnValue(true),
    usePermission: jest.fn((getValue: () => boolean) => getValue()),
    useUser: jest.fn(() => ({ id: 'admin' })),
    state: jest.fn().mockReturnValue({ getValue: jest.fn().mockReturnValue(undefined) }),
  };
  return { __esModule: true, default: mockExtJS, ExtJS: mockExtJS };
});

const mockUseRepositoryForm = useRepositoryForm as jest.MockedFunction<typeof useRepositoryForm>;

const REPOSITORY = {
  name: 'docker-onprem',
  type: 'hosted',
  format: 'docker',
  online: false,
  url: 'http://localhost:8081/repository/docker-onprem',
};
const RECIPE = { format: 'docker', type: 'hosted' as const, name: 'docker-hosted' };
const FORM_DATA = {
  name: 'docker-onprem',
  type: 'hosted',
  format: 'docker',
  online: false,
  storage: { blobStoreName: 'default', strictContentTypeValidation: true },
};

function hookReturn(overrides: Record<string, unknown> = {}) {
  return {
    form: {
      field: jest.fn((name: string) => ({ name, value: '', onChange: jest.fn(), onBlur: jest.fn() })),
      data: FORM_DATA,
      isPristine: true,
      isSaving: false,
      isLoading: false,
      saveError: null,
      validationErrors: {},
      state: { matches: jest.fn(() => false), context: { data: FORM_DATA } },
      send: jest.fn(),
    },
    repository: REPOSITORY,
    isCreate: false,
    hasFirewallLicense: false,
    isCloud: false,
    activeTab: 'settings',
    setActiveTab: jest.fn(),
    originChangeWarning: false,
    setOriginChangeWarning: jest.fn(),
    formData: FORM_DATA,
    pristineData: FORM_DATA,
    errors: {},
    blobStores: [{ name: 'default' }],
    routingRules: [],
    cleanupPolicies: [],
    memberRepositories: [],
    handleChange: jest.fn(),
    handleNestedChange: jest.fn(),
    ...overrides,
  } as any;
}

function renderEdit(renderHeader?: (actions?: React.ReactNode) => React.ReactNode) {
  return render(
    <Theme>
      <ToastProvider>
        <RepositoryForm
          repository={REPOSITORY as any}
          recipe={RECIPE}
          isCreate={false}
          onSave={jest.fn()}
          onCancel={jest.fn()}
          onDelete={jest.fn()}
          onRebuildIndex={jest.fn()}
          onToggleOnline={jest.fn()}
          onBrowseRepository={jest.fn()}
          renderHeader={renderHeader}
        />
      </ToastProvider>
    </Theme>
  );
}

describe('RepositoryForm edit page layout', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockUseRepositoryForm.mockReturnValue(hookReturn());
  });

  it('hands the repository actions to the page header, with Delete last', () => {
    renderEdit((actions) => <header data-testid="page-header">{actions}</header>);

    const header = screen.getByTestId('page-header');
    const buttons = within(header).getAllByRole('button').map((b) => b.textContent);
    expect(buttons).toEqual(['Browse Repository', 'Rebuild Index', 'Bring Online', 'Delete']);
  });

  it('names the online toggle after what it does', () => {
    mockUseRepositoryForm.mockReturnValue(hookReturn({ pristineData: { ...FORM_DATA, online: true } }));
    renderEdit((actions) => <header>{actions}</header>);

    expect(screen.getByTestId('form-toggle-online')).toHaveTextContent('Take Offline');
    expect(screen.queryByText(/system status/i)).not.toBeInTheDocument();
  });

  it('keeps Save and Cancel in a bar below the settings', () => {
    renderEdit((actions) => <header>{actions}</header>);

    const form = screen.getByTestId('repository-form');
    const bar = form.querySelector('.settings-form__action-bar');
    expect(bar).toHaveClass('settings-form__action-bar--bottom');
    expect(form.lastElementChild).toBe(bar);
    expect(within(bar as HTMLElement).getByTestId('form-submit')).toHaveTextContent('Save Changes');
    expect(within(bar as HTMLElement).queryByTestId('form-delete')).not.toBeInTheDocument();
  });

  it('hides Save and Cancel on a read-only tab while nothing is changed', () => {
    mockUseRepositoryForm.mockReturnValue(hookReturn({ activeTab: 'summary' }));
    renderEdit((actions) => <header>{actions}</header>);

    expect(screen.queryByTestId('form-submit')).not.toBeInTheDocument();
    expect(screen.queryByTestId('form-cancel')).not.toBeInTheDocument();
  });

  it('shows Save and Cancel on a read-only tab when the settings have unsaved changes', () => {
    const dirty = hookReturn({ activeTab: 'summary' });
    dirty.form.isPristine = false;
    mockUseRepositoryForm.mockReturnValue(dirty);
    renderEdit((actions) => <header>{actions}</header>);

    expect(screen.getByTestId('form-submit')).toBeInTheDocument();
  });

  it('renders the header while the form data loads', () => {
    const loading = hookReturn();
    loading.form.isLoading = true;
    mockUseRepositoryForm.mockReturnValue(loading);
    renderEdit(() => <header data-testid="page-header">docker-onprem</header>);

    expect(screen.getByTestId('page-header')).toBeInTheDocument();
  });

  it('lists the actions above the tabs when there is no page header to put them in', () => {
    renderEdit();

    expect(screen.getByTestId('form-delete')).toBeInTheDocument();
    expect(screen.getByTestId('form-browse-repository')).toBeInTheDocument();
  });
});

describe('RepositoryChips', () => {
  it('shows status, type and format by their names', () => {
    render(
      <Theme>
        <RepositoryChips format="maven2" type="proxy" online />
      </Theme>
    );

    expect(screen.getByTestId('repository-chip-status')).toHaveTextContent('Online');
    expect(screen.getByTestId('repository-chip-type')).toHaveTextContent('Proxy');
    expect(screen.getByTestId('repository-chip-format')).toHaveTextContent('Maven');
  });

  it('shows an offline repository as offline', () => {
    render(
      <Theme>
        <RepositoryChips format="docker" type="hosted" online={false} />
      </Theme>
    );

    expect(screen.getByTestId('repository-chip-status')).toHaveTextContent('Offline');
  });
});
