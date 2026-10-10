

import React from 'react';
import { compose, propOr, sortBy, toLower } from 'ramda';
import { useMachine } from '@xstate/react';
import { useRouter } from '@uirouter/react';
import { AsForm, AsFormGroup, AsFormSelect, AsH2, AsLoadWrapper, AsP, AsTile } from '@globalart/astrum-react-shared-components';
import { ContentBody, Page, PageHeader, PageTitle, Section } from '../../layout';
import UIStrings from '../../../constants/UIStrings';
import { AdminRouteNames } from '../../../constants/admin/AdminRouteNames';
import FormUtils from '../../../interface/FormUtils';
import CapabilitiesCreateMachine from './CapabilitiesCreateMachine';
import DynamicFormField from '../../widgets/DynamicFormField/DynamicFormField';
import './Capabilities.scss';

const { CAPABILITIES } = UIStrings;

export default function CapabilitiesCreate() {
  const router = useRouter();
  const [current, send] = useMachine(CapabilitiesCreateMachine, {
    actions: {
      onSaveSuccess: () => {
        router.stateService.go(AdminRouteNames.SYSTEM.CAPABILITIES.LIST);
      },
    },
    devTools: false,
  });

  const {
    types: rawTypes = [],
    loadError,
    selectedType,
    shouldShowErrors,
    hasFields,
    allFieldsOptional,
  } = current.context;
  const isLoading = current.matches('loading');

  // Sort types alphabetically by name using Ramda
  const types = sortBy(compose(toLower, propOr('', 'name')), rawTypes);

  const handleRetry = () => {
    send({ type: 'RETRY' });
  };

  const handleTypeChange = value => {
    const type = types?.find(type => type.id === value);
    if (type) {
      send({
        type: 'SET_SELECTED_TYPE',
        selectedType: type,
      });
    } else {
      send({
        type: 'SET_SELECTED_TYPE',
        selectedType: null,
      });
    }
  };

  const updateField = (name, value) => {
    send({
      type: 'UPDATE',
      data: {
        [name]: value,
      },
    });
  };

  const handleCancel = () => {
    router.stateService.go(AdminRouteNames.SYSTEM.CAPABILITIES.LIST);
  };

  // Custom formProps that handles pristine state for forms with all optional fields
  const baseFormProps = FormUtils.formProps(current, send);
  const shouldAllowPristineSave = !hasFields || allFieldsOptional;

  const formProps = {
    ...baseFormProps,
    // Override validationErrors to not show pristine tooltip if pristine saves are allowed
    validationErrors:
      shouldAllowPristineSave && current.context.isPristine && !FormUtils.isInvalid(current.context.validationErrors)
        ? null
        : baseFormProps.validationErrors,
  };

  const handleSubmit = () => {
    formProps.onSubmit(); // Calls send({ type: 'SAVE' })
  };

  return (
    <Page>
      <PageHeader>
        <PageTitle text={CAPABILITIES.CREATE.TITLE} description={CAPABILITIES.CREATE.DESCRIPTION} />
      </PageHeader>
      <ContentBody>
        <AsLoadWrapper loading={isLoading} error={loadError} retryHandler={handleRetry}>
          <Section>
            <AsForm
              {...formProps}
              showValidationErrors={shouldShowErrors}
              onSubmit={handleSubmit}
              onCancel={handleCancel}
              data-analytics-id='as-create-capabilities-form'
              className='as-capabilities-form'
            >
              <AsTile.Subsection>
                <AsTile.SubsectionHeader>
                  <AsH2>{CAPABILITIES.CREATE.FORM.TITLE}</AsH2>
                </AsTile.SubsectionHeader>
              </AsTile.Subsection>
              <AsTile.Subsection>
                <AsFormGroup label={CAPABILITIES.CREATE.FORM.SELECT_TYPE} isRequired>
                  <AsFormSelect value={selectedType?.id || ''} onChange={handleTypeChange}>
                    <option value='' disabled={!!selectedType}>
                      {CAPABILITIES.CREATE.FORM.SELECT_TYPE}
                    </option>
                    {types.map(type => (
                      <option key={type.id} value={type.id}>
                        {type.name}
                      </option>
                    ))}
                  </AsFormSelect>
                </AsFormGroup>
              </AsTile.Subsection>
              {selectedType && (
                <>
                  <AsTile.Subsection>
                    <AsTile.SubsectionHeader>
                      <AsH2>{selectedType.name}</AsH2>
                    </AsTile.SubsectionHeader>
                    <AsP dangerouslySetInnerHTML={{ __html: selectedType.about }} />
                  </AsTile.Subsection>
                  {selectedType.formFields && selectedType.formFields.length > 0 && (
                    <AsTile.Subsection data-testid='form-fields'>
                      {selectedType.formFields.map(field => (
                        <DynamicFormField
                          key={`${selectedType.id}-${field.id}`}
                          id={field.id}
                          current={current}
                          initialValue={field.initialValue}
                          onChange={updateField}
                          dynamicProps={field}
                        />
                      ))}
                    </AsTile.Subsection>
                  )}
                </>
              )}
            </AsForm>
          </Section>
        </AsLoadWrapper>
      </ContentBody>
    </Page>
  );
}
