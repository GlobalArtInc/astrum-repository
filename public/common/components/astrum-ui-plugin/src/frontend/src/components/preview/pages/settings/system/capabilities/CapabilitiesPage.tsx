

import React, { useState, useCallback, useEffect, useMemo, useRef } from 'react';
import { Box, Flex, Text } from '@radix-ui/themes';
import { Plus } from 'lucide-react';
import { ExtJS } from '../../../../../../interface/ExtJS';
import { useCurrentStateAndParams, useRouter } from '@uirouter/react';

import { SettingsButton, WizardForm } from '../../../../shared/form';
import { useToast, PageHeader } from '../../../../shared';
import { CapabilitiesList } from './CapabilitiesList';
import { CapabilityTypeSelector } from './CapabilityTypeSelector';
import { CapabilityDetail } from './CapabilityDetail';
import { DynamicCapabilityForm } from './DynamicCapabilityForm';
import { useCapabilitiesApi } from './useCapabilitiesApi';
import {
  Capability,
  CapabilityType,
  CapabilityFormData,
  CapabilitiesViewMode,
  CapabilitiesPageProps,
  getCapabilityDescription,
} from './types';

import './CapabilitiesPage.scss';
import CapabilitiesStrings from '../../../../constants/pages/settings/system/CapabilitiesStrings';
import CommonStrings from '../../../../constants/CommonStrings';

/**
 * CapabilitiesPage - Main Capabilities management page for Preview UI
 *
 * Displays capability list with search, and allows creating, editing,
 * enabling/disabling, and deleting capabilities. Uses dynamic forms
 * based on capability type configuration.
 */
const ROUTE_BASE = 'preview.admin.system.capabilities';

export function CapabilitiesPage({ className }: CapabilitiesPageProps) {
  const {
    loading,
    setError,
    fetchCapabilities,
    fetchCapability,
    fetchCapabilityTypes,
    createCapability,
    updateCapability,
    deleteCapability,
    enableCapability,
    disableCapability,
  } = useCapabilitiesApi();

  const toast = useToast();
  const router = useRouter();
  const { state: routeState, params } = useCurrentStateAndParams();

  const viewMode: CapabilitiesViewMode = useMemo(() => {
    const name = routeState?.name || '';
    if (name.endsWith('.detail')) return 'detail';
    if (name.endsWith('.createWithType')) return 'create';
    if (name.endsWith('.create')) return 'selectType';
    return 'list';
  }, [routeState?.name]);

  const [selectedCapability, setSelectedCapability] = useState<Capability | null>(null);
  const [selectedType, setSelectedType] = useState<CapabilityType | null>(null);
  const [refreshKey, setRefreshKey] = useState(0);
  const [createStep, setCreateStep] = useState(0);
  const [formReady, setFormReady] = useState(false);
  const formSubmitRef = useRef<(() => void) | null>(null);

  const canCreate = ExtJS.checkPermission('astrum:capabilities:create');

  // Deep link: load capability or type from URL params
  useEffect(() => {
    if (viewMode === 'detail' && params.capabilityId && !selectedCapability) {
      fetchCapabilities().then((caps) => {
        const cap = caps.find((c) => c.id === params.capabilityId);
        if (cap) {
          setSelectedCapability(cap);
        } else {
          router.stateService.go(`${ROUTE_BASE}.list`);
        }
      }).catch(() => {
        router.stateService.go(`${ROUTE_BASE}.list`);
      });
    }
    // Only auto-load the type on a genuine deep-link entry (wizard still at step 0).
    // Guarding on createStep prevents re-entering the create flow after handleBack
    // clears selectedType while the route change is still settling (ASTRUM-53839).
    if (viewMode === 'create' && params.typeId && !selectedType && createStep === 0) {
      fetchCapabilityTypes().then((types) => {
        const type = types.find((t) => t.id === params.typeId);
        if (type) {
          setSelectedType(type);
          setCreateStep(1);
        } else {
          router.stateService.go(`${ROUTE_BASE}.create`);
        }
      }).catch(() => {
        router.stateService.go(`${ROUTE_BASE}.create`);
      });
    }
    // Reset step when entering selectType view
    if (viewMode === 'selectType') {
      setCreateStep(0);
    }
  }, [viewMode, params.capabilityId, params.typeId, fetchCapabilities, fetchCapabilityTypes, router.stateService.go, selectedCapability, selectedType, createStep]);

  const handleSelectCapability = useCallback((capability: Capability) => {
    setSelectedType(null);
    setError(null);
    setSelectedCapability(capability);
    router.stateService.go(`${ROUTE_BASE}.detail`, { capabilityId: capability.id });
  }, [setError, router]);

  const handleStartCreate = useCallback(() => {
    setSelectedCapability(null);
    setSelectedType(null);
    setError(null);
    setCreateStep(0);
    router.stateService.go(`${ROUTE_BASE}.create`);
  }, [setError, router]);

  const handleSelectType = useCallback((type: CapabilityType) => {
    setSelectedType(type);
    setError(null);
  }, [setError]);

  const handleBack = useCallback(() => {
    setSelectedCapability(null);
    setSelectedType(null);
    setError(null);
    // Clear the stale submit ref/readiness from the create wizard. We intentionally
    // do NOT reset createStep here: while the route change to .list settles, the
    // deep-link effect must still see createStep === 1 so it does not re-fetch the
    // type and bounce back into the create flow (ASTRUM-53839). createStep is reset
    // on the next genuine create entry (handleStartCreate / selectType view).
    setFormReady(false);
    formSubmitRef.current = null;
    router.stateService.go(`${ROUTE_BASE}.list`);
  }, [setError, router]);

  const handleBackToTypeSelect = useCallback(() => {
    setSelectedType(null);
    setError(null);
    setCreateStep(0);
    setFormReady(false);
    formSubmitRef.current = null;
    router.stateService.go(`${ROUTE_BASE}.create`);
  }, [setError, router]);

  const handleWizardStepChange = useCallback((step: number) => {
    if (step === 1 && selectedType) {
      setCreateStep(1);
      router.stateService.go(`${ROUTE_BASE}.createWithType`, { typeId: selectedType.id });
    } else if (step === 0) {
      setCreateStep(0);
      setSelectedType(null);
      handleBackToTypeSelect();
    } else {
      setCreateStep(step);
    }
  }, [selectedType, router, handleBackToTypeSelect]);

  const handleWizardComplete = useCallback(() => {
    // Trigger form submission
    if (formSubmitRef.current) {
      formSubmitRef.current();
    }
  }, []);

  const handleWizardCancel = useCallback(() => {
    if (createStep === 0) {
      handleBack();
    } else {
      handleBackToTypeSelect();
    }
  }, [createStep, handleBack, handleBackToTypeSelect]);

  const handleSave = useCallback(async (data: CapabilityFormData) => {
    try {
      if (viewMode === 'create') {
        const result = await createCapability(data);
        toast.success(CapabilitiesStrings.PAGE.CAPABILITY_CREATED(getCapabilityDescription(result)));
      } else if (selectedCapability) {
        const result = await updateCapability(data);
        toast.success(CapabilitiesStrings.PAGE.CAPABILITY_UPDATED(getCapabilityDescription(result)));
      }
      setRefreshKey((k) => k + 1);
      handleBack();
    } catch (err) {
      const message = err instanceof Error ? err.message : CapabilitiesStrings.PAGE.FAILED_SAVE_CAPABILITY;
      toast.error(message);
      throw err;
    }
  }, [viewMode, selectedCapability, createCapability, updateCapability, handleBack, toast.error, toast.success]);

  const handleDelete = useCallback(async () => {
    if (!selectedCapability) return;

    try {
      await deleteCapability(selectedCapability.id);
      toast.success(CapabilitiesStrings.PAGE.CAPABILITY_DELETED(getCapabilityDescription(selectedCapability)));
      setRefreshKey((k) => k + 1);
      handleBack();
    } catch (err) {
      const message = err instanceof Error ? err.message : CapabilitiesStrings.PAGE.FAILED_DELETE_CAPABILITY;
      toast.error(message);
    }
  }, [selectedCapability, deleteCapability, handleBack, toast.error, toast.success]);

  // The server decides the resulting state (an enabled capability can stay passive or fail), so
  // replace the optimistic state with the capability as the server reports it
  const reloadSelectedCapability = useCallback((id: string) => {
    fetchCapability(id)
      .then((fresh) => setSelectedCapability((prev) => (prev && prev.id === id ? fresh : prev)))
      .catch(() => undefined);
  }, [fetchCapability]);

  const handleEnable = useCallback(async () => {
    if (!selectedCapability) return;

    try {
      await enableCapability(selectedCapability);
      toast.success(CapabilitiesStrings.PAGE.CAPABILITY_ENABLED(selectedCapability.typeName));
      setRefreshKey((k) => k + 1);
      setSelectedCapability((prev) => prev ? { ...prev, enabled: true, state: 'active' } : null);
      reloadSelectedCapability(selectedCapability.id);
    } catch (err) {
      const message = err instanceof Error ? err.message : CapabilitiesStrings.PAGE.FAILED_ENABLE_CAPABILITY;
      toast.error(message);
    }
  }, [selectedCapability, enableCapability, reloadSelectedCapability, toast.error, toast.success]);

  const handleDisable = useCallback(async () => {
    if (!selectedCapability) return;

    try {
      await disableCapability(selectedCapability);
      toast.success(CapabilitiesStrings.PAGE.CAPABILITY_DISABLED(selectedCapability.typeName));
      setRefreshKey((k) => k + 1);
      // Refresh detail view
      setSelectedCapability((prev) => prev ? { ...prev, enabled: false, state: 'disabled' } : null);
      reloadSelectedCapability(selectedCapability.id);
    } catch (err) {
      const message = err instanceof Error ? err.message : CapabilitiesStrings.PAGE.FAILED_DISABLE_CAPABILITY;
      toast.error(message);
    }
  }, [selectedCapability, disableCapability, reloadSelectedCapability, toast.error, toast.success]);

  // Navigation helper for Settings breadcrumb
  const navigateToSettings = () => {
    window.location.hash = '#preview/admin/settings';
  };

  const detailBreadcrumbs = [
    { label: CommonStrings.LABELS.SETTINGS, onClick: navigateToSettings },
    { label: CapabilitiesStrings.PAGE.CAPABILITIES, onClick: handleBack },
    { label: selectedCapability?.typeName || CapabilitiesStrings.PAGE.CAPABILITY },
  ];

  // Render header based on view mode
  const renderHeader = () => {
    if (viewMode === 'list') {
      const breadcrumbs = [
        { label: CommonStrings.LABELS.SETTINGS, onClick: navigateToSettings },
        { label: CapabilitiesStrings.PAGE.CAPABILITIES },
      ];
      const actions = canCreate ? (
        <SettingsButton variant="primary" onClick={handleStartCreate} data-analytics-id="as-capability-create" icon={Plus}>
          {CapabilitiesStrings.PAGE.CREATE_CAPABILITY}
        </SettingsButton>
      ) : undefined;
      return (
        <PageHeader
          title={CapabilitiesStrings.PAGE.CAPABILITIES}
          description={CapabilitiesStrings.PAGE.MANAGE_REPOSITORY_MANAGER_CAPABILITIES_PLUGINS}
          breadcrumbs={breadcrumbs}
          actions={actions}
          className="capabilities-page__header"
        />
      );
    }

    // selectType and create views: show breadcrumbs only (WizardForm provides its own title)
    if (viewMode === 'selectType' || viewMode === 'create') {
      return (
        <PageHeader
          breadcrumbs={[
            { label: CommonStrings.LABELS.SETTINGS, onClick: navigateToSettings },
            { label: CapabilitiesStrings.PAGE.CAPABILITIES, onClick: handleBack },
            { label: CommonStrings.ACTIONS.CREATE },
          ]}
          className="capabilities-page__header"
        />
      );
    }

    // Detail view: once the capability is loaded, CapabilityDetail renders the header with its
    // name, state and actions
    if (selectedCapability) {
      return null;
    }

    return (
      <PageHeader
        title={CapabilitiesStrings.PAGE.CAPABILITY}
        breadcrumbs={detailBreadcrumbs}
        className="capabilities-page__header"
      />
    );
  };

  return (
    <Box 
      className={`capabilities-page ${className || ''}`.trim()}
      data-testid="capabilities-page"
      data-view={viewMode}
      data-loading={loading ? 'true' : 'false'}
    >
      {renderHeader()}

      {/* Alerts removed - using Toast notifications instead (Sprint 15 UX fix) */}

      {/* Content */}
      <Box className="capabilities-page__content">
        {viewMode === 'list' && (
          <CapabilitiesList
            key={refreshKey}
            onSelect={handleSelectCapability}
          />
        )}

        {/* Wizard Form for Create Flow */}
        {(viewMode === 'selectType' || viewMode === 'create') && (
          <WizardForm
            steps={[
              { id: 'type', label: CapabilitiesStrings.PAGE.SELECT_TYPE },
              { id: 'config', label: CommonStrings.LABELS.CONFIGURE_STEP },
            ]}
            currentStep={createStep}
            onStepChange={handleWizardStepChange}
            onComplete={handleWizardComplete}
            onCancel={handleWizardCancel}
            completeLabel={CommonStrings.ACTIONS.CREATE}
            dirty={createStep > 0}
            canAdvance={createStep === 0 ? !!selectedType : formReady}
            loading={loading}
            testId="capability-wizard"
            noDirtyTracking={true}
          >
            {createStep === 0 && (
              <CapabilityTypeSelector onSelect={handleSelectType} selectedTypeId={selectedType?.id} />
            )}
            {createStep === 1 && selectedType && (
              <DynamicCapabilityForm
                capabilityType={selectedType}
                isCreate={true}
                onSave={handleSave}
                onCancel={handleBackToTypeSelect}
                onSaveComplete={handleBack}
                loading={loading}
                embedded={true}
                onSubmitRef={(submitFn) => {
                  formSubmitRef.current = submitFn;
                  setFormReady(submitFn !== null);
                }}
              />
            )}
            {createStep === 1 && !selectedType && params.typeId && (
              <Flex align="center" justify="center" p="6" gap="2">
                <Text size="2">{CommonStrings.MESSAGES.LOADING}</Text>
              </Flex>
            )}
          </WizardForm>
        )}

        {viewMode === 'detail' && !selectedCapability && params.capabilityId && (
          <Flex align="center" justify="center" p="6" gap="2">
            <Text size="2">{CommonStrings.MESSAGES.LOADING}</Text>
          </Flex>
        )}

        {viewMode === 'detail' && selectedCapability && (
          <CapabilityDetail
            capability={selectedCapability}
            onSave={handleSave}
            onDelete={handleDelete}
            onEnable={handleEnable}
            onDisable={handleDisable}
            onBack={handleBack}
            breadcrumbs={detailBreadcrumbs}
            loading={loading}
          />
        )}
      </Box>
    </Box>
  );
}

export default CapabilitiesPage;
