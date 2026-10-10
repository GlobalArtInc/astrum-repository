
import React, { useEffect, useState } from 'react';

import { DynamicFormField, ValidationUtils } from '@globalart/astrum-ui-plugin';
import { AsFormGroup, AsH2, AsTile, AsForm } from '@globalart/astrum-react-shared-components';
import { ContentBody, Page, PageHeader, PageTitle } from '@globalart/astrum-ui-plugin';
import { isEmpty } from 'ramda';
// Real server data examples for each DynamicFormField type
const testData = {
  // String field example from rutauth capability
  stringField: {
    id: 'httpHeader',
    type: 'string',
    label: 'HTTP Header name (string)',
    helpText:
      'Handled HTTP Header should contain the name of the header that is used to source the principal of already authenticated user.',
    required: true,
    disabled: false,
    readOnly: false,
    initialValue: null,
    attributes: {},
  },

  // String field with long attribute example
  stringLongField: {
    id: 'title',
    type: 'string',
    label: 'Title (string - long)',
    helpText: 'Browser page title',
    required: true,
    disabled: false,
    readOnly: false,
    initialValue: 'GlobalArt Astrum Repository',
    attributes: { long: true },
  },

  // String field with disabled state
  stringDisabledField: {
    id: 'outreachContentUrl',
    type: 'string',
    label: 'Default Outreach Content URL (string - read-only)',
    helpText: 'Default external URL for downloading new Outreach content.',
    required: true,
    disabled: false,
    readOnly: true,
    initialValue: 'https://links.globalart.dev/products/astrum/outreach',
    attributes: {},
  },

  // Number field example from StorageSettings capability
  numberField: {
    id: 'lastDownloadedInterval',
    type: 'number',
    label: 'Asset (number)',
    helpText: "'Last Downloaded' Update Interval (hours)",
    required: true,
    disabled: false,
    readOnly: false,
    initialValue: '12',
    attributes: {
      minValue: '1',
      maxValue: null,
    },
  },

  // Checkbox field example from crowd capability
  checkboxField: {
    id: 'useTrustStore',
    type: 'checkbox',
    label: 'Use the Astrum Repository truststore (checkbox)',
    helpText: null,
    required: false,
    disabled: false,
    readOnly: false,
    initialValue: false,
    attributes: {},
  },

  // URL field example from crowd capability
  urlField: {
    id: 'crowdServerUrl',
    type: 'url',
    label: 'Crowd Server URL (url - required)',
    helpText: 'The URL of the Crowd server. Example: http://localhost:8095/crowd',
    required: true,
    disabled: false,
    readOnly: false,
    initialValue: null,
    attributes: {},
  },

  // URL field - Optional (not required)
  urlOptionalField: {
    id: 'baseUrl',
    type: 'url',
    label: 'Base URL (url - optional)',
    helpText: 'Optional base URL for external links. Example: https://example.com',
    required: false,
    disabled: false,
    readOnly: false,
    initialValue: 'something.com',
    attributes: {},
  },

  // Password field example from crowd capability
  passwordField: {
    id: 'applicationPassword',
    type: 'password',
    label: 'Application Password (password)',
    helpText: 'The password to use when authenticating the Application connection to the Crowd server.',
    required: true,
    disabled: false,
    readOnly: false,
    initialValue: 'my pass',
    attributes: {},
  },

  // Text-area field example from license-expiration capability
  textareaField: {
    id: 'notificationEmails',
    type: 'text-area',
    label: 'Notification Emails (text-area)',
    helpText: 'Comma-separated list of email addresses to notify when license is about to expire',
    required: true,
    disabled: false,
    readOnly: false,
    initialValue: 'Lorem ipsum dolor sit, amet consectetur adipisicing elit. Id aperiam fuga nulla, beatae reiciendis dolore aspernatur. Animi dolorum ab eius beatae, nostrum, ex similique accusamus reiciendis hic impedit incidunt pariatur?',
    attributes: {},
  },

  // Combobox field example from webhook.repository capability
  comboboxField: {
    id: 'repository',
    type: 'combobox',
    label: 'Repository (combobox)',
    helpText: 'Repository to discriminate events from',
    required: true,
    disabled: false,
    readOnly: false,
    initialValue: ['maven-central', 'npmjs'],
    attributes: {
      options: {
        'maven-central': 'Maven Central',
        npmjs: 'npmjs',
        'nuget.org': 'nuget.org',
        pypi: 'PyPI',
        rubygems: 'RubyGems',
      },
    },
  },

  // Item select field example from webhook.repository capability
  itemselectField: {
    id: 'names',
    type: 'itemselect',
    label: 'Event Types (itemselect)',
    helpText: 'Event types which trigger this Webhook',
    required: true,
    disabled: false,
    readOnly: false,
    initialValue: ['repository.created', 'component.created'],
    attributes: {
      toTitle: 'Selected',
      fromTitle: 'Available',
      options: [
        'repository.created',
        'repository.updated',
        'repository.deleted',
        'component.created',
        'component.updated',
        'component.deleted',
        'asset.created',
        'asset.updated',
        'asset.deleted',
      ],
    },
  },

  // Combobox with storeApi example from webhook.repository capability
  comboboxWithStoreApi: {
    id: 'repositoryWebhook',
    type: 'combobox',
    label: 'Repository (combobox with storeApi)',
    helpText: 'Repository to discriminate events from',
    required: true,
    disabled: false,
    readOnly: false,
    initialValue: null,
    attributes: {},
    storeApi: 'coreui_Repository.readReferences',
    storeFilters: {
      type: '!group',
    },
    idMapping: null,
    nameMapping: null,
    allowAutocomplete: true,
  },

  // Combobox with storeApi and complex filters from firewall.audit capability
  comboboxWithStoreApiFilters: {
    id: 'repositoryFirewall',
    type: 'combobox',
    label: 'Repository (combobox with storeApi + filters)',
    helpText: 'Select a repository to be evaluated.',
    required: true,
    disabled: false,
    readOnly: false,
    initialValue: 'something',
    attributes: {},
    storeApi: 'coreui_Repository.readReferences',
    storeFilters: {
      format:
        'r,go,p2,apt,rubygems,npm,yum,pypi,cargo,conan,conda,nuget,maven2,composer,cocoapods,huggingface,docker,raw',
      facets: 'org.globalart.astrum.repository.proxy.ProxyFacet',
    },
    idMapping: null,
    nameMapping: null,
    allowAutocomplete: true,
  },

  // Combobox with storeApi (simple, no filters) from defaultrole capability
  comboboxWithStoreApiSimple: {
    id: 'role',
    type: 'combobox',
    label: 'Role (combobox with storeApi - simple)',
    helpText: 'The role which is automatically granted to authenticated users',
    required: true,
    disabled: false,
    readOnly: false,
    initialValue: 'something',
    attributes: {},
    storeApi: 'coreui_Role.read',
    storeFilters: null,
    idMapping: null,
    nameMapping: null,
    allowAutocomplete: false,
  },

  // Itemselect with storeApi from webhook.repository capability
  itemselectWithStoreApi: {
    id: 'webhookEventsRepository',
    type: 'itemselect',
    label: 'Event Types (itemselect with storeApi - Repository)',
    helpText: 'Event types which trigger this Webhook',
    required: true,
    disabled: false,
    readOnly: false,
    initialValue: null,
    attributes: {
      toTitle: 'Selected',
      fromTitle: 'Available',
      buttons: ['add', 'remove'],
    },
    storeApi: 'coreui_Webhook.listWithTypeRepository',
    storeFilters: null,
    idMapping: null,
    nameMapping: null,
    allowAutocomplete: false,
  },

  // Itemselect with storeApi from webhook.global capability
  itemselectWithStoreApiGlobal: {
    id: 'webhookEventsGlobal',
    type: 'itemselect',
    label: 'Event Types (itemselect with storeApi - Global)',
    helpText: 'Event types which trigger this Webhook',
    required: true,
    disabled: false,
    readOnly: false,
    initialValue: null,
    attributes: {
      toTitle: 'Selected',
      fromTitle: 'Available',
      buttons: ['add', 'remove'],
    },
    storeApi: 'coreui_Webhook.listWithTypeGlobal',
    storeFilters: null,
    idMapping: null,
    nameMapping: null,
    allowAutocomplete: false,
  },
};

// Mock form context for DynamicFormField with automatic required field validation
const createMockContext = (data, fieldValidationErrors = {}, fieldDefinitions = {}, showValidationErrors = false) => {
  // Auto-generate validation errors for required fields that are empty
  // Only generate errors when showValidationErrors is true (after form submission)
  const autoValidationErrors = { ...fieldValidationErrors };

  if (showValidationErrors) {
    Object.entries(fieldDefinitions).forEach(([fieldId, fieldDef]) => {
      if (fieldDef.required && ValidationUtils.isBlank(data[fieldId])) {
        autoValidationErrors[fieldId] = 'This field is required';
      }
    });
  }

  // Mark all fields as touched when showValidationErrors is true (e.g., on form submit)
  const isTouched = {};
  if (showValidationErrors) {
    Object.keys(fieldDefinitions).forEach(fieldId => {
      isTouched[fieldId] = true;
    });
  }

  return {
    context: {
      data,
      pristineData: data,
      isTouched,
      validationErrors: autoValidationErrors,
      saveErrors: {},
      saveErrorData: {},
    },
  };
};

export default function TestForm() {
  const [formData, setFormData] = useState({
    httpHeader: '',
    title: 'GlobalArt Astrum Repository',
    outreachContentUrl: 'https://links.globalart.dev/products/astrum/outreach',
    baseUrl: '',
    lastDownloadedInterval: '12',
    useTrustStore: false,
    crowdServerUrl: '',
    applicationPassword: '',
    notificationEmails: '',
    repository: '',
    names: [],
    repositoryWebhook: '',
    repositoryFirewall: '',
    role: '',
    webhookEventsRepository: [],
    webhookEventsGlobal: [],
  });

  const [showValidationErrors, setShowValidationErrors] = useState(false);
  const [validationErrors, setValidationErrors] = useState(undefined);
  const [submitMaskState, setSubmitMaskState] = useState(null);

  // Create field definitions for automatic validation
  const fieldDefinitions = {
    httpHeader: { required: testData.stringField.required },
    title: { required: testData.stringLongField.required },
    outreachContentUrl: { required: testData.stringDisabledField.required },
    baseUrl: { required: testData.urlOptionalField.required },
    lastDownloadedInterval: { required: testData.numberField.required },
    useTrustStore: { required: testData.checkboxField.required },
    crowdServerUrl: { required: testData.urlField.required },
    applicationPassword: { required: testData.passwordField.required },
    notificationEmails: { required: testData.textareaField.required },
    repository: { required: testData.comboboxField.required },
    repositoryWebhook: { required: testData.comboboxWithStoreApi.required },
    repositoryFirewall: { required: testData.comboboxWithStoreApiFilters.required },
    role: { required: testData.comboboxWithStoreApiSimple.required },
    names: { required: testData.itemselectField.required },
    webhookEventsRepository: { required: testData.itemselectWithStoreApi.required },
    webhookEventsGlobal: { required: testData.itemselectWithStoreApiGlobal.required },
  };

  const updateField = (name, value) => {
    setFormData(prev => ({ ...prev, [name]: value }));
  };

  const handleSubmit = () => {
    // Generate validation errors for required fields that are empty (before checking)
    // This mirrors the logic in createMockContext but runs before setting showValidationErrors
    const autoErrors = {};
    Object.entries(fieldDefinitions).forEach(([fieldId, fieldDef]) => {
      if (fieldDef.required && ValidationUtils.isBlank(formData[fieldId])) {
        autoErrors[fieldId] = 'This field is required';
      }
    });

    // Check if there are any validation errors
    const hasErrors = Object.keys(autoErrors).length > 0;

    // Always update showValidationErrors as per RSC rules
    setShowValidationErrors(true);

    if (hasErrors) {
      // Set validation errors for AsForm to display
      setValidationErrors('Please correct the validation errors below');
      return;
    }

    // Clear validation errors if form is valid
    setValidationErrors(undefined);

    // Show submitting mask
    setSubmitMaskState(false);

    // Simulate form submission
    setTimeout(() => {
      setSubmitMaskState(true);

      // Show success message
      alert('Form is VALID! All required fields are filled and validation passed.');

      // Hide success mask after a short delay
      setTimeout(() => {
        setSubmitMaskState(null);
        setShowValidationErrors(false);
      }, 2000);
    }, 1000);
  };

  const handleCancel = () => {
    // Reset form to initial state
    setFormData({
      httpHeader: '',
      title: 'GlobalArt Astrum Repository',
      outreachContentUrl: 'https://links.globalart.dev/products/astrum/outreach',
      baseUrl: '',
      lastDownloadedInterval: '12',
      useTrustStore: false,
      crowdServerUrl: '',
      applicationPassword: '',
      notificationEmails: '',
      repository: '',
      names: [],
      repositoryWebhook: '',
      repositoryFirewall: '',
      role: '',
      webhookEventsRepository: [],
      webhookEventsGlobal: [],
    });
    setShowValidationErrors(false);
    setValidationErrors(undefined);
    setSubmitMaskState(null);
  };

  // Create context with current form data and automatic validation
  const current = createMockContext(formData, {}, fieldDefinitions, showValidationErrors);

  const readOnly = true;

  useEffect(() => {
    if (isEmpty(current?.context.validationErrors)) {
      setValidationErrors(undefined);
    }
  }, [current?.context.validationErrors]);
  return (
    <Page className="as-malware-risk">
      <PageHeader>
        <PageTitle text="DynamicFormField Test Form" />
      </PageHeader>
      <ContentBody>
        <AsTile>
          <AsTile.Header>
            <AsTile.HeaderTitle>
              <AsH2>DynamicFormField Test Form</AsH2>
            </AsTile.HeaderTitle>
          </AsTile.Header>
          <AsTile.Content>
            <AsForm
              onSubmit={handleSubmit}
              onCancel={handleCancel}
              showValidationErrors={showValidationErrors}
              validationErrors={validationErrors}
              submitMaskState={submitMaskState}
              submitBtnText="Submit Form"
              submitMaskMessage="Submitting form..."
              submitMaskSuccessMessage="Form submitted successfully!"
            >
              {/* String Field - Basic */}
              <DynamicFormField
                id={testData.stringField.id}
                current={current}
                initialValue={testData.stringField.initialValue}
                onChange={updateField}
                dynamicProps={testData.stringField}
                readOnly={readOnly}
              />

              {/* String Field - Long */}
              <DynamicFormField
                id={testData.stringLongField.id}
                current={current}
                initialValue={testData.stringLongField.initialValue}
                onChange={updateField}
                dynamicProps={testData.stringLongField}
                readOnly={readOnly}
              />

              {/* String Field - Read Only */}
              <DynamicFormField
                id={testData.stringDisabledField.id}
                current={current}
                initialValue={testData.stringDisabledField.initialValue}
                onChange={updateField}
                dynamicProps={testData.stringDisabledField}
                readOnly={readOnly}
              />

              {/* Number Field */}
              <DynamicFormField
                id={testData.numberField.id}
                current={current}
                initialValue={testData.numberField.initialValue}
                onChange={updateField}
                dynamicProps={testData.numberField}
                readOnly={readOnly}
              />

              {/* Checkbox Field */}
              <DynamicFormField
                id={testData.checkboxField.id}
                current={current}
                initialValue={testData.checkboxField.initialValue}
                onChange={updateField}
                dynamicProps={testData.checkboxField}
                readOnly={readOnly}
              />

              {/* URL Field - Required */}
              <DynamicFormField
                id={testData.urlField.id}
                current={current}
                initialValue={testData.urlField.initialValue}
                onChange={updateField}
                dynamicProps={testData.urlField}
                readOnly={readOnly}
              />

              {/* URL Field - Optional */}
              <DynamicFormField
                id={testData.urlOptionalField.id}
                current={current}
                initialValue={testData.urlOptionalField.initialValue}
                onChange={updateField}
                dynamicProps={testData.urlOptionalField}
                readOnly={readOnly}
              />

              {/* Password Field */}
              <DynamicFormField
                id={testData.passwordField.id}
                current={current}
                initialValue={testData.passwordField.initialValue}
                onChange={updateField}
                dynamicProps={testData.passwordField}
                readOnly={readOnly}
              />

              {/* Text-area Field */}
              <DynamicFormField
                id={testData.textareaField.id}
                current={current}
                initialValue={testData.textareaField.initialValue}
                onChange={updateField}
                dynamicProps={testData.textareaField}
                readOnly={readOnly}
              />

              {/* Combobox Field */}
              <DynamicFormField
                id={testData.comboboxField.id}
                current={current}
                initialValue={testData.comboboxField.initialValue}
                onChange={updateField}
                dynamicProps={testData.comboboxField}
                readOnly={readOnly}
              />

              {/* Item Select Field */}
              <DynamicFormField
                id={testData.itemselectField.id}
                current={current}
                initialValue={testData.itemselectField.initialValue}
                onChange={updateField}
                dynamicProps={testData.itemselectField}
                readOnly={readOnly}
              />

              {/* Combobox with storeApi */}
              <DynamicFormField
                id={testData.comboboxWithStoreApi.id}
                current={current}
                initialValue={testData.comboboxWithStoreApi.initialValue}
                onChange={updateField}
                dynamicProps={testData.comboboxWithStoreApi}
                readOnly={readOnly}
              />

              {/* Combobox with storeApi + filters */}
              <DynamicFormField
                id={testData.comboboxWithStoreApiFilters.id}
                current={current}
                initialValue={testData.comboboxWithStoreApiFilters.initialValue}
                onChange={updateField}
                dynamicProps={testData.comboboxWithStoreApiFilters}
                readOnly={readOnly}
              />

              {/* Combobox with storeApi - simple */}
              <DynamicFormField
                id={testData.comboboxWithStoreApiSimple.id}
                current={current}
                initialValue={testData.comboboxWithStoreApiSimple.initialValue}
                onChange={updateField}
                dynamicProps={testData.comboboxWithStoreApiSimple}
                readOnly={readOnly}
              />

              {/* Itemselect with storeApi - Repository */}
              <DynamicFormField
                id={testData.itemselectWithStoreApi.id}
                current={current}
                initialValue={testData.itemselectWithStoreApi.initialValue}
                onChange={updateField}
                dynamicProps={testData.itemselectWithStoreApi}
                readOnly={readOnly}
              />

              {/* Itemselect with storeApi - Global */}
              <DynamicFormField
                id={testData.itemselectWithStoreApiGlobal.id}
                current={current}
                initialValue={
                  testData.itemselectWithStoreApiGlobal.initialValue
                }
                onChange={updateField}
                dynamicProps={testData.itemselectWithStoreApiGlobal}
                readOnly={readOnly}
              />

              {/* Display current form data */}
              <AsFormGroup
                label="Current Form Data"
                sublabel="Real-time form state"
              >
                <pre
                  style={{
                    background: "#f5f5f5",
                    padding: "10px",
                    borderRadius: "4px",
                    fontSize: "12px",
                    overflow: "auto",
                    maxHeight: "200px",
                  }}
                >
                  {JSON.stringify(formData, null, 2)}
                </pre>
              </AsFormGroup>
            </AsForm>
          </AsTile.Content>
        </AsTile>
      </ContentBody>
    </Page>
  );
}