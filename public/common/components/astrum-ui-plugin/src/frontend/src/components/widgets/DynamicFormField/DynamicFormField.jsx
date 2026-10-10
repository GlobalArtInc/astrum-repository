
import React from 'react';
import { equals, mapObjIndexed, values } from 'ramda';
import { assign, createMachine } from 'xstate';
import { useMachine } from '@xstate/react';
import { AsFormSelect, AsTextInput, AsCheckbox, AsReadOnly, AsFormGroup } from '@globalart/astrum-react-shared-components';
import { AsStatefulTransferList } from '@globalart/astrum-react-shared-components';

import FormUtils from '../../../interface/FormUtils';
import ValidationUtils from '../../../interface/ValidationUtils';
import UIStrings from '../../../constants/UIStrings';
import ExtAPIUtils from '../../../interface/ExtAPIUtils';
import { AsLoadWrapper } from '@globalart/astrum-react-shared-components';

export default function DynamicFormField({ current, dynamicProps, id, initialValue, readOnly, onChange }) {
  // Note: storeApi, storeFilters, idMapping, nameMapping are legacy ExtJS fields
  // These will be removed once all ExtJS descriptors are migrated to React

  // Handle read-only fields - just display the value as text
  if (dynamicProps.readOnly || readOnly) {
    return readOnlyComponent(current, dynamicProps, id, initialValue);
  } else {
    if (dynamicProps.type === 'checkbox') {
      return editComponent(current, dynamicProps, id, initialValue, onChange)
    } else {
      return (
        <AsFormGroup
          label={dynamicProps.label}
          isRequired={dynamicProps.required}
          sublabel={dynamicProps.helpText}
        >
          {editComponent(current, dynamicProps, id, initialValue, onChange)}
        </AsFormGroup>
      );
    }
  }
}

function readOnlyComponent(current, dynamicProps, id, initialValue) {
  switch (dynamicProps.type) {
    case "string":
    case "number":
    case "url":
    case "combobox":
    case "text-area":
      return simpleReadOnlyComponent(
        id,
        dynamicProps.label || id,
        current.context.data[id] || initialValue || "",
        dynamicProps
      )
    case "checkbox":
      return simpleReadOnlyComponent(
        id,
        dynamicProps.label || id,
        current.context.data[id] || initialValue || false,
        dynamicProps
      )
    case "password":
      return simpleReadOnlyComponent(
        id,
        dynamicProps.label || id,
        "********",
        dynamicProps
      );
    case "itemselect":
      return listReadOnlyComponent(
        id,
        dynamicProps.label || id,
        (Array.isArray(current.context.data[id]) &&
        current.context.data[id].length > 0
          ? current.context.data[id]
          : null) ||
          initialValue ||
          [],
        dynamicProps
      );
    default: {
      console.warn(`form field type=${dynamicProps.type} is unknown`);
      return <div />;
    }
  }
}

function editComponent(current, dynamicProps, id, initialValue, onChange) {
  switch (dynamicProps.type) {
    case 'string': {
      const { fieldProps, className } = getFieldPropsAndClassName(id, current, initialValue, dynamicProps);
      const textInputProps = getTextInputProps(fieldProps, className, dynamicProps, 'text', onChange);

      return <AsTextInput {...textInputProps} />;
    }
    case 'itemselect': {
      return (
        <ItemSelectField
          id={id}
          current={current}
          initialValue={initialValue}
          dynamicProps={dynamicProps}
          onChange={onChange}
        />
      );
    }
    case 'combobox': {
      return (
        <ComboboxField
          id={id}
          current={current}
          initialValue={initialValue}
          dynamicProps={dynamicProps}
          onChange={onChange}
        />
      );
    }
    case 'number': {
      const { fieldProps, className } = getFieldPropsAndClassName(id, current, initialValue, dynamicProps);

      // Filter input to only allow digits
      const handleNumberChange = value => {
        const numericValue = value.replace(/[^\d]/g, '');
        onChange(fieldProps.name, numericValue);
      };

      const textInputProps = getTextInputProps(
        fieldProps,
        className,
        dynamicProps,
        'text',
        onChange,
        handleNumberChange
      );
      return <AsTextInput {...textInputProps} />;
    }
    case 'checkbox': {
      const checkboxProps = FormUtils.checkboxProps(id, current, initialValue || false);
      return (
        <AsCheckbox
          {...checkboxProps}
          disabled={dynamicProps.disabled}
          readOnly={dynamicProps.readOnly}
          onChange={value => onChange(checkboxProps.name, value)}
        >
          {dynamicProps.label}
        </AsCheckbox>
      );
    }
    case 'url': {
      const { fieldProps, className } = getFieldPropsAndClassName(id, current, initialValue, dynamicProps);

      // Validate URL using existing ValidationUtils
      const validateUrl = value => {
        if (!value) return null;
        return ValidationUtils.validateIsUrl(value);
      };

      const textInputProps = getTextInputProps(fieldProps, className, dynamicProps, 'text', onChange);
      return (
        <AsTextInput
          {...textInputProps}
          validatable={true}
          validationErrors={validateUrl(fieldProps.value) || fieldProps.validationErrors}
        />
      );
    }
    case 'password': {
      const { fieldProps, className } = getFieldPropsAndClassName(id, current, initialValue, dynamicProps);
      const textInputProps = getTextInputProps(fieldProps, className, dynamicProps, 'password', onChange);

      return <AsTextInput {...textInputProps} />;
    }
    case 'text-area': {
      const { fieldProps, className } = getFieldPropsAndClassName(id, current, initialValue, dynamicProps);
      const textInputProps = getTextInputProps(fieldProps, className, dynamicProps, 'textarea', onChange);

      return <AsTextInput {...textInputProps} />;
    }
    default: {
      console.warn(`form field type=${dynamicProps.type} is unknown`);
      return <div />;
    }
  }
}

function simpleReadOnlyComponent(id, label, value, dynamicProps = {}) {
  label = dynamicProps?.label || label;
  value = (typeof value === 'boolean' ? (value ? 'Enabled' : 'Disabled') : value) || 'NA';
  value = (value === 'true' && 'Enabled' || value === 'false' && 'Disabled') || value;

  return (
    <AsReadOnly>
      <AsReadOnly.Label id={id}>{label}</AsReadOnly.Label>
      <AsReadOnly.Data aria-labeledby={id}>{value}</AsReadOnly.Data>
    </AsReadOnly>
  );
}

function listReadOnlyComponent(id, label, values, dynamicProps = {}) {
  label = dynamicProps?.label || label;

  return (
    <AsReadOnly>
      <AsReadOnly.Label id={id}>{label}</AsReadOnly.Label>
      {Array.isArray(values) && values.length > 0 && values.map((val, index) => (
        <AsReadOnly.Data key={index} aria-labeledby={id}>{val}</AsReadOnly.Data>
      )) || <AsReadOnly.Data>NA</AsReadOnly.Data>}
    </AsReadOnly>
  );
}

// Helper function to get common field props and className
function getFieldPropsAndClassName(id, current, initialValue, dynamicProps) {
  const fieldProps = FormUtils.fieldProps(id, current, initialValue || '');
  const className = dynamicProps.attributes.long ? 'as-text-input--long' : '';
  return { fieldProps, className };
}

// Helper function to get common AsTextInput props
function getTextInputProps(fieldProps, className, dynamicProps, type, onChange, customOnChange) {
  return {
    ...fieldProps,
    className,
    type,
    disabled: dynamicProps.disabled,
    readOnly: dynamicProps.readOnly,
    autoComplete: dynamicProps.allowAutocomplete || 'off',
    onChange: customOnChange || (value => onChange(fieldProps.name, value)),
  };
}

// Helper function to get field errors following FormUtils.fieldProps pattern
function getFieldErrors(id, current, initialValue = '') {
  const fieldProps = FormUtils.fieldProps(id, current, initialValue);
  const validationErrors = current.context.validationErrors || {};
  const saveErrors = current.context.saveErrors || {};
  const saveErrorData = current.context.saveErrorData || {};

  const saveError = saveErrors[id];
  const savedValue = saveErrorData[id];
  const currentValue = current.context.data[id] ?? initialValue;
  const validationError = validationErrors[id];

  // Error display logic matching FormUtils.fieldProps exactly
  // Use Ramda's equals for deep equality comparison (handles arrays and objects)
  let errors = null;
  const valuesMatch = equals(savedValue, currentValue);
  if (Boolean(savedValue) && saveError && valuesMatch) {
    errors = saveError;
  } else if (Boolean(validationError)) {
    errors = validationError;
  }

  return { errors, validationError, fieldProps };
}

// Helper function to convert storeFilters to ExtAPI format
function convertStoreFiltersToApiOptions(storeFilters) {
  return storeFilters
    ? {
        filter: Object.entries(storeFilters).map(([key, value]) => ({
          property: key,
          value: value,
        })),
      }
    : null;
}

// Component to handle combobox field type with support for both static options and storeApi
function ComboboxField({ id, current, initialValue, dynamicProps, onChange }) {
  const { storeApi, storeFilters, idMapping, nameMapping, attributes, required } = dynamicProps;
  const selectedValue = current.context.data[id] || initialValue || '';
  const { errors, fieldProps } = getFieldErrors(id, current, initialValue || '');

  // Validate required field
  const validateCombobox = value => {
    if (required && (!value || value === '')) {
      return UIStrings.ERROR.FIELD_REQUIRED;
    }
    return null;
  };

  const displayError = fieldProps.isPristine ? null : validateCombobox(selectedValue) || errors;

  // Priority: Use attributes.options if available, otherwise fall back to storeApi
  if (attributes?.options) {
    return (
      <>
        <AsFormSelect
          {...fieldProps}
          validatable={true}
          validationErrors={displayError}
          onChange={value => onChange(id, value)}
        >
          <option />
          {values(
            mapObjIndexed(
              (v, k) => (
                <option key={k} value={k}>
                  {v}
                </option>
              ),
              attributes.options
            )
          )}
        </AsFormSelect>
        {displayError && (
          <div className='as-field-validation-message' role='alert'>
            {displayError}
          </div>
        )}
      </>
    );
  }

  // Fallback to storeApi if attributes.options is not provided
  if (storeApi) {
    const [action, method] = storeApi.split('.');
    const idProp = idMapping || 'id';
    const nameProp = nameMapping || 'name';

    // Convert storeFilters to ExtAPI format if provided
    const apiOptions = convertStoreFiltersToApiOptions(storeFilters);

    // Create a custom machine to load data from storeApi
    const [state] = useMachine(
      () =>
        createMachine(
          {
            id: 'ComboboxMachine',
            initial: 'fetching',
            states: {
              fetching: {
                invoke: {
                  src: 'fetch',
                  onDone: {
                    target: 'loaded',
                    actions: ['setData'],
                  },
                  onError: {
                    target: 'error',
                    actions: ['setError'],
                  },
                },
              },
              loaded: {},
              error: {},
            },
          },
          {
            actions: {
              setData: assign({
                data: (_, event) => {
                  const apiData = event.data || [];
                  // Convert array of objects to options map {id: name, id2: name2}
                  const optionsMap = {};
                  apiData.forEach(item => {
                    const itemId = item[idProp];
                    const itemName = item[nameProp];
                    if (itemId !== undefined && itemName !== undefined) {
                      optionsMap[itemId] = itemName;
                    }
                  });
                  return optionsMap;
                },
              }),
              setError: assign({
                error: (_, event) => event?.data?.message || UIStrings.ERROR.UNKNOWN,
              }),
            },
            services: {
              fetch: async () => {
                const response = await ExtAPIUtils.extAPIRequest(action, method, apiOptions);
                ExtAPIUtils.checkForError(response);
                return ExtAPIUtils.extractResult(response, []);
              },
            },
          }
        ),
      {
        context: {
          data: {},
          error: null,
        },
        devTools: false,
      }
    );

    const { data: options, error } = state.context;
    const isLoading = state.matches('fetching');

    if (error) {
      console.error('Failed to load combobox options:', error);
    }

    return (
      <>
        <AsLoadWrapper loading={isLoading} error={error}>
          <AsFormSelect
            {...fieldProps}
            validatable={true}
            validationErrors={displayError}
            onChange={value => onChange(id, value)}
          >
            <option />
            {values(
              mapObjIndexed(
                (v, k) => (
                  <option key={k} value={k}>
                    {v}
                  </option>
                ),
                options || {}
              )
            )}
          </AsFormSelect>
        </AsLoadWrapper>
        {displayError && (
          <div className='as-field-validation-message' role='alert'>
            {displayError}
          </div>
        )}
      </>
    );
  }

  // Neither attributes.options nor storeApi provided
  console.warn('combobox field missing both attributes.options and storeApi');
  return (
    <>
      <AsFormSelect
        {...fieldProps}
        validatable={true}
        validationErrors={displayError}
        onChange={value => onChange(id, value)}
      >
        <option />
      </AsFormSelect>
      {displayError && (
        <div className='as-field-validation-message' role='alert'>
          {displayError}
        </div>
      )}
    </>
  );
}

// Component to handle itemselect field type with support for both static options and storeApi
function ItemSelectField({ id, current, initialValue, dynamicProps, onChange }) {
  const { storeApi, storeFilters, idMapping, nameMapping, attributes } = dynamicProps;
  const selectedItems = current.context.data[id] || initialValue || [];
  const { errors, validationError, fieldProps } = getFieldErrors(id, current, initialValue || []);

  // Show errors if field is touched (not pristine) or if there's a validation error
  // Always show validation errors, even if field appears pristine (after submit, touchAllFields marks it as touched)
  // The validationErrors context is the source of truth after form submission
  const displayError = validationError ? errors : fieldProps.isPristine ? null : errors;

  // Priority: Use attributes.options if available, otherwise fall back to storeApi
  if (attributes?.options) {
    const allItems = attributes.options.map(it => ({ id: it, displayName: it }));

    return (
      <>
        <AsStatefulTransferList
          allItems={allItems}
          selectedItems={selectedItems}
          availableItemsLabel={attributes.fromTitle}
          selectedItemsLabel={attributes.toTitle}
          onChange={value => onChange(id, value)}
          allowReordering
        />
        {displayError && (
          <div className='as-field-validation-message' role='alert'>
            {displayError}
          </div>
        )}
      </>
    );
  }

  // Fallback to storeApi if attributes.options is not provided
  if (storeApi) {
    const [action, method] = storeApi.split('.');
    const idProp = idMapping || 'id';
    const nameProp = nameMapping || 'name';

    // Convert storeFilters to ExtAPI format if provided
    const apiOptions = convertStoreFiltersToApiOptions(storeFilters);

    // Create a custom machine to load data from storeApi
    const [state] = useMachine(
      () =>
        createMachine(
          {
            id: 'ItemSelectMachine',
            initial: 'fetching',
            states: {
              fetching: {
                invoke: {
                  src: 'fetch',
                  onDone: {
                    target: 'loaded',
                    actions: ['setData'],
                  },
                  onError: {
                    target: 'error',
                    actions: ['setError'],
                  },
                },
              },
              loaded: {},
              error: {},
            },
          },
          {
            actions: {
              setData: assign({
                data: (_, event) => {
                  const apiData = event.data || [];
                  return apiData.map(item => ({
                    id: item[idProp],
                    displayName: item[nameProp],
                  }));
                },
              }),
              setError: assign({
                error: (_, event) => event?.data?.message || UIStrings.ERROR.UNKNOWN,
              }),
            },
            services: {
              fetch: async () => {
                const response = await ExtAPIUtils.extAPIRequest(action, method, apiOptions);
                ExtAPIUtils.checkForError(response);
                return ExtAPIUtils.extractResult(response, []);
              },
            },
          }
        ),
      {
        context: {
          data: [],
          error: null,
        },
        devTools: false,
      }
    );

    const { data, error } = state.context;
    const isLoading = state.matches('fetching');

    if (error) {
      console.error('Failed to load itemselect options:', error);
    }

    // Show loading state or empty state while loading
    if (isLoading) {
      return (
        <>
          <AsStatefulTransferList
            allItems={[]}
            selectedItems={selectedItems}
            availableItemsLabel={attributes?.fromTitle || 'Available'}
            selectedItemsLabel={attributes?.toTitle || 'Selected'}
            onChange={value => onChange(id, value)}
            allowReordering
          />
          {displayError && (
            <div className='as-field-validation-message' role='alert'>
              {displayError}
            </div>
          )}
        </>
      );
    }

    return (
      <>
        <AsStatefulTransferList
          allItems={data || []}
          selectedItems={selectedItems}
          availableItemsLabel={attributes?.fromTitle || 'Available'}
          selectedItemsLabel={attributes?.toTitle || 'Selected'}
          onChange={value => onChange(id, value)}
          allowReordering
        />
        {displayError && (
          <div className='as-field-validation-message' role='alert'>
            {displayError}
          </div>
        )}
      </>
    );
  }

  // Neither attributes.options nor storeApi provided
  console.warn('itemselect field missing both attributes.options and storeApi');
  return (
    <>
      <AsStatefulTransferList
        allItems={[]}
        selectedItems={selectedItems}
        availableItemsLabel={attributes?.fromTitle || 'Available'}
        selectedItemsLabel={attributes?.toTitle || 'Selected'}
        onChange={value => onChange(id, value)}
        allowReordering
      />
      {displayError && (
        <div className='as-field-validation-message' role='alert'>
          {displayError}
        </div>
      )}
    </>
  );
}
